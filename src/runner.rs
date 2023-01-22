use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Runner: Send + Sync + 'static {
    async fn validate_clients(&self, names: &Vec<String>, reffer: &str) -> Result<(), String>;
    async fn make_client_paid(&self, name: &str) -> Result<(), String>;
}

pub mod manjaliof;
