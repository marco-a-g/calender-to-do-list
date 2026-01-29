use dioxus::prelude::*;
use std::sync::{LazyLock, Mutex};

// (group_id, file_id, filename, uploaded_at)
pub type FileTransfer = (String, i32, String, String);

#[derive(Clone, Debug)]
pub struct GroupFile {
    pub group_id: String,
    pub file_id: i32,
    pub filename: String,
    pub uploaded_at: String,
}

pub(crate) static MOCK_FILES: LazyLock<Mutex<Vec<GroupFile>>> = LazyLock::new(|| {
    Mutex::new(vec![
        GroupFile {
            group_id: "00000000-0000-0000-0000-000000000011".to_string(),
            file_id: 1,
            filename: "Sprint-Plan.pdf".to_string(),
            uploaded_at: "07.01.2026".to_string(),
        },
        GroupFile {
            group_id: "00000000-0000-0000-0000-000000000011".to_string(),
            file_id: 2,
            filename: "Designs.zip".to_string(),
            uploaded_at: "06.01.2026".to_string(),
        },
    ])
});

fn to_transfer(file: &GroupFile) -> FileTransfer {
    (
        file.group_id.clone(),
        file.file_id,
        file.filename.clone(),
        file.uploaded_at.clone(),
    )
}

#[server]
pub async fn fetch_files(group_id: String) -> Result<Vec<FileTransfer>, ServerFnError> {
    let files = MOCK_FILES
        .lock()
        .map_err(|_| ServerFnError::new("FILES mutex poisoned"))?;

    Ok(files
        .iter()
        .filter(|f| f.group_id == group_id)
        .map(to_transfer)
        .collect())
}

#[server]
pub async fn upload_file_mock(group_id: String, filename: String) -> Result<(), ServerFnError> {
    let mut files = MOCK_FILES
        .lock()
        .map_err(|_| ServerFnError::new("FILES mutex poisoned"))?;

    let new_id = files.iter().map(|f| f.file_id).max().unwrap_or(0) + 1;

    files.push(GroupFile {
        group_id,
        file_id: new_id,
        filename,
        uploaded_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
    });

    Ok(())
}

#[server]
pub async fn delete_file_mock(group_id: String, file_id: i32) -> Result<(), ServerFnError> {
    let mut files = MOCK_FILES
        .lock()
        .map_err(|_| ServerFnError::new("FILES mutex poisoned"))?;

    files.retain(|f| !(f.group_id == group_id && f.file_id == file_id));
    Ok(())
}
