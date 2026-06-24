use std::fs;
use std::path::Path;

use anyhow::Result;

const LIB_RS: &str = "pub mod factory { pub struct Made; impl Made { pub fn normalize(&self) {} } pub fn make() -> Made { Made } }\nmacro_rules! custom_macro { () => {}; }\n#[cfg(feature = \"fast\")]\npub fn gated() {}\npub fn repeated_alpha() -> usize {\n    let answer = 42;\n    answer\n}\npub fn repeated_beta() -> usize {\n    let answer = 42;\n    answer\n}\npub fn demo() { let value = Some(1); let _ = value.unwrap(); let made = crate::factory::make(); let _ = made.normalize(); custom_macro!(); let _typed: u32 = \"wrong\"; }\n";

pub fn write_unified_cli_workspace(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nfast = []\n",
    )?;
    fs::write(root.join("src").join("lib.rs"), LIB_RS)?;

    fs::create_dir_all(root.join("tests"))?;
    fs::write(
        root.join("tests").join("integration.rs"),
        "pub fn helper() { let value = Some(1); let _ = value.unwrap(); assert!(true); }\n",
    )?;

    fs::create_dir_all(root.join("generated"))?;
    fs::write(
        root.join("generated").join("bindings.rs"),
        "pub fn binding() { let value = Some(1); let _ = value.unwrap(); }\n",
    )?;
    Ok(())
}
