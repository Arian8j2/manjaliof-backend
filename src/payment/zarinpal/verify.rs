use super::{code::ZarinpalCode, ZARINPAL_MERCHANT_ID};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ZarinpalVerifyPayment {
    merchant_id: String,
    amount: u32,
    authority: String,
}

impl ZarinpalVerifyPayment {
    pub fn from(authority: String, amount: u32) -> Self {
        ZarinpalVerifyPayment {
            merchant_id: ZARINPAL_MERCHANT_ID.clone(),
            amount,
            authority,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ZarinpalVerifyPaymentResult {
    pub data: ZarinpalVerifyPaymentResultData,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ZarinpalVerifyPaymentResultData {
    pub code: ZarinpalCode,
    pub message: String,
    pub card_hash: String,
    pub card_pan: String,
    pub ref_id: u64,
    pub fee_type: String,
    pub fee: u32,
}
