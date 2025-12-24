use futures::channel::mpsc;
use futures::SinkExt;
use reqwest::Proxy;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::types::{ApiResponse, CheckResult, ResultStatus};

const API_URL: &str = "https://api.hytl.tools/check";

pub fn is_valid_username(username: &str) -> bool {
    let len = username.len();
    (3..=16).contains(&len)
        && username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub async fn check_single_username(client: &reqwest::Client, username: String) -> CheckResult {
    if !is_valid_username(&username) {
        return CheckResult {
            username,
            status: ResultStatus::Invalid,
        };
    }

    let url = format!("{}/{}", API_URL, urlencoding::encode(&username));

    match client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            if response.status() == 429 {
                return CheckResult {
                    username,
                    status: ResultStatus::Error("Rate limited".into()),
                };
            }

            match response.json::<ApiResponse>().await {
                Ok(data) => CheckResult {
                    username,
                    status: if data.available.unwrap_or(false) {
                        ResultStatus::Available
                    } else {
                        ResultStatus::Taken
                    },
                },
                Err(e) => CheckResult {
                    username,
                    status: ResultStatus::Error(e.to_string()),
                },
            }
        }
        Err(e) => CheckResult {
            username,
            status: ResultStatus::Error(e.to_string()),
        },
    }
}

#[derive(Debug, Clone)]
pub enum CheckEvent {
    Result(CheckResult),
    Done,
}

pub fn check_usernames_stream(
    usernames: Vec<String>,
    proxies: Vec<String>,
    delay_ms: u64,
    concurrency: usize,
) -> mpsc::Receiver<CheckEvent> {
    let (mut tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let proxy_index = Arc::new(AtomicUsize::new(0));

        let clients: Vec<Arc<reqwest::Client>> = if proxies.is_empty() {
            vec![Arc::new(build_client(None))]
        } else {
            let built: Vec<_> = proxies
                .iter()
                .filter_map(|proxy_url| {
                    Proxy::all(proxy_url)
                        .ok()
                        .map(|proxy| Arc::new(build_client(Some(proxy))))
                })
                .collect();
            if built.is_empty() {
                vec![Arc::new(build_client(None))]
            } else {
                built
            }
        };

        for chunk in usernames.chunks(concurrency) {
            let futures: Vec<_> = chunk
                .iter()
                .map(|username| {
                    let idx = proxy_index.fetch_add(1, Ordering::SeqCst) % clients.len();
                    let client = Arc::clone(&clients[idx]);
                    let username = username.clone();
                    async move { check_single_username(&client, username).await }
                })
                .collect();

            let chunk_results = futures::future::join_all(futures).await;

            for result in chunk_results {
                let _ = tx.send(CheckEvent::Result(result)).await;
            }

            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        let _ = tx.send(CheckEvent::Done).await;
    });

    rx
}

fn build_client(proxy: Option<Proxy>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36");

    if let Some(p) = proxy {
        builder = builder.proxy(p);
    }

    builder.build().unwrap()
}
