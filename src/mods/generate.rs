use std::{
    fs::File,
    io::Write,
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
pub struct ModuleGeneratorOptions {}

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

impl<I, S> Iterator for SplitByCountModuleGenerator<I, S>
where
    S: ResourceStorageType,
{
    type Item = Result<ModulePrototype>;

    fn next(&mut self) -> Option<Self::Item> {
        None
        // todo!()
    }
}

impl<I, S> ResourceStorageType for SplitByCountModuleGenerator<I, S>
where
    S: ResourceStorageType,
{
    fn namespace(&self) -> &'static str {
        self.generate.resource_storage.namespace()
    }
}

struct ModuleOptions {
    root_module: String,
}

pub struct ModulePrototype {
    name: String,
    options: Arc<ModuleOptions>,
}

impl WriteTo for ModulePrototype {
    fn write(&self, output: impl Write, resource_storage: &impl ResourceStorageType) -> Result<()> {
        todo!()
    }
}

impl WriteToFunction for ModulePrototype {
    fn write_header(&self, mut output: impl Write, options: &FunctionOptions) -> Result<()> {
        Ok(writeln!(
            output,
            "mod {};\n{} use {}::{};",
            self.options.root_module,
            options.modifiers(),
            self.options.root_module,
            options.name()
        )?)
    }
}

#[derive(Default)]
pub struct FunctionOptions {
    name: Option<String>,
    path: Option<PathBuf>,
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
                    .map(|x| PathBuf::from(x).join("generated.rs"))
                    .map_err(|_e| ResourceError::WrongOutDir)
            },
            Ok,
        )
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

impl<N: AsRef<str>> From<N> for FunctionOptions {
    fn from(value: N) -> Self {
        Self {
            name: Some(value.as_ref().to_string()),
            ..Default::default()
        }
    }
}

pub trait WriteTo {
    fn write(&self, output: impl Write, resource_storage: &impl ResourceStorageType) -> Result<()>;
}

pub trait WriteToFunction: WriteTo {
    fn write_header(&self, output: impl Write, options: &FunctionOptions) -> Result<()>;
}

pub struct MapOptions {}

pub trait ToFunction {
    fn to_function(self, options: impl Into<FunctionOptions>) -> Result<()>;
}

impl<I, T> ToFunction for T
where
    T: Iterator<Item = Result<I>> + ResourceStorageType,
    I: WriteToFunction,
{
    fn to_function(mut self, options: impl Into<FunctionOptions>) -> Result<()> {
        let options = options.into();

        let generated_filename = options.path()?;
        let mut generated_file = File::create(&generated_filename)?;

        if let Some(first) = self.next().transpose()? {
            first.write_header(&mut generated_file, &options)?;
            first.write(&mut generated_file, &self)?;
            while let Some(writable) = self.next().transpose()? {
                writable.write(&mut generated_file, &self)?;
            }
        } else {
            write!(
                &mut generated_file,
                "{}\n{} fn {}() -> impl ::{}::ResourceStorage {{\n\"Empty set\"\n}}\n",
                options.annotations().join("\n"),
                options.modifiers(),
                options.name(),
                self.namespace(),
            )?;
        }

        Ok(())
    }
}
