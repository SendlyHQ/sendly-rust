use std::path::Path;

use reqwest::multipart;
use tokio::fs;

use crate::client::Sendly;
use crate::error::{Error, Result};
use crate::models::MediaFile;

pub struct Media<'a> {
    client: &'a Sendly,
}

impl<'a> Media<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn upload(&self, file_path: &str) -> Result<MediaFile> {
        let path = Path::new(file_path);

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        let content_type = mime_from_extension(path);

        let data = fs::read(file_path).await.map_err(|e| Error::Validation {
            message: format!("Failed to read file: {}", e),
        })?;

        self.upload_bytes(data, &filename, &content_type).await
    }

    pub async fn upload_bytes(
        &self,
        data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<MediaFile> {
        let part = multipart::Part::bytes(data)
            .file_name(filename.to_string())
            .mime_str(content_type)
            .map_err(|e| Error::Validation {
                message: format!("Invalid content type: {}", e),
            })?;

        let form = multipart::Form::new().part("file", part);

        let response = self.client.post_multipart("/media", form).await?;
        let media_file: MediaFile = response.json().await?;

        Ok(media_file)
    }
}

fn mime_from_extension(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("png") => "image/png".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("webp") => "image/webp".to_string(),
        Some("mp4") => "video/mp4".to_string(),
        Some("3gp") => "video/3gpp".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("vcf") => "text/vcard".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
