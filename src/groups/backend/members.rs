use dioxus::prelude::*;
use std::sync::{LazyLock, Mutex};

pub type MemberTransfer = (i32, i32, String, String); // (group_id, user_id, name, role)

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub group_id: i32,
    pub user_id: i32,
    pub name: String,
    pub role: String,
    pub removed: bool,
}

pub(crate) static MOCK_MEMBERS: LazyLock<Mutex<Vec<Member>>> = LazyLock::new(|| {
    Mutex::new(vec![
        Member {
            group_id: 11,
            user_id: 1,
            name: "Lero".into(),
            role: "Owner".into(),
            removed: false,
        },
        Member {
            group_id: 11,
            user_id: 2,
            name: "Marco".into(),
            role: "Admin".into(),
            removed: false,
        },
        Member {
            group_id: 11,
            user_id: 3,
            name: "Anna".into(),
            role: "Member".into(),
            removed: false,
        },
        Member {
            group_id: 11,
            user_id: 4,
            name: "Ben".into(),
            role: "Member".into(),
            removed: false,
        },
        Member {
            group_id: 10,
            user_id: 5,
            name: "Chris".into(),
            role: "Owner".into(),
            removed: false,
        },
        Member {
            group_id: 13,
            user_id: 6,
            name: "Mia".into(),
            role: "Owner".into(),
            removed: false,
        },
    ])
});

fn to_transfer(m: &Member) -> MemberTransfer {
    (m.group_id, m.user_id, m.name.clone(), m.role.clone())
}

#[server]
pub async fn fetch_members(group_id: i32) -> Result<Vec<MemberTransfer>, ServerFnError> {
    let members = MOCK_MEMBERS.lock().unwrap();
    Ok(members
        .iter()
        .filter(|m| m.group_id == group_id && !m.removed)
        .map(to_transfer)
        .collect())
}

#[server]
pub async fn invite_member(group_id: i32, name: String, role: String) -> Result<(), ServerFnError> {
    let mut members = MOCK_MEMBERS.lock().unwrap();

    let new_user_id = members.iter().map(|m| m.user_id).max().unwrap_or(0) + 1;
    members.push(Member {
        group_id,
        user_id: new_user_id,
        name,
        role,
        removed: false,
    });

    Ok(())
}

#[server]
pub async fn remove_member(group_id: i32, user_id: i32) -> Result<(), ServerFnError> {
    let mut members = MOCK_MEMBERS.lock().unwrap();
    if let Some(m) = members
        .iter_mut()
        .find(|m| m.group_id == group_id && m.user_id == user_id)
    {
        m.removed = true;
    }
    Ok(())
}

#[server]
pub async fn update_member_role(
    group_id: i32,
    user_id: i32,
    role: String,
) -> Result<(), ServerFnError> {
    let mut members = MOCK_MEMBERS.lock().unwrap();
    if let Some(m) = members
        .iter_mut()
        .find(|m| m.group_id == group_id && m.user_id == user_id)
    {
        m.role = role;
    }
    Ok(())
}
