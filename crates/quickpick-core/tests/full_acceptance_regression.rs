use quickpick_core::{ai, app_settings::AppSettings};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

// All requests and responses use fixed synthetic data on loopback only.
struct Server {
    base: String,
    release: mpsc::Sender<()>,
    worker: thread::JoinHandle<(String, Value)>,
}
impl Server {
    fn new(status: u16, content: &str, hold_response: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let body =
            serde_json::to_vec(&json!({"choices":[{"message":{"content":content}}]})).unwrap();
        let (release, wait) = mpsc::channel();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            Instant::now() < deadline,
                            "loopback request was not received"
                        );
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => panic!("loopback accept failed"),
                }
            };
            // Accepted sockets can inherit the listener's nonblocking mode on Windows.
            // Poll only accept; request reads below use their bounded blocking timeout.
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let (header_end, length) = loop {
                let mut chunk = [0; 4096];
                let count = stream.read(&mut chunk).unwrap();
                assert!(count > 0, "incomplete loopback request");
                bytes.extend_from_slice(&chunk[..count]);
                assert!(bytes.len() < 1024 * 1024, "loopback request exceeded limit");
                if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = std::str::from_utf8(&bytes[..end]).unwrap();
                    let length = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap();
                    break (end + 4, length);
                }
            };
            while bytes.len() < header_end + length {
                let mut chunk = [0; 4096];
                let count = stream.read(&mut chunk).unwrap();
                assert!(count > 0, "incomplete loopback body");
                bytes.extend_from_slice(&chunk[..count]);
            }
            let path = std::str::from_utf8(&bytes[..header_end])
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap()
                .to_string();
            let request = serde_json::from_slice(&bytes[header_end..header_end + length]).unwrap();
            if hold_response {
                let _ = wait.recv_timeout(Duration::from_secs(5));
            }
            let header = format!(
                "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&body);
            (path, request)
        });
        Self {
            base,
            release,
            worker,
        }
    }
    fn settings(&self) -> AppSettings {
        AppSettings {
            text_ai_base_url: format!("{}/text", self.base),
            text_ai_model: "fixture-text".into(),
            input_ai_base_url: format!("{}/input", self.base),
            input_ai_model: "fixture-input".into(),
            vision_ai_base_url: format!("{}/vision", self.base),
            vision_ai_model: "fixture-vision".into(),
            ..AppSettings::default()
        }
    }
    fn finish(self) -> (String, Value) {
        let _ = self.release.send(());
        self.worker.join().unwrap()
    }
}
async fn invoke(domain: u8, settings: &AppSettings) -> Result<String, String> {
    match domain {
        0 => {
            ai::run_text_action(
                settings,
                "fixture-key",
                "translate",
                "fixture",
                "auto",
                "en",
            )
            .await
        }
        1 => {
            ai::run_input_text_action(
                settings,
                "fixture-key",
                "translate",
                "fixture",
                "auto",
                "en",
            )
            .await
        }
        _ => {
            ai::run_image_action(
                settings,
                "fixture-key",
                "translate",
                b"synthetic-image",
                "auto",
                "en",
            )
            .await
        }
    }
}

#[tokio::test]
async fn successful_response_preserves_leading_indentation_and_trailing_newline() {
    for domain in 0..3 {
        for expected in [
            "    fixture\n",
            "\tfixture\r\n\n",
            "  i18n:ai.requestFailed\n",
        ] {
            let server = Server::new(200, expected, false);
            let result = invoke(domain, &server.settings()).await;
            server.finish();
            assert!(
                result.is_ok_and(|actual| actual == expected),
                "successful content formatting changed in domain {domain}"
            );
        }
    }
}

#[tokio::test]
async fn whitespace_only_success_is_still_an_empty_response() {
    for domain in 0..3 {
        for content in ["", " \t\r\n"] {
            let server = Server::new(200, content, false);
            let result = invoke(domain, &server.settings()).await;
            server.finish();
            assert!(result.is_err_and(|error| error == "i18n:ai.emptyResponse"));
        }
    }
}

#[tokio::test]
async fn each_domain_uses_its_own_endpoint_and_model() {
    for (domain, name) in [(0, "text"), (1, "input"), (2, "vision")] {
        let server = Server::new(200, "i18n:ai.requestFailed", false);
        let result = invoke(domain, &server.settings()).await;
        let (path, request) = server.finish();
        assert!(result.is_ok_and(|content| content == "i18n:ai.requestFailed"));
        assert!(
            path == format!("/{name}/chat/completions"),
            "wrong domain endpoint"
        );
        assert!(
            request["model"] == format!("fixture-{name}"),
            "wrong domain model"
        );
    }
}

#[tokio::test]
async fn http_failures_return_localized_errors_in_each_domain() {
    for domain in 0..3 {
        for (status, key) in [
            (400, "httpBadRequest"),
            (401, "httpUnauthorized"),
            (404, "httpNotFound"),
            (408, "httpTimeout"),
            (429, "httpRateLimited"),
            (500, "httpServerError"),
        ] {
            let server = Server::new(status, "synthetic failure", false);
            let result = invoke(domain, &server.settings()).await;
            server.finish();
            let key = if domain == 2 && status == 400 {
                "visionRejected"
            } else {
                key
            };
            assert!(
                result.is_err_and(|error| error == format!("i18n:ai.{key}")),
                "wrong error mapping in domain {domain}, HTTP {status}"
            );
        }
    }
}

#[tokio::test]
async fn actual_request_timeout_does_not_prevent_an_independent_retry() {
    let server = Server::new(200, "late fixture", true);
    let mut settings = server.settings();
    settings.ai_timeout_seconds = 1;
    let result = invoke(0, &settings).await;
    server.finish();
    assert!(result.is_err_and(|error| error == "i18n:ai.requestTimeout"));
    let retry = Server::new(200, "retry fixture", false);
    let result = invoke(0, &retry.settings()).await;
    retry.finish();
    assert!(result.is_ok_and(|content| content == "retry fixture"));
}
