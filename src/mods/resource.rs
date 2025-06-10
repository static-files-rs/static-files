/*!
Resource definition and single module based generation.
 */
use std::{
    fs::{self, File, Metadata},
    io::{self, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use path_slash::PathExt;

/// Static files resource.
pub struct Resource {
    pub data: &'static [u8],
    pub modified: u64,
    pub mime_type: &'static str,
}

/// Used internally in generated functions.
#[inline]
#[must_use]
pub fn new_resource(data: &'static [u8], modified: u64, mime_type: &'static str) -> Resource {
    Resource {
        data,
        modified,
        mime_type,
    }
}

pub(crate) const DEFAULT_VARIABLE_NAME: &str = "r";

/// Generate resources for `project_dir` using `filter`.
/// Result saved in `generated_filename` and function named as `fn_name`.
///
/// in `build.rs`:
/// ```rust
/// use std::{env, path::Path};
/// use static_files::resource::generate_resources;
///
/// fn main() {
///     let out_dir = env::var("OUT_DIR").unwrap();
///     let generated_filename = Path::new(&out_dir).join("generated.rs");
///     generate_resources("./tests", None, generated_filename, "generate").unwrap();
/// }
/// ```
///
/// in `main.rs`:
/// ```rust
/// include!(concat!(env!("OUT_DIR"), "/generated.rs"));
///
/// fn main() {
///     let generated_file = generate();
///
///     assert_eq!(generated_file.len(), 4);
/// }
/// ```
pub fn generate_resources<P: AsRef<Path>, G: AsRef<Path>>(
    project_dir: P,
    filter: Option<fn(p: &Path) -> bool>,
    generated_filename: G,
    fn_name: &str,
) -> io::Result<()> {
    let resources = collect_resources(&project_dir, filter)?;

    let mut f = File::create(&generated_filename)?;

    generate_function_header(&mut f, fn_name)?;
    generate_uses(&mut f)?;

    generate_variable_header(&mut f, DEFAULT_VARIABLE_NAME)?;
    generate_resource_inserts(&mut f, &project_dir, DEFAULT_VARIABLE_NAME, &resources)?;
    generate_variable_return(&mut f, DEFAULT_VARIABLE_NAME)?;

    generate_function_end(&mut f)?;

    Ok(())
}

/// Generate resource mapping for `project_dir` using `filter`.
/// Result saved in `generated_filename` as anonymous block which returns `HashMap<&'static str, Resource>`.
///
/// in `build.rs`:
/// ```rust
///
/// use std::{env, path::Path};
/// use static_files::resource::generate_resources_mapping;
///
/// fn main() {
///     let out_dir = env::var("OUT_DIR").unwrap();
///     let generated_filename = Path::new(&out_dir).join("generated_mapping.rs");
///     generate_resources_mapping("./tests", None, generated_filename).unwrap();
/// }
/// ```
///
/// in `main.rs`:
/// ```rust
/// use std::collections::HashMap;
///
/// use static_files::Resource;
///
/// fn generate_mapping() -> HashMap<&'static str, Resource> {
///   include!(concat!(env!("OUT_DIR"), "/generated_mapping.rs"))
/// }
///
/// fn main() {
///     let generated_file = generate_mapping();
///
///     assert_eq!(generated_file.len(), 4);
///
/// }
/// ```
pub fn generate_resources_mapping<P: AsRef<Path>, G: AsRef<Path>>(
    project_dir: P,
    filter: Option<fn(p: &Path) -> bool>,
    generated_filename: G,
) -> io::Result<()> {
    let resources = collect_resources(&project_dir, filter)?;

    let mut f = File::create(&generated_filename)?;
    writeln!(f, "{{")?;

    generate_uses(&mut f)?;

    generate_variable_header(&mut f, DEFAULT_VARIABLE_NAME)?;

    generate_resource_inserts(&mut f, &project_dir, DEFAULT_VARIABLE_NAME, &resources)?;

    generate_variable_return(&mut f, DEFAULT_VARIABLE_NAME)?;

    writeln!(f, "}}")?;
    Ok(())
}

#[cfg(not(feature = "sort"))]
pub(crate) fn collect_resources<P: AsRef<Path>>(
    path: P,
    filter: Option<fn(p: &Path) -> bool>,
) -> io::Result<Vec<(PathBuf, Metadata)>> {
    collect_resources_nested(path, filter)
}

#[cfg(feature = "sort")]
pub(crate) fn collect_resources<P: AsRef<Path>>(
    path: P,
    filter: Option<fn(p: &Path) -> bool>,
) -> io::Result<Vec<(PathBuf, Metadata)>> {
    let mut resources = collect_resources_nested(path, filter)?;
    resources.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(resources)
}

#[inline]
fn collect_resources_nested<P: AsRef<Path>>(
    path: P,
    filter: Option<fn(p: &Path) -> bool>,
) -> io::Result<Vec<(PathBuf, Metadata)>> {
    let mut result = vec![];

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ref filter) = filter {
            if !filter(path.as_ref()) {
                continue;
            }
        }

        if path.is_dir() {
            let nested = collect_resources(path, filter)?;
            result.extend(nested);
        } else {
            result.push((path, entry.metadata()?));
        }
    }

    Ok(result)
}

pub(crate) fn generate_resource_inserts<P: AsRef<Path>, W: Write>(
    f: &mut W,
    project_dir: &P,
    variable_name: &str,
    resources: &[(PathBuf, Metadata)],
) -> io::Result<()> {
    for resource in resources {
        generate_resource_insert(f, project_dir, variable_name, resource)?;
    }
    Ok(())
}

#[allow(clippy::unnecessary_debug_formatting)]
pub(crate) fn generate_resource_insert<P: AsRef<Path>, W: Write>(
    f: &mut W,
    project_dir: &P,
    variable_name: &str,
    resource: &(PathBuf, Metadata),
) -> io::Result<()> {
    let (path, metadata) = resource;
    let abs_path = path.canonicalize()?;
    let key_path = path.strip_prefix(project_dir).unwrap().to_slash().unwrap();

    let modified = if let Ok(Ok(modified)) = metadata
        .modified()
        .map(|x| x.duration_since(SystemTime::UNIX_EPOCH))
    {
        modified.as_secs()
    } else {
        0
    };
    let mime_type = mime_guess::MimeGuess::from_path(path).first_or_octet_stream();
    writeln!(
        f,
        "{variable_name}.insert({key_path:?},n(i!({abs_path:?}),{modified:?},{mime_type:?}));",
    )
}

pub(crate) fn generate_function_header<F: Write>(f: &mut F, fn_name: &str) -> io::Result<()> {
    writeln!(
        f,
        "#[allow(clippy::unreadable_literal)] pub fn {fn_name}() -> ::std::collections::HashMap<&'static str, ::static_files::Resource> {{",
    )
}

pub(crate) fn generate_function_end<F: Write>(f: &mut F) -> io::Result<()> {
    writeln!(f, "}}")
}

pub(crate) fn generate_uses<F: Write>(f: &mut F) -> io::Result<()> {
    writeln!(
        f,
        "\
use ::static_files::resource::new_resource as n;
use ::std::include_bytes as i;",
    )
}

pub(crate) fn generate_variable_header<F: Write>(f: &mut F, variable_name: &str) -> io::Result<()> {
    writeln!(
        f,
        "let mut {variable_name} = ::std::collections::HashMap::new();",
    )
}

pub(crate) fn generate_variable_return<F: Write>(f: &mut F, variable_name: &str) -> io::Result<()> {
    writeln!(f, "{variable_name}")
}
