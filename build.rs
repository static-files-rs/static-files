#![allow(dead_code)]
#![doc(html_no_source)]
#[path ="src/mods/mod.rs"]
mod mods;

use std::{env, path::Path};

use mods::{
    resource::generate_resources_mapping,
    resource_dir::resource_dir,
    sets::{generate_resources_sets, SplitByCount},
};

fn main() -> std::io::Result<()> {
    resource_dir("./tests").build()?;

    let out_dir = env::var("OUT_DIR").unwrap();

    generate_resources_mapping(
        "./tests",
        None,
        Path::new(&out_dir).join("generated_mapping.rs"),
    )?;

    generate_resources_sets(
        "./tests",
        None,
        Path::new(&out_dir).join("generated_sets.rs"),
        "sets",
        "generate",
        &mut SplitByCount::new(2),
    )?;

    Ok(())
}
