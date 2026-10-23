use crate::models::{DiscogsSearchResponse, DiscogsSearchResult};
use anyhow::{Context, Result};
use reqwest::Client;

pub struct DiscogsFetcher {
    client: Client,
    token: String,
}

impl DiscogsFetcher {
    pub fn new() -> Result<Self> {
        let token = std::env::var("DISCOGS_TOKEN").unwrap_or_default();
        let client = Client::builder()
            .user_agent("Datamile/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client, token })
    }

    pub async fn search_masters(
        &self,
        artist: &str,
        album: &str,
    ) -> Result<Vec<DiscogsSearchResult>> {
        let mut req = self
            .client
            .get("https://api.discogs.com/database/search")
            .query(&[
                ("type", "master"),
                ("artist", artist),
                ("release_title", album),
                ("per_page", "5"),
            ]);

        if !self.token.is_empty() {
            let token = &self.token;
            req = req.header("Authorization", format!("Discogs token={token}"));
        }

        let resp = req.send().await.context("Search request failed")?;
        if resp.status().is_success() {
            let parsed: DiscogsSearchResponse =
                resp.json().await.context("Search JSON parse error")?;
            if !parsed.results.is_empty() {
                return Ok(parsed.results);
            }
        }

        let query_fallback = format!("{artist} {album}");
        let mut fallback_req = self
            .client
            .get("https://api.discogs.com/database/search")
            .query(&[
                ("type", "master"),
                ("q", &query_fallback),
                ("per_page", "5"),
            ]);

        if !self.token.is_empty() {
            let token = &self.token;
            fallback_req =
                fallback_req.header("Authorization", format!("Discogs token={token}"));
        }

        let fallback_resp = fallback_req
            .send()
            .await
            .context("Fallback search request failed")?;
        if fallback_resp.status().is_success() {
            let parsed: DiscogsSearchResponse = fallback_resp
                .json()
                .await
                .context("Fallback search JSON parse error")?;
            return Ok(parsed.results);
        }

        Ok(Vec::new())
    }

    pub async fn fetch_master_detail(&self, master_id: u64) -> Result<serde_json::Value> {
        let url = format!("https://api.discogs.com/masters/{master_id}");
        let mut req = self.client.get(&url);

        if !self.token.is_empty() {
            let token = &self.token;
            req = req.header("Authorization", format!("Discogs token={token}"));
        }

        let resp = req.send().await.context("Fetch master details failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            anyhow::bail!("Discogs API returned status {status}");
        }

        let json_val: serde_json::Value = resp
            .json()
            .await
            .context("Master details JSON parse error")?;
        Ok(json_val)
    }
}
