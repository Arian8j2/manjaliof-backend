mod request;
mod verify;
mod code;

use super::Payment;
use request::{ZarinpalRequestPayment, ZarinpalRequestPaymentResult};
use verify::{ZarinpalVerifyPayment, ZarinpalVerifyPaymentResult};
use async_trait::async_trait;
use reqwest::Client;
use std::env;

const ZARINPAL_API_URL: &str = "https://api.zarinpal.com/pg/v4/payment";

lazy_static! {
    pub static ref ZARINPAL_MERCHANT_ID: String = 
        env::var("ZARINPAL_MERCHANT_ID").expect("environment variable 'ZARINPAL_MERCHANT_ID' is not set");
}

pub struct Zarinpal { }

impl Zarinpal {
    pub fn new() -> Self {
        Zarinpal { }
    }
}

#[async_trait]
impl Payment for Zarinpal {
    async fn request_payment_authority(&self, description: &str, amount: u32) -> Result<String, String> {
        let description = description.replace("ip", "server");
        let resp = Client::new()
            .post(format!("{ZARINPAL_API_URL}/request.json"))
            .json(&ZarinpalRequestPayment::from(amount, description))
            .send().await.map_err(|e| format!("send failed: {e}"))?
            .text().await.map_err(|e| format!("receiving failed: {e}"))?;

        let result: ZarinpalRequestPaymentResult = serde_json::from_str(&resp)
            .map_err(|e| format!("desrializing '{resp}' failed: {e}"))?;

        let code = result.data.code;
        if code.is_success() {
            Ok(result.data.authority)
        } else {
            Err(code.to_string())
        }
    }

    #[allow(unused_variables)]
    async fn verify(&self, authority: &str, amount: u32) -> Result<(), String> {
        let resp = Client::new()
            .post(format!("{ZARINPAL_API_URL}/verify.json"))
            .json(&ZarinpalVerifyPayment::from(authority.to_string(), amount))
            .send().await.map_err(|e| format!("send failed: {e}"))?
            .text().await.map_err(|e| format!("receiving failed: {e}"))?;
            
        let result: ZarinpalVerifyPaymentResult = serde_json::from_str(&resp)
            .map_err(|e| format!("desrializing '{resp}' failed: {e}"))?;

        let code = result.data.code;
        if code.is_success() {
            Ok(())
        } else {
            Err(code.to_string())
        }
    }
}
