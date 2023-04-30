use mysql_async::{prelude::Queryable, Pool};
use tracing::info;

use crate::secrets::get_secret;

pub mod delete;
pub mod insert;
pub mod select;
pub mod statements;
pub mod structs;

//get the sql db link
pub async fn get_db_link() -> String {
    let db = get_secret("DB").value;
    let user = get_secret("MYSQL_USER").value;
    let pass = get_secret("MYSQL_PASS").value;
    let ip = get_secret("MYSQL_IP").value;
    return format!("mysql://{}:{}@{}/{}", user, pass, ip, db);
}

//initialize
pub async fn init_sql() {
    let url = get_db_link().await;

    let pool = Pool::new(url.as_str());

    let mut conn = pool.get_conn().await.unwrap();

    info!("creating table if not exists");

    //execute the query to create the table
    match conn.query_drop(statements::CREATE_ROULETTE_TABLE).await {
        Ok(_) => info!("table created"),
        Err(e) => info!("table already exists: {}", e),
    };
}

//test connection to mysql
pub async fn test_connection() -> Result<(), mysql_async::Error> {
    info!("the db link is \"{}\"", get_db_link().await);
    let url = get_db_link().await;

    let pool = Pool::new(url.as_str());

    match pool.get_conn().await {
        Ok(v) => {
            return match v.disconnect().await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
        Err(e) => return Err(e),
    };
}
