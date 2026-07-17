use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "unipack-cloud-cli-{}-{}",
        name,
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn write_payload_zip(path: &Path, payload: &[u8]) {
    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file(
        "payload.json",
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
    )
    .unwrap();
    zip.write_all(payload).unwrap();
    zip.finish().unwrap();
}

fn stderr_event(output: &std::process::Output) -> serde_json::Value {
    let stderr = String::from_utf8_lossy(&output.stderr);
    serde_json::from_str(stderr.trim())
        .unwrap_or_else(|error| panic!("CLI stderr is not one JSON event ({error}): {stderr}"))
}

#[test]
fn cli_rejects_v1_payload_with_structured_upgrade_message() {
    let root = temp_dir("legacy-payload");
    let payload_zip = root.join("payload.zip");
    write_payload_zip(&payload_zip, br#"{"version":1,"buildId":"legacy"}"#);

    let output = Command::new(env!("CARGO_BIN_EXE_unipack-cloud-build"))
        .arg(&payload_zip)
        .current_dir(&root)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let event = stderr_event(&output);
    assert_eq!(event["channel"], "build-log");
    assert_eq!(event["event"]["level"], "error");
    let message = event["event"]["message"].as_str().unwrap();
    assert!(message.contains("payload v1"));
    assert!(message.contains("升级桌面端"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn cli_redacts_github_token_from_top_level_errors() {
    let root = temp_dir("redaction");
    let token = "github-token-must-not-leak";
    let missing_payload = root.join(token).join("missing.zip");

    let output = Command::new(env!("CARGO_BIN_EXE_unipack-cloud-build"))
        .arg(&missing_payload)
        .current_dir(&root)
        .env("GH_TOKEN", token)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains(token));
    let event = stderr_event(&output);
    assert!(event["event"]["message"].as_str().unwrap().contains("***"));

    let _ = std::fs::remove_dir_all(root);
}
