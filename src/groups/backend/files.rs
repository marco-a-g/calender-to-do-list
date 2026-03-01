//! Supabase Storage backend for group files.
//!
//! Bucket: `group-files`
//! Layout (matches storage policy): `private/{group_id}/{filename}`
//!
//! This module provides server functions to:
//!  - List files in the bucket under `private/{group_id}/`
//!  - Upload a file to `group-files/private/{group_id}/{filename}`
//!  - Delete a file from the same path
//!  - Create a signed, time-limited download URL

use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use serde::Deserialize;

const BUCKET: &str = "group-files";
const ROOT_PREFIX: &str = "private";

/// (group_id, object_id, filename, uploaded_at_date)
pub type FileTransfer = (String, String, String, String);

// Partial representation of a Storage object as returned by the list endpoint
#[derive(Debug, Deserialize)]
struct StorageObject {
    id: String,
    name: String,
    created_at: String,
}

/// Lists all files for a group from the Supabase Storage bucket.
pub async fn fetch_files(
    group_id: String,
    access_token: String,
) -> Result<Vec<FileTransfer>, String> {
    let url = SUPABASE_URL;

    let endpoint = format!("{}/storage/v1/object/list/{}", url, BUCKET);
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "prefix": format!("{}/{}/", ROOT_PREFIX, group_id),
        "limit": 100
    });

    let response = client
        .post(&endpoint)
        .header("apikey", ANON_KEY)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(format!("Storage List Error: {}", err));
    }

    let objects: Vec<StorageObject> = response.json().await.map_err(|e| e.to_string())?;

    let files: Vec<FileTransfer> = objects
        .into_iter()
        .filter(|o| !o.name.ends_with("/"))
        .map(|o| {
            let filename = o.name.split('/').next_back().unwrap_or(&o.name).to_string();
            let date = o
                .created_at
                .split('T')
                .next()
                .unwrap_or(&o.created_at)
                .to_string();
            (group_id.clone(), o.id, filename, date)
        })
        .collect();

    Ok(files)
}

/// Uploads a file to the group's storage folder (upserts on conflict).
pub async fn upload_file(
    group_id: String,
    filename: String,
    file_data: Vec<u8>,
    content_type: String,
    access_token: String,
) -> Result<(), String> {
    let url = SUPABASE_URL;

    let file_path = format!("{}/{}/{}", ROOT_PREFIX, group_id, filename);
    let endpoint = format!("{}/storage/v1/object/{}/{}", url, BUCKET, file_path);

    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint)
        .header("apikey", ANON_KEY)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", &content_type)
        .header("x-upsert", "true")
        .body(file_data)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(format!("Upload Error: {}", err));
    }

    Ok(())
}

/// Deletes a file from the group's storage folder.
pub async fn delete_file(
    group_id: String,
    filename: String,
    access_token: String,
) -> Result<(), String> {
    let url = SUPABASE_URL;

    let file_path = format!("{}/{}/{}", ROOT_PREFIX, group_id, filename);
    let endpoint = format!("{}/storage/v1/object/{}/{}", url, BUCKET, file_path);

    let client = reqwest::Client::new();

    let response = client
        .delete(&endpoint)
        .header("apikey", ANON_KEY)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(format!("Delete Error: {}", err));
    }

    Ok(())
}

/// Creates a signed download URL valid for one hour.
pub async fn get_file_url(
    group_id: String,
    filename: String,
    access_token: String,
) -> Result<String, String> {
    let url = SUPABASE_URL;

    let encoded_filename = urlencoding::encode(&filename);
    let file_path = format!("{}/{}/{}", ROOT_PREFIX, group_id, encoded_filename);
    let endpoint = format!("{}/storage/v1/object/sign/{}/{}", url, BUCKET, file_path);

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "expiresIn": 3600
    });

    let response = client
        .post(&endpoint)
        .header("apikey", ANON_KEY)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Sign URL Error: {}", body_text));
    }

    #[derive(Deserialize)]
    struct SignedUrl {
        #[serde(rename = "signedURL")]
        signed_url: String,
    }

    let result: SignedUrl = serde_json::from_str(&body_text).map_err(|e| e.to_string())?;

    let signed = if result.signed_url.starts_with('/') {
        format!(
            "{}/storage/v1{}",
            url,
            result.signed_url.replace(' ', "%20")
        )
    } else {
        format!(
            "{}/storage/v1/{}",
            url,
            result.signed_url.replace(' ', "%20")
        )
    };

    Ok(signed)
}