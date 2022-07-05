use super::private::ClientBase;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait UnscopedClient: ClientBase {
    async fn track(&self, track_id: &str) -> Result<()>;
}
