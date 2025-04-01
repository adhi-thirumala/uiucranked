use chrono::{SecondsFormat, Utc};
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::{Client, header};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

static ACTION: &str = "promote";
static TARGET_ID: &str = "67e257a74abaefa8b4285fc5";
static JASMINE: &str = "67e218334abaefa8b4285dfb";

// Moved helper to top level so it's 'static for tokio::spawn.
async fn get_other_lists(
  client: &Client,
  whitelist: &HashSet<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
  let resp = client
    .get("https://www.uiucranked.com/api/getLeaderboard")
    .send()
    .await?
    .json::<Value>()
    .await?;
  let items = resp.as_array().ok_or("Expected JSON array")?;
  let mut list = Vec::new();
  for item in items {
    if let Some(id) = item.get("_id").and_then(|v| v.as_str()) {
      if id != TARGET_ID && !whitelist.contains(id) {
        list.push(id.to_string());
      }
    }
  }
  Ok(list)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let whitelist: HashSet<&str> = [TARGET_ID, JASMINE].iter().copied().collect();
  let client = Client::builder().build()?;

  // Build headers
  let mut headers = header::HeaderMap::new();
  headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
  headers.insert(
    "accept-language",
    header::HeaderValue::from_static("en-US,en;q=0.9"),
  );
  headers.insert(
    header::CONTENT_TYPE,
    header::HeaderValue::from_static("application/json"),
  );
  headers.insert(
    "origin",
    header::HeaderValue::from_static("https://www.uiucranked.com"),
  );
  headers.insert("priority", header::HeaderValue::from_static("u=1, i"));
  headers.insert(
    header::REFERER,
    header::HeaderValue::from_static("https://www.uiucranked.com/"),
  );
  headers.insert(
    "sec-ch-ua",
    header::HeaderValue::from_static(
      "\"Chromium\";v=\"134\", \"Not:A-Brand\";v=\"24\", \"Google Chrome\";v=\"134\"",
    ),
  );
  headers.insert("sec-ch-ua-mobile", header::HeaderValue::from_static("?0"));
  headers.insert(
    "sec-ch-ua-platform",
    header::HeaderValue::from_static("\"macOS\""),
  );
  headers.insert("sec-fetch-dest", header::HeaderValue::from_static("empty"));
  headers.insert("sec-fetch-mode", header::HeaderValue::from_static("cors"));
  headers.insert(
    "sec-fetch-site",
    header::HeaderValue::from_static("same-origin"),
  );
  headers.insert(header::USER_AGENT, header::HeaderValue::from_static(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"
    ));

  // Initialize shared state for other_ids using a Mutex.
  let initial_ids = get_other_lists(&client, &whitelist).await?;
  let other_ids = Arc::new(Mutex::new(initial_ids));
  const NUM_REQUESTS: u64 = 1000; // adjust as needed

  let mut tasks = FuturesUnordered::new();

  for i in 0..NUM_REQUESTS {
    let client = client.clone();
    let headers = headers.clone();
    let other_ids = Arc::clone(&other_ids);
    let whitelist = whitelist.clone();

    tasks.push(tokio::spawn(async move {
      let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

      // Refresh the list every 50 iterations.
      if i % 50 == 0 {
        if let Ok(new_ids) = get_other_lists(&client, &whitelist).await {
          let mut ids = other_ids.lock().await;
          *ids = new_ids;
        }
      }

      // Get id to use.
      let id = {
        let ids = other_ids.lock().await;
        if !ids.is_empty() {
          ids[0].clone()
        } else {
          eprintln!("No other ids available");
          return Ok(());
        }
      };

      // Request token.
      let payload = serde_json::json!({
          "leftProfileId": TARGET_ID,
          "rightProfileId": id,
          "timestamp": ts
      });

      let token_resp = client
        .post("https://www.uiucranked.com/api/getToken")
        .headers(headers.clone())
        .json(&payload)
        .send()
        .await?
        .json::<Value>()
        .await?;

      let token = token_resp
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap_or("");

      // Update ELO.
      let update_payload = serde_json::json!({
          "leftProfileId": TARGET_ID,
          "rightProfileId": id,
          "winner": "left",
          "timestamp": ts,
          "token": token
      });

      let update_resp = client
        .post("https://www.uiucranked.com/api/updateElo")
        .headers(headers)
        .json(&update_payload)
        .send()
        .await?
        .json::<Value>()
        .await?;

      println!("{:?}", update_resp);
      if let Some(left_new_rating) = update_resp.get("leftNewRating") {
        println!("{:?}", left_new_rating);
      }
      Ok::<(), Box<dyn std::error::Error>>(())
    }));
  }

  // Await all spawned tasks.
  while let Some(res) = tasks.next().await {
    if let Err(e) = res {
      eprintln!("Task error: {:?}", e);
    }
  }

  Ok(())
}

