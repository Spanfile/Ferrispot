use futures::lock::Mutex;
use reqwest::Client as AsyncClient;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ImplicitGrantUserClient {
    inner: Arc<ImplicitGrantUserClientRef>,
    http_client: AsyncClient,
}

#[derive(Debug)]
struct ImplicitGrantUserClientRef {
    access_token: Mutex<String>,
}
