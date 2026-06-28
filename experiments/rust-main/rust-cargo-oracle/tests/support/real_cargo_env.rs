#![allow(dead_code)]

#[path = "env/mod.rs"]
mod cargo_env;
#[path = "fixtures/mod.rs"]
mod fixtures;
#[path = "process_env.rs"]
pub mod process_env;

pub use cargo_env::RealCargoEnv;
