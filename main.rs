// Rust
use chrono::{SecondsFormat, Utc};
use reqwest::{header, Client};
use serde_json::Value;
use std::collections::HashSet;

static ACTION: &str = "promote";
static TARGET_ID: &str = "67e257a74abaefa8b4285fc5";
static JASMINE: &str = "67e218334abaefa8b4285dfb";
// Note: alice is defined in the python file but not used; omitted here.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let whitelist: HashSet<&str> = [TARGET_ID, JASMINE].iter().copied().collect();
    let client = Client::builder().build()?;

    // Build headers
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
    headers.insert("accept-language", header::HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
    headers.insert("origin", header::HeaderValue::from_static("https://www.uiucranked.com"));
    headers.insert("priority", header::HeaderValue::from_static("u=1, i"));
    headers.insert(header::REFERER, header::HeaderValue::from_static("https://www.uiucranked.com/"));
    headers.insert("sec-ch-ua", header::HeaderValue::from_static("\"Chromium\";v=\"134\", \"Not:A-Brand\";v=\"24\", \"Google Chrome\";v=\"134\""));
    headers.insert("sec-ch-ua-mobile", header::HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", header::HeaderValue::from_static("\"macOS\""));
    headers.insert("sec-fetch-dest", header::HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", header::HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", header::HeaderValue::from_static("same-origin"));
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"
    ));

    // Function to get other lists
    async fn get_other_lists(client: &Client, whitelist: &HashSet<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

    let mut other_ids = get_other_lists(&client, &whitelist).await?;

    for i in 0u64..1000000000000 {
        if ACTION == "promote" {
            let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
            if i % 50 == 0 {
                other_ids = get_other_lists(&client, &whitelist).await?;
            }
            if other_ids.is_empty() {
                println!("No other ids available");
                continue;
            }
            let id = &other_ids[0];

            // Request token
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

            // Update ELO
            let update_payload = serde_json::json!({
                "leftProfileId": TARGET_ID,
                "rightProfileId": id,
                "winner": "left",
                "timestamp": ts,
                "token": token
            });

            let update_resp = client
                .post("https://www.uiucranked.com/api/updateElo")
                .headers(headers.clone())
                .json(&update_payload)
                .send()
                .await?
                .json::<Value>()
                .await?;

            println!("{:?}", update_resp);
            if let Some(left_new_rating) = update_resp.get("leftNewRating") {
                println!("{:?}", left_new_rating);
            }
        }
    }

    Ok(())
}
