use banner::utils::shutdown::join_tasks;
use tokio::task::JoinHandle;

#[tokio::test]
async fn test_join_tasks_success() {
    // Create some tasks that complete successfully
    let handles: Vec<JoinHandle<()>> = vec![
        tokio::spawn(async { tokio::time::sleep(tokio::time::Duration::from_millis(10)).await }),
        tokio::spawn(async { tokio::time::sleep(tokio::time::Duration::from_millis(20)).await }),
        tokio::spawn(async { /* immediate completion */ }),
    ];

    // All tasks should complete successfully
    let result = join_tasks(handles).await;
    assert!(result.is_ok(), "Expected all tasks to complete successfully");
}

#[tokio::test]
async fn test_join_tasks_with_panic() {
    // Create some tasks, including one that panics
    let handles: Vec<JoinHandle<()>> = vec![
        tokio::spawn(async { tokio::time::sleep(tokio::time::Duration::from_millis(10)).await }),
        tokio::spawn(async { panic!("intentional test panic") }),
        tokio::spawn(async { /* immediate completion */ }),
    ];

    // Should return an error because one task panicked
    let result = join_tasks(handles).await;
    assert!(result.is_err(), "Expected an error when a task panics");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("1 task(s) panicked"), "Error message should mention panicked tasks");
}
