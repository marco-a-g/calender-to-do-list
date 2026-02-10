/*
Supabase Storage backend for group files.
This module provides server functions to:
 - List files in the `group-files` bucket under a `group_id/` prefix
 - Upload a file to `group-files/{group_id}/{filename}`
 - Delete a file from the same path
 - Create a signed, time-limited download URL
 */

use dioxus::prelude::*;
use serde::Deserialize;

// (group_id, object_id, filename, uploaded_at_date)
pub type FileTransfer = (String, String, String, String);

// Partial representation of a Storage object as returned by the list endpoint
#[derive(Debug, Deserialize)]
struct StorageObject {
    id: String,
    name: String,
    created_at: String,
}

// Lists files in the `group-files` bucket for a given group
// The bucket layout is: `{group_id}/{filename}`
//#[server]
pub async fn fetch_files(group_id: String) -> Result<Vec<FileTransfer>, ServerFnError> {
    let url = std::env::var("SUPABASE_URL").map_err(|e| ServerFnError::new(e.to_string()))?;
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Storage list endpoint for a bucket. We use POST with a JSON body containing the prefix
    let endpoint = format!("{}/storage/v1/object/list/group-files", url);

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "prefix": format!("{}/", group_id),
        "limit": 100
    });

    let response = client
        .post(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Bubble up Supabase error bodies (permissions/validation) to simplify debugging
    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Storage List Error: {}", err)));
    }

    let objects: Vec<StorageObject> = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Convert Storage objects into the compact frontend transfer type
    let files: Vec<FileTransfer> = objects
        .into_iter()
        .filter(|o| !o.name.ends_with("/"))
        .map(|o| {
            let filename = o.name.split('/').last().unwrap_or(&o.name).to_string();
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

// Uploads a file to `group-files/{group_id}/{filename}`
//#[server]
pub async fn upload_file(
    group_id: String,
    filename: String,
    file_data: Vec<u8>,
    content_type: String,
) -> Result<(), ServerFnError> {
    let url = std::env::var("SUPABASE_URL").map_err(|e| ServerFnError::new(e.to_string()))?;
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Object path inside the bucket. Slashes create a virtual folder structure in Storage
    let file_path = format!("{}/{}", group_id, filename);
    let endpoint = format!("{}/storage/v1/object/group-files/{}", url, file_path);

    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", &content_type)
        .header("x-upsert", "true")
        .body(file_data)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Upload Error: {}", err)));
    }

    Ok(())
}

// Deletes a file at `group-files/{group_id}/{filename}`
//#[server]
pub async fn delete_file(group_id: String, filename: String) -> Result<(), ServerFnError> {
    let url = std::env::var("SUPABASE_URL").map_err(|e| ServerFnError::new(e.to_string()))?;
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let file_path = format!("{}/{}", group_id, filename);
    let endpoint = format!("{}/storage/v1/object/group-files/{}", url, file_path);

    let client = reqwest::Client::new();

    let response = client
        .delete(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Delete Error: {}", err)));
    }

    Ok(())
}

// Creates a signed, time-limited download URL for a private object
//#[server]
pub async fn get_file_url(group_id: String, filename: String) -> Result<String, ServerFnError> {
    let url = std::env::var("SUPABASE_URL").map_err(|e| ServerFnError::new(e.to_string()))?;
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Encode the filename so spaces and special characters don't break the URL path
    let encoded_filename = urlencoding::encode(&filename);
    let endpoint = format!(
        "{}/storage/v1/object/sign/group-files/{}/{}",
        url, group_id, encoded_filename
    );

    println!("DEBUG get_file_url:");
    println!("  endpoint: {}", endpoint);

    let client = reqwest::Client::new();

    // Signed URLs expire after the given number of seconds
    let body = serde_json::json!({
        "expiresIn": 3600
    });

    let response = client
        .post(&endpoint)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    println!("DEBUG response: status={}, body={}", status, body_text);

    if !status.is_success() {
        return Err(ServerFnError::new(format!("Sign URL Error: {}", body_text)));
    }

    // Response shape from Supabase Storage sign endpoint
    #[derive(Deserialize)]
    struct SignedUrl {
        #[serde(rename = "signedURL")]
        signed_url: String,
    }

    let result: SignedUrl =
        serde_json::from_str(&body_text).map_err(|e| ServerFnError::new(e.to_string()))?;

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

    println!("DEBUG signed URL: {}", signed);

    Ok(signed)
}
