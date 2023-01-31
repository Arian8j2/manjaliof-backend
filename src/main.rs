#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

mod cors;
mod db;
mod payment;
mod runner;
#[cfg(test)]
mod tests;
mod token;

use cors::Cors;
use db::{db_add_transaction, db_find_name, Db};
use payment::{zarinpal::Zarinpal, Payment};
use rocket::{
    serde::{json::Json, Deserialize, Serialize},
    Build, State,
};
use rocket_db_pools::Connection;
use runner::{manjaliof::Manjaliof, Runner};
use std::sync::Arc;
use token::Token;

macro_rules! try_in_request {
    ($expr:expr) => {
        match $expr {
            Ok(smth) => smth,
            Err(error) => {
                eprintln!("ERROR: {error}");
                return Json(RequestResult {
                    success: false,
                    message: error,
                });
            }
        }
    };
}

type PaymentState = State<Arc<dyn Payment>>;
type RunnerState = State<Arc<dyn Runner>>;

#[rocket::main]
async fn main() -> Result<(), String> {
    let payment = Zarinpal::new();
    let runner = Manjaliof::new();
    let _rocket = rocket(payment, runner)
        .launch()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn rocket<T, R>(payment: T, runner: R) -> rocket::Rocket<Build>
where
    T: Payment,
    R: Runner,
{
    let db = Db::new();
    let shared_payment: Arc<dyn Payment> = Arc::new(payment);
    let shared_runner: Arc<dyn Runner> = Arc::new(runner);
    rocket::build()
        .attach(db)
        .attach(Cors)
        .manage(shared_payment)
        .manage(shared_runner)
        .mount("/", routes![create_payment, verify_payment])
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct RequestResult {
    success: bool,
    message: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct CreatePaymentArgs {
    clients: Vec<String>,
}

#[post("/create_payment", data = "<args>")]
async fn create_payment(
    _token: Token,
    mut db: Connection<Db>,
    args: Json<CreatePaymentArgs>,
    payment: &PaymentState,
    runner: &RunnerState,
) -> Json<RequestResult> {
    try_in_request!((!args.clients.is_empty())
        .then_some(())
        .ok_or("at least provide one client".to_string()));

    try_in_request!(runner
        .validate_clients(&args.clients)
        .await
        .map_err(|e| format!("cannot validate clients: {e}")));

    let price = args.clients.len() as u32 * 55 * 10000;
    let price = price + calculate_tax(price);

    let names = args.clients.join(",");
    let authority = try_in_request!(payment
        .request_payment_authority(&names, price)
        .await
        .map_err(|e| format!("cannot request payment: {e}")));

    try_in_request!(db_add_transaction(&mut db, &authority, &names, price)
        .await
        .map_err(|e| format!("cannot add transactiont to database: {e}")));
    Json(RequestResult {
        success: true,
        message: authority,
    })
}

fn calculate_tax(price: u32) -> u32 {
    let tax = price as f32 / 100.0;
    let tax = tax + (tax / 100.0);
    tax.round() as u32
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct VerifyPaymentArgs {
    authority: String,
}

#[post("/verify_payment", data = "<args>")]
async fn verify_payment(
    mut db: Connection<Db>,
    args: Json<VerifyPaymentArgs>,
    payment: &PaymentState,
    runner: &RunnerState,
) -> Json<RequestResult> {
    let (names, amount) = try_in_request!(db_find_name(&mut db, &args.authority)
        .await
        .map_err(|e| format!("cannot find authority in db: {e}")));

    try_in_request!(payment
        .verify(&args.authority, amount)
        .await
        .map_err(|e| format!("cannot verify payment: {e}")));

    for name in names.split(",") {
        try_in_request!(runner
            .make_client_paid(name)
            .await
            .map_err(|e| format!("CRITICAL: runner failed on names '{names}': {e}")))
    }

    Json(RequestResult {
        success: true,
        message: String::new(),
    })
}
