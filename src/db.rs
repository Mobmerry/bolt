use std::env;
use dotenv::dotenv;

use mongodb::{Client, ThreadedClient};
use mongodb::db::Database;

pub fn get_connection() -> Database {
  dotenv().ok();

  let db_host = env::var("MONGO_HOST").unwrap();
  let db_port = 27017; //env::var("MONGO_PORT").unwrap();
  let db_name = &* env::var("DB_NAME").unwrap();

  println!("Connecting to DB - {:?}:{:?}", db_host, db_port);

  let client = Client::connect(&*db_host, db_port)
                      .ok()
                      .expect("Failed to initialize client.");

  return client.db(&db_name);
}
