use dioxus::prelude::*;
use std::sync::{LazyLock, Mutex};

// (id, name, color, member_count)
pub type GroupTransfer = (i32, String, String, i32);

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub color: String,
}

pub fn to_transfer(g: &Group, member_count: i32) -> GroupTransfer {
    (g.id, g.name.clone(), g.color.clone(), member_count)
}

static MOCK_GROUPS: LazyLock<Mutex<Vec<Group>>> = LazyLock::new(|| {
    Mutex::new(vec![
        Group {
            id: 10,
            name: "Marketing Team".into(),
            color: "#A855F7".into(),
        },
        Group {
            id: 11,
            name: "Dev Squad".into(),
            color: "#3A6BFF".into(),
        },
        Group {
            id: 12,
            name: "Design Crew".into(),
            color: "#EC4899".into(),
        },
        Group {
            id: 13,
            name: "Finance & Ops".into(),
            color: "#10B981".into(),
        },
        Group {
            id: 14,
            name: "HR & People".into(),
            color: "#F59E0B".into(),
        },
        Group {
            id: 15,
            name: "Customer Support".into(),
            color: "#06B6D4".into(),
        },
    ])
});

#[server]
pub async fn fetch_groups() -> Result<Vec<GroupTransfer>, ServerFnError> {
    let groups = MOCK_GROUPS.lock().unwrap();

    let list = groups.iter().map(|g| to_transfer(g, 0)).collect();

    Ok(list)
}

#[server]
pub async fn fetch_group_by_id(id: i32) -> Result<Option<(i32, String, String)>, ServerFnError> {
    let groups = MOCK_GROUPS.lock().unwrap();
    Ok(groups
        .iter()
        .find(|g| g.id == id)
        .map(|g| (g.id, g.name.clone(), g.color.clone())))
}

#[server]
pub async fn create_group(name: String, color: String) -> Result<(), ServerFnError> {
    let mut groups = MOCK_GROUPS.lock().unwrap();

    let next_id = groups.iter().map(|g| g.id).max().unwrap_or(0) + 1;

    groups.push(Group {
        id: next_id,
        name,
        color,
    });

    Ok(())
}

#[server]
pub async fn delete_group(id: i32) -> Result<(), ServerFnError> {
    let mut groups = MOCK_GROUPS.lock().unwrap();

    groups.retain(|g| g.id != id);

    Ok(())
}
