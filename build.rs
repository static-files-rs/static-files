#![allow(dead_code)]
#![doc(html_no_source)]
mod mods {
    include!("src/mods/mod.rs");
}

use std::{env, path::Path};

use mods::{generate_resources_mapping, generate_resources_sets, resource_dir, SplitByCount};

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
