pub type GroupId = i32;
pub type UserId = i64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroupTab {
    Overview,
    Members,
    Files,
    Roles,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroupRole {
    Owner,
    Admin,
    Member,
}

#[derive(Clone, Debug)]
pub struct GroupsState {
    pub selected_group: Option<GroupId>,
    pub active_tab: GroupTab,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl Default for GroupsState {
    fn default() -> Self {
        Self {
            selected_group: None,
            active_tab: GroupTab::Overview,
            is_loading: false,
            error: None,
        }
    }
}
