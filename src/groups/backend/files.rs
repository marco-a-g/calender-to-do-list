use dioxus::prelude::*;
use std::sync::{LazyLock, Mutex};

pub type FileTransfer = (i32, i32, String, String); // (group_id, file_id, filename, uploaded_at)

#[derive(Clone, Debug)]
pub struct GroupFile {
    pub group_id: i32,
    pub file_id: i32,
    pub filename: String,
    pub uploaded_at: String,
}

pub(crate) static MOCK_FILES: LazyLock<Mutex<Vec<GroupFile>>> = LazyLock::new(|| {
    Mutex::new(vec![
        GroupFile {
            group_id: 11,
            file_id: 1,
            filename: "Sprint-Plan.pdf".into(),
            uploaded_at: "07.01.2026".into(),
        },
        GroupFile {
            group_id: 11,
            file_id: 2,
            filename: "Designs.zip".into(),
            uploaded_at: "06.01.2026".into(),
        },
    ])
});

fn to_transfer(f: &GroupFile) -> FileTransfer {
    (
        f.group_id,
        f.file_id,
        f.filename.clone(),
        f.uploaded_at.clone(),
    )
}

#[server]
pub async fn fetch_files(group_id: i32) -> Result<Vec<FileTransfer>, ServerFnError> {
    let files = MOCK_FILES.lock().unwrap();
    Ok(files
        .iter()
        .filter(|f| f.group_id == group_id)
        .map(to_transfer)
        .collect())
}

#[server]
pub async fn upload_file_mock(group_id: i32, filename: String) -> Result<(), ServerFnError> {
    let mut files = MOCK_FILES.lock().unwrap();
    let new_id = files.iter().map(|f| f.file_id).max().unwrap_or(0) + 1;

    files.push(GroupFile {
        group_id,
        file_id: new_id,
        filename,
        uploaded_at: chrono::Local::now().format("%d.%m.%Y").to_string(),
    });

    Ok(())
}

#[server]
pub async fn delete_file_mock(group_id: i32, file_id: i32) -> Result<(), ServerFnError> {
    let mut files = MOCK_FILES.lock().unwrap();
    files.retain(|f| !(f.group_id == group_id && f.file_id == file_id));
    Ok(())
}
