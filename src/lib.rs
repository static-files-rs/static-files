#![doc(test(no_crate_inject))]
#![doc = include_str!("../README.md")]

mod mods;

pub use crate::mods::{
    npm_build::{npm_resource_dir, NodeModulesStrategy, NpmBuild},
    resource::{self, Resource},
    resource_dir::{resource_dir, ResourceDir},
    sets,
};
