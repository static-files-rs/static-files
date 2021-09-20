use static_files::{
    ModuleGenerators, NoopCompressConverter, ResourceFileAdapters, ResourceFiles, ResourceStorages,
    ToFunction,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ResourceFiles::new("./src")?
        .compress(NoopCompressConverter)
        .compress(NoopCompressConverter)
        .for_each(|x| {
            dbg!(&x);
        });
    ResourceFiles::new("./src")?
        .map(|x| {
            dbg!(&x);
            x
        })
        .compress(NoopCompressConverter)
        .generate(ResourceStorages::hash_map())
        .module_generator(ModuleGenerators::split_by_count(10))
        .to_function("generate")
        .map_err(|e| e.into())
}
