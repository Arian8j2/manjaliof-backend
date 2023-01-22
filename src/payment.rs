use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Payment: Send + Sync + 'static {
    async fn request_payment_authority(&self, description: &str, amount: u32) -> Result<String, String>;
    async fn verify(&self, authority: &str, amount: u32) -> Result<(), String>;
}

pub mod zarinpal;
