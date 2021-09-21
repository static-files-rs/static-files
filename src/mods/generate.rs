use std::marker::PhantomData;

use super::{Adapter, Convert, ResourcePrototype, ResourceStorage, Result};

pub struct Generate<I, S: ResourceStorage> {
    pub(super) iter: I,
    pub(super) resource_storage: S,
}

impl<I, S> Generate<I, S>
where
    I: Iterator<Item = Result<ResourcePrototype>>,
    S: ResourceStorage,
{
    pub fn module_generator<G>(self, module_options: G) -> G::Implementation
    where
        G: ModuleGeneratorOptions<I, S>,
    {
        module_options.build(self)
    }
}

pub trait ModuleGeneratorOptions<I, S>
where
    S: ResourceStorage,
{
    type Implementation: ModuleGenerator<I, S>;

    fn build(self, generate: Generate<I, S>) -> Self::Implementation;
}

pub trait ModuleGenerator<I, S> {}

pub struct ModuleGenerators;

impl ModuleGenerators {
    pub fn split_by_count(count: usize) -> SplitByCountModuleGeneratorOptions {
        SplitByCountModuleGeneratorOptions { count }
    }
}

pub struct SplitByCountModuleGeneratorOptions {
    count: usize,
}

impl<I, S> ModuleGeneratorOptions<I, S> for SplitByCountModuleGeneratorOptions
where
    S: ResourceStorage,
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
    S: ResourceStorage,
{
    options: SplitByCountModuleGeneratorOptions,
    generate: Generate<I, S>,
}

impl<I, S> ModuleGenerator<I, S> for SplitByCountModuleGenerator<I, S> where S: ResourceStorage {}

impl<I, S> Iterator for SplitByCountModuleGenerator<I, S>
where
    S: ResourceStorage,
{
    type Item = Result<ModulePrototype>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct ModulePrototype;

impl WriteTo for ModulePrototype {}

pub struct FunctionOptions {
    name: String,
}

impl<'a> From<&'a str> for FunctionOptions {
    fn from(value: &'a str) -> Self {
        Self {
            name: value.into(),
            ..Default::default()
        }
    }
}

pub trait WriteTo {}

impl Default for FunctionOptions {
    fn default() -> Self {
        Self {
            name: "generate".to_string(),
        }
    }
}

pub struct MapOptions {}

pub trait ToFunction {
    fn to_function(self, options: impl Into<FunctionOptions>) -> Result<()>;
}

impl<I, T> ToFunction for T
where
    T: Iterator<Item = Result<I>>,
    I: WriteTo,
{
    fn to_function(self, options: impl Into<FunctionOptions>) -> Result<()> {
        todo!()
    }
}
