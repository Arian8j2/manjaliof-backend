use serde::{Deserialize, Serialize};

macro_rules! s {
    ($str:expr) => {
        $str.to_string()
    };
}

#[derive(Serialize, Deserialize)]
pub struct ZarinpalCode(i32);

impl std::string::ToString for ZarinpalCode {
    fn to_string(&self) -> String {
        match self.0 {
            -9 => s!("خطای اعتبار سنجی"),
            -10 => s!("ای پی و يا مرچنت كد پذيرنده صحيح نيست"),
            -11 => s!("مرچنت کد فعال نیست لطفا با تیم پشتیبانی ما تماس بگیرید"),
            -12 => s!("تلاش بیش از حد در یک بازه زمانی کوتاه."),
            -15 => s!("ترمینال شما به حالت تعلیق در آمده با تیم پشتیبانی تماس بگیرید"),
            -16 => s!("سطح تاييد پذيرنده پايين تر از سطح نقره اي است."),
            100 => s!("عملیات موفق"),
            -30 => s!("اجازه دسترسی به تسویه اشتراکی شناور ندارید"),
            -31 => s!("حساب بانکی تسویه را به پنل اضافه کنید مقادیر وارد شده واسه تسهیم درست نیست"),
            -32 => s!("Wages is not valid, Total wages(floating) has been overload max amount."),
            -33 => s!("درصد های وارد شده درست نیست"),
            -34 => s!("مبلغ از کل تراکنش بیشتر است"),
            -35 => s!("تعداد افراد دریافت کننده تسهیم بیش از حد مجاز است"),
            -40 => s!("Invalid extra params, expire_in is not valid."),
            -50 => s!("مبلغ پرداخت شده با مقدار مبلغ در وریفای متفاوت است"),
            -51 => s!("پرداخت ناموفق"),
            -52 => s!("خطای غیر منتظره با پشتیبانی تماس بگیرید"),
            -53 => s!("اتوریتی برای این مرچنت کد نیست"),
            -54 => s!("اتوریتی نامعتبر است"),
            101 => s!("تراکنش وریفای شده"),
            _ => s!("Unknown error"),
        }
    }
}

impl ZarinpalCode {
    pub fn is_success(&self) -> bool {
        self.0 == 100
    }
}
