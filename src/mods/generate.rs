use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use super::{ResourceError, ResourcePrototype, ResourceStorageType, Result};

pub struct Generate<I, S: ResourceStorageType> {
    pub(super) iter: I,
    pub(super) resource_storage: S,
}

impl<I, S> Generate<I, S>
where
    I: Iterator<Item = Result<ResourcePrototype>>,
    S: ResourceStorageType,
{
    pub fn module_generator<G>(self, module_options: G) -> G::Implementation
    where
        G: ModuleGeneratorBuilder<I, S>,
    {
        module_options.build(self)
    }
}

pub trait ModuleGeneratorBuilder<I, S>
where
    S: ResourceStorageType,
{
    type Implementation: ModuleGenerator<I, S>;

    fn build(self, generate: Generate<I, S>) -> Self::Implementation;
}

pub trait ModuleGenerator<I, S> {}

pub struct ModuleGenerators;

impl ModuleGenerators {
    pub fn split_by_count(count: usize) -> SplitByCountModuleGeneratorOptions {
        SplitByCountModuleGeneratorOptions {
            count,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct ModuleGeneratorOptions {
    name: Option<String>,
}

impl ModuleGeneratorOptions {
    fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("sets")
    }

    fn name_for_function(&self, fn_options: &FunctionOptions) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("{}_sets", fn_options.name()))
    }
}

#[derive(Default)]
pub struct SplitByCountModuleGeneratorOptions {
    count: usize,
    generic_options: ModuleGeneratorOptions,
}

impl<I, S> ModuleGeneratorBuilder<I, S> for SplitByCountModuleGeneratorOptions
where
    S: ResourceStorageType,
{
    type Implementation = SplitByCountModuleGenerator<I, S>;

    fn build(self, generate: Generate<I, S>) -> Self::Implementation {
        Self::Implementation {
            options: self,
            generate,
        }
    }
}

pub struct SplitByCountModuleGenerator<I, S>
where
    S: ResourceStorageType,
{
    options: SplitByCountModuleGeneratorOptions,
    generate: Generate<I, S>,
}

impl<I, S> ModuleGenerator<I, S> for SplitByCountModuleGenerator<I, S> where S: ResourceStorageType {}

impl<I, S> SplitByCountModuleGenerator<I, S>
where
    S: ResourceStorageType,
{
    pub(crate) fn create_set_module_file(
        resource_storage: &S,
        module_dir: &PathBuf,
        modules_count: i32,
    ) -> Result<File> {
        todo!()
    }
}

impl<I, S> ToFunction for SplitByCountModuleGenerator<I, S>
where
    S: ResourceStorageType,
    I: Iterator<Item = Result<ResourcePrototype>>,
{
    fn write_function(self, options: impl Into<FunctionOptions>) -> Result<()> {
        let Generate {
            iter,
            resource_storage,
        } = self.generate;
        let function_options = options.into();
        let namespace = resource_storage.namespace();

        let module_name = self.options.generic_options.name_for_function(&function_options);

        let module_dir = function_options.path()?.join(&module_name);

        fs::create_dir_all(&module_dir)?;

        let mut count = 0usize;
        {
            let mut module_file = File::create(module_dir.join("mod.rs"))?;

            writeln!(
                module_file,
                "use ::{0}::new_resource as n;\n\
                use ::std::include_bytes as i;\n\
                use ::{0}::Resource;\n\
                use {1};",
                namespace,
                resource_storage.storage_type(),
            )?;

            let mut modules_count = 1;

            let mut set_file =
                Self::create_set_module_file(&resource_storage, &module_dir, modules_count)?;

            for resource in iter {
                let resource = resource?;
                count += 1;
                if count % self.options.count == 0 {

                }
            }
        }

        {
            let generated_filename = function_options.generated_filename()?;
            dbg!(&generated_filename);
            let mut generated_file = File::create(&generated_filename)?;

            if count > 0 {
                write!(
                    generated_file,
                    "mod {};\n\
                    pub use {}::{};",
                    module_name,
                    module_name,
                    function_options.name()
                )?;
            } else {
                write!(
                    generated_file,
                    "{}\n{} fn {}() -> impl ::{}::ResourceStorage<{}> {{\n\"Empty set\"\n}}\n",
                    function_options.annotations().join("\n"),
                    function_options.modifiers(),
                    function_options.name(),
                    namespace,
                    resource_storage.tag_type(),
                )?;
            }
        }

        Ok(())
    }
}

struct ModuleOptions {
    root_module: String,
}

#[derive(Default)]
pub struct FunctionOptions {
    name: Option<String>,
    path: Option<PathBuf>,
    filename: Option<String>,
    annotations: Option<Vec<String>>,
    modifiers: Option<String>,
}

impl FunctionOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_name<S: ToString>(mut self, name: impl Into<Option<S>>) -> Self {
        self.name = name.into().as_ref().map(S::to_string);
        self
    }

    pub fn with_path<P: AsRef<Path>>(mut self, path: impl Into<Option<P>>) -> Self {
        self.path = path.into().as_ref().map(AsRef::as_ref).map(PathBuf::from);
        self
    }

    pub fn with_annotations<S: ToString>(mut self, annotations: impl Into<Option<Vec<S>>>) -> Self {
        self.annotations = annotations
            .into()
            .map(|x| x.iter().map(S::to_string).collect());
        self
    }

    pub fn with_modifiers<S: ToString>(mut self, modifiers: impl Into<Option<S>>) -> Self {
        self.modifiers = modifiers.into().as_ref().map(S::to_string);
        self
    }

    fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("generate")
    }

    fn path(&self) -> Result<PathBuf> {
        self.path.clone().map_or_else(
            || {
                std::env::var("OUT_DIR")
                    .map(PathBuf::from)
                    .map_err(|_e| ResourceError::WrongOutDir)
            },
            Ok,
        )
    }

    fn filename(&self) -> &str {
        self.filename.as_deref().unwrap_or("generated.rs")
    }

    fn generated_filename(&self) -> Result<PathBuf> {
        self.path().map(|path| path.join(self.filename()))
    }

    fn annotations(&self) -> Vec<&str> {
        self.annotations
            .as_ref()
            .map(|x| x.iter().map(String::as_str).collect())
            .unwrap_or_else(|| vec!["#[allow(clippy::unreadable_literal)]"])
    }

    fn modifiers(&self) -> &str {
        self.modifiers.as_deref().unwrap_or("pub")
    }
}

pub trait ToFunction {
    fn write_function(self, options: impl Into<FunctionOptions>) -> Result<()>;
}

pub trait ToMap {
    fn write_map(self) -> Result<()>;
}
