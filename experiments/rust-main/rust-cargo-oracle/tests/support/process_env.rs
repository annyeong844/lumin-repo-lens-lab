#![allow(dead_code, unused_imports)]

#[path = "process_env/compilation/mod.rs"]
mod compilation;
#[path = "process_env/lock.rs"]
mod lock;
#[path = "process_env/scoped_var.rs"]
mod scoped_var;

#[allow(unused_imports)]
pub use compilation::{with_clean_compilation_env, with_rustflags};
pub use lock::lock_process_env;
pub use scoped_var::with_env_var;
