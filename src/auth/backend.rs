//! Authentication functionality
use server_fn::ServerFnError;
use std::fmt;
use std::sync::OnceLock;
use supabase::Client;
use uuid::Uuid;

use crate::database::local::init_fetch::init_fetch_local_db::delete_db;

/// Holds the supabase client for remote connection
///
/// Since it stores the session, reuse it for all actions
static SUPABASE_CLIENT: OnceLock<Client> = OnceLock::new();
pub const SUPABASE_URL: &str = "https://wyqawnnkpusgtnhmeebn.supabase.co";
pub const ANON_KEY: &str = "sb_publishable_pFC_H--zfkiAc8ErS55x_Q__N0Yy4iJ";

/// Holds authentication status with user-id `Uuid` if authenticated
#[derive(Clone, Debug, PartialEq)]
pub enum AuthStatus {
    Unauthenticated,
    Authenticated { user_id: Uuid },
}

/// Different authentication views
#[derive(Clone, Debug, PartialEq)]
pub enum AuthView {
    Login,
    Register,
    CreateProfile,
}

/// Error that contains all relevant cases for authentication
///
/// Implements from for `supabase::Error` and `ServerFnError`
pub enum AuthError {
    ClientNotInitialized,
    ClientAlreadyInitialized,
    InvalidCredentials,
    NoUserReturned,
    UserAlreadyExists,
    Supabase(supabase::Error),
    Server(ServerFnError),
}

impl From<ServerFnError> for AuthError {
    fn from(error: ServerFnError) -> Self {
        AuthError::Server(error)
    }
}

impl From<supabase::Error> for AuthError {
    fn from(error: supabase::Error) -> Self {
        match &error {
            supabase::Error::Auth { message, .. } => {
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

/// Initialises the Supabase client
///
/// ### Errors
///
/// Throws `AuthError::ClientAlreadyInitialized`
pub fn init_client() -> Result<(), AuthError> {
    let client = Client::new(SUPABASE_URL, ANON_KEY)?;
    SUPABASE_CLIENT
        .set(client)
        .map_err(|_| AuthError::ClientAlreadyInitialized)?;

    Ok(())
}

/// Get the supabase client
///
/// Reuse for requests since it also stores the session
///
/// ### Returns
///
/// Result: Current supabase client `supabase::Client`
///
/// ### Errors
///
/// Throws ``AuthError::ClientNotInitialized``
pub fn get_client() -> Result<&'static Client, AuthError> {
    SUPABASE_CLIENT.get().ok_or(AuthError::ClientNotInitialized)
}

/// Log in to the application using `email` and `password`
///
/// ### Returns
///
/// Result: `AuthStatus { user id }`
///
/// ### Errors
///
/// Throws ``AuthError::ClientNotInitialized`` if client not initialized
///
/// Throws ``AuthError::InvalidCredentials`` if given invalid credentials
///
/// Throws ``AuthError::´Supabase(error)`` for other supabase errors
pub async fn login(email: &str, password: &str) -> Result<AuthStatus, AuthError> {
    let client = get_client()?;

    let response = client
        .auth()
        .sign_in_with_email_and_password(email, password)
        .await?;

    Ok(AuthStatus::Authenticated {
        user_id: response.user.ok_or(AuthError::NoUserReturned)?.id,
    })
}

/// Sign up to the application using `email` and `password`
///
/// ### Errors
///
/// Throws ``AuthError::ClientNotInitialized`` if client not initialized
///
/// Throws ``AuthError::UserAlreadyExists`` if user already exists
///
/// Throws ``AuthError::´Supabase(error)`` for other supabase errors
pub async fn signup(email: &str, password: &str) -> Result<(), AuthError> {
    let client = get_client()?;

    client
        .auth()
        .sign_up_with_email_and_password(email, password)
        .await?;

    Ok(())
}

/// Log out from the application
///
/// ### Errors
///
/// Throws ``AuthError::ClientNotInitialized`` if client not initialized
///
/// Throws ``AuthError::´Supabase(error)`` for other supabase errors
pub async fn logout() -> Result<(), AuthError> {
    let client = get_client()?;

    client.auth().sign_out().await?;

    // delete db so that noone can access data after logout
    // needs further checking if data is really deleted
    delete_db();

    Ok(())
}
