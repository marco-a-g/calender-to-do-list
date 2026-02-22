// TODO:
// - other auth components
// - tests
// - (optional) auth event listener with on_auth_state_change
// - maybe put api key into env var or config file
#![allow(dead_code, unused_imports)]
use http::Response;
use serde::de::value::Error as SerdeError;
use server_fn::ServerFnError;
use std::fmt;
use std::sync::OnceLock;
use supabase::{Auth, Client};
use uuid::Uuid;

static SUPABASE_CLIENT: OnceLock<Client> = OnceLock::new(); // Idea by AI
pub const SUPABASE_URL: &str = "https://wyqawnnkpusgtnhmeebn.supabase.co";
pub const ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Ind5cWF3bm5rcHVzZ3RuaG1lZWJuIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjU4NDM5MjksImV4cCI6MjA4MTQxOTkyOX0.0_m5aLSKNdqiqCNFWI8Hfa5iSOKrjf97qb9ZXxnboGA";

// gets called from main.rs
/// Initialises the Supabase client
///
/// Throws AuthError::ClientAlreadyInitialized
pub fn init_client() -> Result<(), AuthError> {
    let client = Client::new(SUPABASE_URL, ANON_KEY)?;
    SUPABASE_CLIENT
        .set(client)
        .map_err(|_| AuthError::ClientAlreadyInitialized)?;

    Ok(())
}

pub fn get_client() -> Result<&'static Client, AuthError> {
    SUPABASE_CLIENT.get().ok_or(AuthError::ClientNotInitialized)
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthStatus {
    Unauthenticated,
    Authenticated { user_id: Uuid }, // brauchts da die id??
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthView {
    Login,
    Register,
    CreateProfile,
}

pub enum AuthError {
    ClientNotInitialized,
    ClientAlreadyInitialized,
    InvalidCredentials,
    NoUserReturned,
    UserAlreadyExists,
    Supabase(supabase::Error),
    Server(ServerFnError),
}
// convert ServerFnError to AuthError until we have a clean error setup
impl From<ServerFnError> for AuthError {
    fn from(error: ServerFnError) -> Self {
        AuthError::Server(error)
    }
}

// Struktur von Implementierung durch AI erfragt
impl From<supabase::Error> for AuthError {
    fn from(error: supabase::Error) -> Self {
        match &error {
            supabase::Error::Auth { message, .. } => {
                // two periods ignores other fields
                if message.contains("invalid_credentials") {
                    AuthError::InvalidCredentials
                } else if message.contains("user_already_exists") {
                    AuthError::UserAlreadyExists
                } else {
                    AuthError::Supabase(error)
                }
            }
            _ => AuthError::Supabase(error),
        }
    }
}

impl fmt::Display for AuthError {
    // maybe rename to LoginError
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::ClientNotInitialized => write!(f, "Remote client not initialized"),
            AuthError::ClientAlreadyInitialized => write!(f, "Remote client already initialized"),
            AuthError::InvalidCredentials => write!(f, "Invalid Credentials"),
            AuthError::NoUserReturned => write!(f, "Auth returned no user"),
            AuthError::UserAlreadyExists => write!(f, "User already exists"),
            AuthError::Supabase(error) => write!(f, "{}", error),
            AuthError::Server(error) => write!(f, "{}", error),
        }
    }
}

/// Log in to the application
pub async fn login(username: &str, password: &str) -> Result<AuthStatus, AuthError> {
    let client = get_client()?;

    let response = client
        .auth()
        .sign_in_with_email_and_password(username, password)
        .await?;

    Ok(AuthStatus::Authenticated {
        user_id: response.user.ok_or(AuthError::NoUserReturned)?.id,
    })
}

pub async fn signup(email: &str, password: &str) -> Result<(), AuthError> {
    let client = get_client()?;

    let response = client
        .auth()
        .sign_up_with_email_and_password(email, password)
        .await?;
    println!("Response: {:?}", response);

    Ok(())
    // sign_up_with_email_password_and_data:
    // data = {
    //   "username": "fritz",
    //   "full_name": "Fritz Müller",
    //   "phone": "+4912345678"
    // }
    // trigger:
    // insert into profiles (id, username, full_name, phone)
    // values (
    //   new.id,
    //   new.raw_user_meta_data->>'username',
    //   new.raw_user_meta_data->>'full_name',
    //   new.raw_user_meta_data->>'phone'
    // );
    //
    // oder normalen signup und über requests
}

pub async fn logout() -> Result<(), AuthError> {
    let client = get_client()?;

    let response = client.auth().sign_out().await?;

    Ok(())
}
