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
