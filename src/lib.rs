/*!
# static-files description

## Features

## Usage

Add dependency to Cargo.toml:

```toml
[dependencies]
static-files = "0.1"
```

*/

mod mods;

pub use mods::{
    generate_resources, generate_resources_mapping, generate_resources_sets, new_resource,
    npm_resource_dir, resource_dir, NpmBuild, Resource, ResourceDir, SetSplitStrategie,
    SplitByCount,
};
