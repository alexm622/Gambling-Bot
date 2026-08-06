use mysql_async::{prelude::Queryable, Pool};
use std::sync::OnceLock;
use tracing::info;

use crate::secrets::get_secret;

pub mod delete;
pub mod insert;
pub mod select;
pub mod statements;
pub mod structs;
pub mod transactions;

static POOL: OnceLock<Pool> = OnceLock::new();

//get the sql db link
pub fn get_db_link() -> String {
    let db = get_secret("DB").value;
    let user = get_secret("MYSQL_USER").value;
    let pass = get_secret("MYSQL_PASS").value;
    let ip = get_secret("MYSQL_IP").value;
    format!("mysql://{}:{}@{}/{}", user, pass, ip, db)
}

pub fn get_pool() -> Pool {
    POOL.get().expect("SQL pool not initialized").clone()
}

fn build_pool() -> Pool {
    Pool::new(get_db_link().as_str())
}

//initialize
pub async fn init_sql() {
    let pool = build_pool();
    let _ = POOL.set(pool);

    let mut conn = get_pool().get_conn().await.unwrap();

    info!("creating tables if not exist");

    //execute the query to create the table
    match conn.query_drop(statements::CREATE_ROULETTE_TABLE).await {
        Ok(_) => info!("roulette_bets table created"),
        Err(e) => info!("roulette_bets table already exists: {}", e),
    };

    match conn.query_drop(statements::CREATE_TRANSACTIONS_TABLE).await {
        Ok(_) => info!("transactions table created"),
        Err(e) => info!("transactions table already exists: {}", e),
    };

    match conn.query_drop(statements::CREATE_USER_STATS_TABLE).await {
        Ok(_) => info!("user_stats table created"),
        Err(e) => info!("user_stats table already exists: {}", e),
    };
}

//test connection to mysql
pub async fn test_connection() -> Result<(), mysql_async::Error> {
    info!("the db link is \"{}\"", get_db_link());
    let pool = build_pool();

    match pool.get_conn().await {
        Ok(v) => match v.disconnect().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}
