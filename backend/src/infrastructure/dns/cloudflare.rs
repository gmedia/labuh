use crate::domain::dns_provider::DnsProvider;
use crate::domain::models::dns::RemoteDnsRecord;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde_json;

pub struct CloudflareProvider {
    api_token: String,
    zone_id: String,
    client: reqwest::Client,
}

impl CloudflareProvider {
    pub fn new(api_token: String, zone_id: String) -> Self {
        Self {
            api_token,
            zone_id,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl DnsProvider for CloudflareProvider {
    async fn create_record(&self, domain: &str, target: &str) -> Result<String> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.zone_id
        );

        let record_type = if target.parse::<std::net::IpAddr>().is_ok() {
            "A"
        } else {
            "CNAME"
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(&serde_json::json!({
                "type": record_type,
                "name": domain,
                "content": target,
                "ttl": 1, // Auto
                "proxied": false
            }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Cloudflare API error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Cloudflare API error ({}): {}",
                status, error_text
            )));
        }

        let body: serde_json::Value = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse Cloudflare response: {}", e))
        })?;

        let id = body["result"]["id"].as_str().ok_or_else(|| {
            AppError::Internal("Cloudflare response missing record ID".to_string())
        })?;

        Ok(id.to_string())
    }

    async fn delete_record(&self, record_id: &str) -> Result<()> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.zone_id, record_id
        );

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Cloudflare API error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Cloudflare API error ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }

    async fn list_available_base_domains(&self) -> Result<Vec<String>> {
        let zones = self.fetch_all_zones().await?;
        Ok(zones.into_iter().map(|z| z.name).collect())
    }

    async fn list_records(&self) -> Result<Vec<RemoteDnsRecord>> {
        let zones = self.fetch_all_zones().await?;
        let mut all_records = Vec::new();

        for zone in zones {
            let url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                zone.id
            );

            let response = self
                .client
                .get(&url)
                .bearer_auth(&self.api_token)
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("Cloudflare API error: {}", e)))?;

            if !response.status().is_success() {
                continue; // Skip failed zones
            }

            let body: serde_json::Value = response.json().await.map_err(|e| {
                AppError::Internal(format!("Failed to parse Cloudflare response: {}", e))
            })?;

            if let Some(records) = body["result"].as_array() {
                for r in records {
                    all_records.push(RemoteDnsRecord {
                        id: r["id"].as_str().unwrap_or_default().to_string(),
                        name: r["name"].as_str().unwrap_or_default().to_string(),
                        content: r["content"].as_str().unwrap_or_default().to_string(),
                        r#type: r["type"].as_str().unwrap_or_default().to_string(),
                        zone_id: zone.id.clone(),
                        zone_name: zone.name.clone(),
                    });
                }
            }
        }

        Ok(all_records)
    }
}

struct CloudflareZone {
    id: String,
    name: String,
}

impl CloudflareProvider {
    async fn fetch_all_zones(&self) -> Result<Vec<CloudflareZone>> {
        let url = "https://api.cloudflare.com/client/v4/zones?status=active";

        let response = self
            .client
            .get(url)
            .bearer_auth(&self.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Cloudflare API error: {}", e)))?;

        let body: serde_json::Value = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse Cloudflare response: {}", e))
        })?;

        let zones_data = body["result"].as_array().ok_or_else(|| {
            AppError::Internal("Cloudflare response missing zones list".to_string())
        })?;

        let zones: Vec<CloudflareZone> = zones_data
            .iter()
            .filter_map(|z| {
                Some(CloudflareZone {
                    id: z["id"].as_str()?.to_string(),
                    name: z["name"].as_str()?.to_string(),
                })
            })
            .collect();

        Ok(zones)
    }
}
