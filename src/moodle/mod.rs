pub mod api;
pub mod assignments;
pub mod auth;
pub mod client;
pub mod courses;
pub mod html;
pub mod normalize;
pub mod urls;

pub use client::MoodleClient;

#[derive(Debug, thiserror::Error)]
pub enum MoodleError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl MoodleError {
    pub fn message(msg: impl Into<String>) -> Self {
        MoodleError::Message(msg.into())
    }
}
