use crate::logic::file::DlFile;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::Duration;
use crate::logic::Command;

const TEST_FILENAME: &str = "some_file.ext";
const TEST_LINK: &str = "http://localhost/some_file.ext";
const EXPECTED_CONTENT: &[u8] = b"This is the content served by localhost.";

fn start_simple_server(addr: &str) -> JoinHandle<()> {
    let addr = addr.to_owned();
    tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = Vec::new();
            buf.extend(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n");
            buf.extend(EXPECTED_CONTENT);
            stream.write_all(&buf).await.unwrap();
        }
    })
}

#[tokio::test]
async fn test_download_via_localhost_integration() {

    // --------- Server side ----------
    let server_test_dir = tempdir().unwrap().path().join("test_dir");
    let server_test_file = server_test_dir.join(TEST_FILENAME);

    tokio::fs::create_dir_all(&server_test_dir).await.unwrap();
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&server_test_file).await.unwrap();
    file.write_all(EXPECTED_CONTENT).await.unwrap();
    assert!(server_test_file.exists());

    let handle = start_simple_server("localhost:80");
    // ------------------------------

    // Give the server a moment to start listening
    tokio::time::sleep(Duration::from_millis(100)).await;

    // -------- Client side ----------
    let (_, rx) = mpsc::unbounded_channel::<Command>();
    let control_rx = Arc::new(RwLock::new(rx));

    let (tx, _) = mpsc::unbounded_channel::<Command>();
    let common_tx = Arc::new(RwLock::new(tx));

    let cur_dir = std::env::current_dir().unwrap();
    let mut dl_file = DlFile::new(TEST_LINK, &cur_dir).unwrap();
    let client = reqwest::Client::new();
    let res = dl_file.download(client, true, common_tx.clone(), control_rx.clone(), 1, 1).await;

    assert!(res.is_ok(), "Download failed. Check if the server is running correctly.");

    let client_test_file = cur_dir.join(TEST_FILENAME);
    assert!(client_test_file.exists());

    let client_content = tokio::fs::read(&client_test_file).await.unwrap();
    assert_eq!(client_content, EXPECTED_CONTENT, "Downloaded content does not match expected content.");
    // ------------------------------

    // --- CLEANUP ---
    handle.await.unwrap();
    tokio::fs::remove_dir_all(&server_test_dir).await.unwrap();
    tokio::fs::remove_file(client_test_file).await.unwrap();
}