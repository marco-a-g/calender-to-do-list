#[derive(Clone, Debug, PartialEq)]
pub enum AuthStatus {
    Unauthenticated,
    Authenticated { user_id: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthView {
    Login,
    Register,
}

pub fn login_mock(username: &str, password: &str) -> Result<AuthStatus, &'static str> {
    if username == "admin" && password == "admin" {
        Ok(AuthStatus::Authenticated {
            user_id: "admin".to_string(),
        })
    } else {
        Err("Invalid credentials")
    }
}
