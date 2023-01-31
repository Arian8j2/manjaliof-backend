use super::{code::ZarinpalCode, ZARINPAL_MERCHANT_ID};
use serde::{Deserialize, Serialize};

const CALLBACK_URL: &str = "https://manjaliof.ts22.ir/verify";

#[derive(Serialize, Deserialize)]
pub struct ZarinpalRequestPayment {
    merchant_id: String,
    amount: u32,
    callback_url: &'static str,
    description: String,
}

impl ZarinpalRequestPayment {
    pub fn from(amount: u32, description: String) -> Self {
        ZarinpalRequestPayment {
            merchant_id: ZARINPAL_MERCHANT_ID.clone(),
            amount,
            callback_url: CALLBACK_URL,
            description,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ZarinpalRequestPaymentResult {
    pub data: ZarinpalRequestPaymentResultData,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ZarinpalRequestPaymentResultData {
    pub code: ZarinpalCode,
    pub message: String,
    pub authority: String,
    pub fee_type: String,
    pub fee: u32,
}
