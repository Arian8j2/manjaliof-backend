#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

mod payment;
mod runner;
mod db;
mod token;
#[cfg(test)] mod tests;

use rocket::{serde::{Deserialize, Serialize, json::Json}, Build, State};
use rocket_db_pools::Connection;
use payment::{Payment, zarinpal::Zarinpal};
use runner::{Runner, manjaliof::Manjaliof};
use db::{Db, db_add_transaction, db_find_name};
use token::Token;
use std::sync::Arc;

macro_rules! try_in_request {
    ($expr:expr) => {
        match $expr {
            Ok(smth) => smth,
            Err(error) => return Json(RequestResult { success: false, message: error })
        }
    };
}

type PaymentState = State<Arc<dyn Payment>>;
type RunnerState = State<Arc<dyn Runner>>;

#[rocket::main]
async fn main() -> Result<(), String> {
    let payment = Zarinpal::new();
    let runner = Manjaliof::new();
    let _rocket = rocket(payment, runner).launch().await.map_err(|e| e.to_string())?;
    Ok(())
}

fn rocket<T, R>(payment: T, runner: R) -> rocket::Rocket<Build>
where
    T: Payment,
    R: Runner
{
    let shared_payment: Arc<dyn Payment> = Arc::new(payment);
    let shared_runner: Arc<dyn Runner> = Arc::new(runner);
    let db = Db::new();
    rocket::build().attach(db).manage(shared_payment).manage(shared_runner).mount("/", routes![
       create_payment,
       verify_payment
    ])
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct RequestResult {
    success: bool,
    message: String
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct CreatePaymentArgs {
    clients: Vec<String>
}

#[post("/create_payment", data = "<args>")]
async fn create_payment(_token: Token, mut db: Connection<Db>, args: Json<CreatePaymentArgs>, payment: &PaymentState, runner: &RunnerState)
    -> Json<RequestResult>
{
    if args.clients.is_empty() {
        return Json(RequestResult { success: false, message: "at least provide one client".to_string() });
    }

    try_in_request!(
        runner.validate_clients(&args.clients).await
            .map_err(|e| format!("cannot validate clients: {e}"))
    );

    let price = args.clients.len() as u32 * 55 * 10000;
    let names = args.clients.join(",");
    let authority = try_in_request!(
        payment.request_payment_authority(&names, price).await
            .map_err(|e| format!("cannot request payment: {e}"))
    );

    try_in_request!(
        db_add_transaction(&mut db, &authority, &names, price).await
            .map_err(|e| format!("cannot add transactiont to database: {e}"))
    );
    Json(RequestResult { success: true, message: authority })
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct VerifyPaymentArgs {
    authority: String
}

#[post("/verify_payment", data = "<args>")]
async fn verify_payment(mut db: Connection<Db>, args: Json<VerifyPaymentArgs>, payment: &PaymentState, runner: &RunnerState)
    -> Json<RequestResult>
{
    let (names, amount) = try_in_request!(db_find_name(&mut db, &args.authority).await);
    try_in_request!(payment.verify(&args.authority, amount).await);

    for name in names.split(",") {
        try_in_request!(runner.make_client_paid(name).await);
    }

    Json(RequestResult { success: true, message: String::new() })
}
