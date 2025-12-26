use director_plan::server;
use std::fs;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_verification_failure_artifacts() -> anyhow::Result<()> {
    // 1. Setup Temp Dir
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path().to_path_buf();

    // Setup directories
    fs::create_dir_all(root.join("plan/tickets"))?;
    fs::create_dir_all(root.join("assets"))?;
    fs::create_dir_all(root.join("tests/snapshots"))?;

    // 2. Create Dummy Golden Image
    let golden_path = root.join("tests/snapshots/test.png");
    fs::write(&golden_path, "golden bytes")?;

    // 3. Create Ticket with failing command that generates artifacts
    // We use a simple shell command.
    let cmd = if cfg!(target_os = "windows") {
        "echo actual > actual.png; echo diff > diff.png; exit 1" // Powershell might need different syntax for exit?
    } else {
        "echo actual > actual.png; echo diff > diff.png; exit 1"
    };

    // For powershell, `echo "actual" > actual.png` works. `exit 1` works.
    // But `director-plan` runs `powershell -Command ...`.
    // If we use `&&` or `;` it works in PS too? PS uses `;`.

    let ticket_content = format!(r#"
[meta]
id = "T-TEST"
title = "Test Ticket"
status = "todo"
priority = "low"
owner = "tester"
created_at = 2024-01-01T00:00:00Z

[spec]
description = "desc"
constraints = []
relevant_files = []

[verification]
command = "sh -c '{}'"
golden_image = "tests/snapshots/test.png"

[history]
log = []
"#, cmd);

    // Adjust for windows if needed, but assuming unix for now or basic shell.
    // director-plan server uses "sh -c" on non-windows.

    fs::write(root.join("plan/tickets/T-TEST.toml"), ticket_content)?;

    // 5. Start Server
    let app = server::create_app(root.clone()).await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let port = addr.port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 6. Verify
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/api/tickets/T-TEST/verify", port);

    let res = client.post(&url).send().await?;
    let status = res.status();
    assert_eq!(status, 200);

    let body: serde_json::Value = res.json().await?;
    println!("Response: {}", body);

    // Check success is false (because exit 1)
    assert_eq!(body["success"], false);

    // Check artifacts path
    assert_eq!(body["artifacts_path"], "/artifacts/T-TEST");

    // 7. Check files exist in target artifacts
    let artifacts_dir = root.join("target/public/artifacts/T-TEST");
    assert!(artifacts_dir.join("golden.png").exists(), "Golden image missing");
    assert!(artifacts_dir.join("actual.png").exists(), "Actual image missing");
    assert!(artifacts_dir.join("diff.png").exists(), "Diff image missing");

    // Verify content
    let actual_read = fs::read_to_string(artifacts_dir.join("actual.png"))?;
    assert!(actual_read.trim().contains("actual"));

    Ok(())
}
