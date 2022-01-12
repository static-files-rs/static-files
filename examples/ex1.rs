use std::path::Path;

use static_files::{
    FunctionOptions, ModuleGenerators, NoopCompressConverter, ResourceError, ResourceFileAdapters,
    ResourceFiles, ResourceStorages, ToFunction,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("OUT_DIR", "./target/ex1");

    match generate_from("./src") {
        Ok(()) => (),
        Err(err) => eprintln!("Error: {}", err),
    }

    eprint!("{}", std::fs::read_to_string("./target/ex1/generated.rs")?);

    Ok(())
}

fn generate_from<P: AsRef<Path>>(path: P) -> Result<(), ResourceError> {
    ResourceFiles::new(&path)?
        .compress(NoopCompressConverter)
        .compress(NoopCompressConverter)
        .for_each(|x| {
            // dbg!(&x);
        });
    ResourceFiles::new(&path)?
        .map(|x| {
            //     dbg!(&x);
            x
        })
        .compress(NoopCompressConverter)
        .generate(ResourceStorages::hash_map())
        .module_generator(ModuleGenerators::split_by_count(10))
        // .to_function("generate")
        .write_function(FunctionOptions::new().with_name("generate"))?;

    Ok(())
}
