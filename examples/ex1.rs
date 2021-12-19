use static_files::{
    FunctionOptions, ModuleGenerators, NoopCompressConverter, ResourceFileAdapters, ResourceFiles,
    ResourceStorages, ToFunction,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("OUT_DIR", "./target/ex1");
    ResourceFiles::new("./src")?
        .compress(NoopCompressConverter)
        .compress(NoopCompressConverter)
        .for_each(|x| {
            // dbg!(&x);
        });
    ResourceFiles::new("./src")?
        .map(|x| {
            dbg!(&x);
            x
        })
        .compress(NoopCompressConverter)
        .generate(ResourceStorages::hash_map())
        .module_generator(ModuleGenerators::split_by_count(10))
        // .to_function("generate")
        .write_function(FunctionOptions::new().with_name("generate"))?;
    eprint!("{}", std::fs::read_to_string("./target/ex1/generated.rs")?);
    Ok(())
}
