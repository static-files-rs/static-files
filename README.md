# static-files - the library to help automate static resource collection

## Legal

Dual-licensed under `MIT` or the [UNLICENSE](http://unlicense.org/).

## Features

- Embed static resources in executuble
- Install dependencies with [npm](https://npmjs.org) package manager
- Run custom `npm` run commands (such as [webpack](https://webpack.js.org/))
- Support for npm-like package managers ([yarn](https://yarnpkg.com/))
- Change detection support to reduce compilation time

## Usage

Create folder with static resources in your project (for example `static`):

```bash
cd project_dir
mkdir static
echo "Hello, world" > static/hello
```

Add to `Cargo.toml` dependency to `static-files`:

```toml
[dependencies]
static-files = "0.3"

[build-dependencies]
static-files = "0.3"
```

Add `build.rs` with call to bundle resources:

```rust, no_run
use static_files::resource_dir;

fn main() -> std::io::Result<()> {
    resource_dir("./static").build()
}
```

Include generated code in `main.rs`:

```rust, no_run
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

fn main() -> std::io::Result<()> {
    let generated = generate(); // <-- this function is defined in generated.rs
    // ...
    Ok(())
}
```

By default, 'static-files' collects all files in operation system defined order, but you can change it by activating 'sort' feature in `Cargo.toml`:

```toml
[dependencies]
static-files = { version = "0.3", features = ["sort"] }

[build-dependencies]
static-files = { version = "0.3", features = ["sort"] }
```
