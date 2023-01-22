use rocket::{fairing::{self, AdHoc}, Rocket, Build};
use rocket_db_pools::{sqlx::{self, Executor, Row}, Database, Connection};
use chrono::Utc;

const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

macro_rules! try_sql {
    ($expr:expr) => {
        match $expr {
            Ok(smth) => smth,
            Err(error) => return Err(format!("sql error: {error}"))
        } 
    };
}

#[derive(Database)]
#[database("sqlitedb")]
pub struct Db(sqlx::SqlitePool);

impl Db {
    pub fn new() -> AdHoc {
        AdHoc::on_ignite("transaction setup", |rocket| async {
            rocket.attach(Self::init()).attach(AdHoc::try_on_ignite("setup tables", Self::setup_tables))
        })
    }

    async fn setup_tables(rocket: Rocket<Build>) -> fairing::Result {
        match Db::fetch(&rocket) {
            Some(db) => {
                let res = db.execute(
                    "CREATE TABLE IF NOT EXISTS transactions (
                        authority TEXT PRIMARY KEY,
                        name TEXT NOT NULL,
                        amount UNSIGNED INTEGER NOT NULL,
                        date TEXT NOT NULL
                    )"
                ).await;

                if let Err(error) = res {
                    error!("cannot create tables: sql error: {error}");
                    return Err(rocket);
                }
                Ok(rocket)
            }
            None => Err(rocket)
        }
    }
}

pub async fn db_add_transaction(db: &mut Connection<Db>, authority: &str, name: &str, amount: u32) -> Result<(), String> {
    let now_date = Utc::now().format(DATETIME_FORMAT).to_string();
    let query = sqlx::query("INSERT INTO transactions (authority, name, amount, date) VALUES (?, ?, ?, ?)")
                    .bind(authority).bind(name).bind(amount).bind(now_date);
    try_sql!(db.execute(query).await);
    Ok(())
}

pub async fn db_find_name(db: &mut Connection<Db>, authority: &str) -> Result<(String, u32), String> {
    let query = sqlx::query("SELECT name, amount FROM transactions WHERE authority=? LIMIT 1").bind(authority);
    let rows = try_sql!(db.fetch_all(query).await);

    if rows.len() == 0 {
        return Err("authority not exists".to_string())
    }

    let row = rows.get(0).unwrap();
    let name: String = row.get(0);
    let amount: u32 = row.get(1);
    Ok((name, amount))
}
