use crate::data::consts::{LOSSLESS_COMPRESSED_FORMATS, LOSSLESS_UNCOMPRESSED_FORMATS, LOSSY_FORMATS};
use crate::utils::logging::Log;
use regex::Regex;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use crate::log;
use std::io::Write;

const SERVER_ADDR: &str = "127.0.0.1:80";
const MUSIC_DIRECTORY: &str = r#"K:\_MUSIC\HOUSE\"#;
const BUFFER_SIZE: usize = 8192;

fn get_files_from_disk() -> Vec<String> {
    let path = Path::new(MUSIC_DIRECTORY);
    let mut file_list = Vec::new();

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() &&
           let Some(filename) = path.file_name().and_then(|name| name.to_str())
        {
            file_list.push(filename.to_string());
        }
    }

    println!("[INFO] Found {} files in the directory.", file_list.len());
    file_list
}

fn manual_url_encode(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        match c {
            ' ' => result.push_str("%20"), // Space
            '/' => result.push_str("%2F"), // Forward slash (path separator)
            '?' => result.push_str("%3F"), // Question mark (query start)
            '&' => result.push_str("%26"), // Ampersand (used for parameters)
            '=' => result.push_str("%3D"), // Equals sign
            '+' => result.push_str("%2B"), // Plus sign (often used for spaces in query strings)
            '\'' => result.push_str("%27"),
            '#' => result.push_str("%23"), // Hash/Fragment identifier
            '<' => result.push_str("%3C"), // Less than
            '>' => result.push_str("%3E"), // Greater than

            _ => {
                    result.push(c);
            }
        }
    }
    result
}

fn manual_url_decode(encoded_input: &str) -> String {
    // Regex to find sequences like %XX where XX are hex digits.
    let re = Regex::new(r"%([0-9A-Fa-f]{2})").unwrap();

    re.replace_all(encoded_input, |caps: &regex::Captures| -> String {
        let hex_code = caps.get(1).unwrap().as_str();

        // 1. Attempt to decode the hex code into a byte, then cast to a char.
        let decoded_char = match u8::from_str_radix(hex_code, 16) {
            Ok(byte) => byte as char,
            Err(_) => {
                // 2. Fallback: If decoding fails, return the first character of the original match.
                caps[0].chars().next().unwrap()
            }
        };

        // 3. Convert the resulting char into a String
        decoded_char.to_string()
    })
        .into_owned()
}

fn generate_html_manifest(file_list: &[String]) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n<head><title>Mix Directory</title></head>\n<body style='font-family: Arial;'>" );
    html.push_str("<h1>Available Mixes</h1>\n");
    html.push_str("<p>Click a link below to download the media file:</p>\n");

    // Generate a list item for every file found
    for filename in file_list {
        // Construct the direct download URL using the MUSIC_DIRECTORY
        let encoded_filename = manual_url_encode(filename);
        let direct_link = format!("http://localhost:80/mixes/house/{}", encoded_filename);

        // Create the HTML link
        html.push_str(&format!(
            "<ul><li style='margin-bottom: 5px;'><a href=\"{}\" target=\"_blank\">{}</a></li></ul>",
            direct_link, filename
        ));
    }

    html.push_str("</body>\n</html>");
    html
}

fn extensions<'a>() -> Vec<&'a str> {
    let mut ext = Vec::from_iter(LOSSY_FORMATS);
    ext.extend(LOSSLESS_COMPRESSED_FORMATS);
    ext.extend(LOSSLESS_UNCOMPRESSED_FORMATS);
    ext
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; BUFFER_SIZE];

    // 1. Read the initial request line
    if let Ok(n) = stream.read(&mut buffer[..BUFFER_SIZE]).await {
        let request = String::from_utf8_lossy(&buffer[..n]);

        // --- LOGIC 1: Check for Direct File Download Request (Has a trailing filename) ---
        if request.starts_with("GET") && request.contains("/mixes/house") &&
           extensions().iter().any(|&ext| request.contains(&format!(".{ext}")))
        {
            // We need to isolate the path part: /mixes/house/FILENAME.ext

            // 1. Find the start of the path after the domain/port
            let path_start_index = request.find("/mixes/house/")
                .expect("Path segment '/mixes/house/' must exist in the request.");

            // 2. Extract the substring starting from the path segment
            let full_path_segment = request.lines().next().unwrap()[path_start_index..]
                .split_whitespace().next().unwrap();

            // 3. Extract just the filename by splitting on the first '/' after the base path
            let parts: Vec<&str> = full_path_segment.split('/').collect();

            if !parts.is_empty() {
                // The filename is the last element after splitting by '/'
                let requested_filename = parts.last().unwrap_or(&"");

                if !requested_filename.is_empty() {
                    // Delegate to the file download handler
                    handle_file_download(stream, requested_filename).await;
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nInvalid file path provided.";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            } else {
                let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nInvalid request format.";
                let _ = stream.write_all(response.as_bytes()).await;
            }
        }

        // --- LOGIC 2: Check for Manifest Request (No trailing filename) ---
        else if request.starts_with("GET") && request.contains("/mixes/house") {
            // This condition targets: GET /mixes/house

            // --- CORE LOGIC: Generate the Manifest ---
            let file_list = get_files_from_disk();
            let html_content = generate_html_manifest(&file_list);

            // Construct the HTTP Response
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html_content.len(),
                html_content
            );

            if let Err(e) = stream.write_all(response.as_bytes()).await {
                log!("[SERVER] Error writing manifest response: {}", e);
            }
        }

        else {
            // Handle requests that don't match the expected path
            let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nPage not found.";
            let _ = stream.write_all(response.as_bytes()).await;
        }
    }
}

async fn handle_file_download(mut stream: TcpStream, requested_path: &str) {
    let requested_path = manual_url_decode(requested_path);
    let full_path = Path::new(MUSIC_DIRECTORY).join(&requested_path);
    let mut file = tokio::fs::File::open(full_path).await.unwrap();
    let mut chunk = [0u8; BUFFER_SIZE];
    // let mut total_bytes_written = 0;

    // 1. Get the file size
    let file_size = file.metadata().await.unwrap().len();
    // 2. Construct the correct HTTP response
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: audio/flac\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        file_size
    );
    // 3. Send the header
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        log!("[SERVER] Error writing response header: {}", e);
        return;
    }

    loop {
        match file.read(&mut chunk).await {
            Ok(0) => {
                // log!("EOF");
                break;
            }
            Ok(n) => {
                if let Err(e) = stream.write_all(&chunk[..n]).await {
                    log!("[SERVER] Error writing chunk to stream: {e}");
                    break;
                }
                // total_bytes_written += n;
                // log!("write {n} bytes ({total_bytes_written})");
            }
            Err(e) => {
                log!("[SERVER] Error reading file chunk: {e}");
                break;
            }
        }
    }
}

async fn start_media_server() {
    let listener = TcpListener::bind(SERVER_ADDR).await.expect("Failed to bind to address");
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                handle_connection(stream).await;
            }
            Err(e) => {
                log!("Error accepting connection: {}", e);
            }
        }
    }
}

// -----------------------------------
// Run server and app separately
#[ignore]
#[tokio::test]
async fn start_test_media_server() {
    Log::init("server.log", 10 * 1024 * 1024 * 1024);
    start_media_server().await;
}

#[test]
fn encode_decode_test() {
    Log::init("server.log", 10 * 1024 * 1024 * 1024);
    let s = "Yorgy Simenon - I'm Staying [Mix 001] (promodj.com).flac";
    let encoded = manual_url_encode(s);
    assert_eq!(r#"Yorgy%20Simenon%20-%20I%27m%20Staying%20[Mix%20001]%20(promodj.com).flac"#, encoded);

    let decoded = manual_url_decode(&encoded);
    assert_eq!(s, decoded);
}