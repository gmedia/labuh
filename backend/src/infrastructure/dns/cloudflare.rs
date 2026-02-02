use crate::domain::dns_provider::DnsProvider;
use crate::domain::models::dns::RemoteDnsRecord;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde_json;

pub struct CloudflareProvider {
    api_token: String,
    client: reqwest::Client,
}

impl CloudflareProvider {
    pub fn new(api_token: String) -> Self {
        Self {
            api_token,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl DnsProvider for CloudflareProvider {
    async fn create_record(
        &self,
        domain: &str,
        record_type: &str,
        content: &str,
    ) -> Result<String> {
        let zone_id = self.get_zone_id(domain).await?;
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(&serde_json::json!({
                "type": record_type,
                "name": domain,
                "content": content,
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

    async fn delete_record(&self, domain: &str, record_id: &str) -> Result<()> {
        let zone_id = self.get_zone_id(domain).await?;
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
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

    async fn update_record(
        &self,
        domain: &str,
        record_id: &str,
        record_type: &str,
        content: &str,
    ) -> Result<()> {
        let zone_id = self.get_zone_id(domain).await?;
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        );

        let response = self
            .client
            .put(&url)
            .bearer_auth(&self.api_token)
            .json(&serde_json::json!({
                "type": record_type,
                "name": domain,
                "content": content,
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

        Ok(())
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

    /// Find the best matching zone for a domain (longest suffix match)
    async fn get_zone_id(&self, domain: &str) -> Result<String> {
        let zones = self.fetch_all_zones().await?;
        let mut best_match: Option<CloudflareZone> = None;

        for zone in zones {
            // Check if domain ends with zone name (e.g. sub.example.com ends with example.com)
            // We need to handle the root case carefully.
            if domain == zone.name || domain.ends_with(&format!(".{}", zone.name)) {
                match &best_match {
                    None => best_match = Some(zone),
                    Some(current) => {
                        // Pick the longer one (more specific)
                        if zone.name.len() > current.name.len() {
                            best_match = Some(zone);
                        }
                    }
                }
            }
        }

        match best_match {
            Some(z) => Ok(z.id),
            None => Err(AppError::Validation(format!(
                "No active Cloudflare zone found for domain: {}",
                domain
            ))),
        }
    }
}
