pub use argon_core::auth::auth_middleware;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct BasicUser {
    id: i32,
    username: String,
    password: String,
}

impl argon_core::auth::AuthenticatableUser for BasicUser {
    type Username = String;
    type Password = String;
    type Id = i32;

    fn get_id(&self) -> Self::Id {
        self.id
    }

    fn get_username(&self) -> Self::Username {
        self.username.clone()
    }

    fn get_password(&self) -> Self::Password {
        self.password.clone()
    }
}

pub struct BasicAuthenticator {
    db: DatabaseConnection,
}

impl argon_core::auth::Authenticator<BasicUser> for BasicAuthenticator {
    type Token = anyhow::Result<String>;

    async fn verify(&self, token: &str) -> Result<BasicUser, axum::http::StatusCode> {
        unimplemented!()
    }

    async fn attempt(&self, username: String, password: String) -> anyhow::Result<BasicUser> {
        unimplemented!()
    }

    async fn generate_token(&self, user: BasicUser) -> Self::Token {
        unimplemented!()
    }

    fn verify_header_name(&self) -> &'static str {
        "Auth"
    }
}

impl Clone for BasicAuthenticator {
    fn clone(&self) -> Self {
        Self { db: self.db.clone() }
    }
}
