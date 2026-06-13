pub mod db;
pub mod handler;
pub mod monitor;
pub mod parser;
pub mod server;
pub mod state;
pub mod supervisor;
pub mod artifact_server;

use db::Database;
use std::sync::{Arc, Mutex};

pub type SharedDatabase = Arc<Mutex<Database>>;
