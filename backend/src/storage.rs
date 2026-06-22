use s3::{Auth, Client, Credentials};

pub struct Storage {
    pub client: Client,
    pub bucket: String,
}

impl Storage {
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("S3_ENDPOINT");
        let bucket = std::env::var("S3_BUCKET").ok()?;
        let region = std::env::var("S3_REGION").unwrap_or_else(|_| "auto".into());
        let access_key = std::env::var("S3_ACCESS_KEY").ok()?;
        let secret_key = std::env::var("S3_SECRET_KEY").ok()?;

        let client = if let Ok(account_id) = std::env::var("R2_ACCOUNT_ID") {
            let preset = s3::providers::cloudflare_r2(
                &account_id,
                s3::providers::R2Endpoint::Global,
            )
            .ok()?;
            Client::builder(preset.endpoint())
                .ok()?
                .region(preset.region())
                .addressing_style(preset.addressing_style())
                .auth(Auth::Static(
                    Credentials::new(&access_key, &secret_key).ok()?,
                ))
                .build()
                .ok()?
        } else if let Ok(endpoint) = endpoint {
            Client::builder(&endpoint)
                .ok()?
                .region(&region)
                .auth(Auth::Static(
                    Credentials::new(&access_key, &secret_key).ok()?,
                ))
                .build()
                .ok()?
        } else {
            return None;
        };

        Some(Storage { client, bucket })
    }

    pub async fn put(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<(), String> {
        self.client
            .objects()
            .put(&self.bucket, key)
            .content_type(content_type)
            .map_err(|e| format!("S3 content type error: {}", e))?
            .body_bytes(data)
            .send()
            .await
            .map_err(|e| format!("S3 upload failed: {}", e))?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, String> {
        let result = self
            .client
            .objects()
            .get(&self.bucket, key)
            .send()
            .await
            .map_err(|e| format!("S3 get failed: {}", e))?;
        result
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("S3 read failed: {}", e))
    }

    pub async fn delete(&self, key: &str) -> Result<(), String> {
        self.client
            .objects()
            .delete(&self.bucket, key)
            .send()
            .await
            .map_err(|e| format!("S3 delete failed: {}", e))?;
        Ok(())
    }
}
