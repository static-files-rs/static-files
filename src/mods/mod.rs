mod npm_build;
mod resource;
pub mod sets;

pub use npm_build::{npm_resource_dir, NpmBuild};
pub use resource::{
    generate_resources, generate_resources_mapping, new_resource, resource_dir, Resource,
    ResourceDir,
};
pub use sets::{generate_resources_sets, SetSplitStrategie, SplitByCount};
