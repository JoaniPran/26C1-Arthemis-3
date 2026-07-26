pub mod artifact_server;
pub mod db;
pub mod event_server;
pub mod handler;
pub mod monitor;
pub mod parser;
pub mod server;
pub mod state;
pub mod supervisor;

use db::Database;
use std::sync::{Arc, Mutex};

pub type SharedDatabase = Arc<Mutex<Database>>;
