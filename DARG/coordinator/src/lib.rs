pub mod db;
pub mod handler;
pub mod monitor;
pub mod parser;
pub mod state;
pub mod supervisor;

use db::Database;
use std::sync::{Arc, Mutex};

pub type SharedDatabase = Arc<Mutex<Database>>;
