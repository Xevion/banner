use tokio::task::JoinHandle;
use tracing::warn;

/// Helper for joining multiple task handles with proper error handling.
///
/// This function waits for all tasks to complete and reports any that panicked.
/// Returns an error if any task panicked, otherwise returns Ok.
pub async fn join_tasks(handles: Vec<JoinHandle<()>>) -> Result<(), anyhow::Error> {
    let results = futures::future::join_all(handles).await;

    let failed = results.iter().filter(|r| r.is_err()).count();
    if failed > 0 {
        warn!(failed_count = failed, "Some tasks panicked during shutdown");
        Err(anyhow::anyhow!("{} task(s) panicked", failed))
    } else {
        Ok(())
    }
}

/// Helper for joining multiple task handles with a timeout.
///
/// Waits for all tasks to complete within the specified timeout.
/// If timeout occurs, remaining tasks are aborted.
pub async fn join_tasks_with_timeout(
    handles: Vec<JoinHandle<()>>,
    timeout: std::time::Duration,
) -> Result<(), anyhow::Error> {
    match tokio::time::timeout(timeout, join_tasks(handles)).await {
        Ok(result) => result,
        Err(_) => Err(anyhow::anyhow!("Task join timed out after {:?}", timeout)),
    }
}
