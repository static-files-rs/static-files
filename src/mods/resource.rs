use path_slash::PathExt;
use std::{
    env,
    fs::{self, File, Metadata},
    io::{self, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Static files resource.
pub struct Resource {
    pub data: &'static [u8],
    pub modified: u64,
    pub mime_type: &'static str,
}

#[inline]
pub fn new_resource(data: &'static [u8], modified: u64, mime_type: &'static str) -> Resource {
    Resource {
        data,
        modified,
        mime_type,
    }
}

/// Generate resources for `resource_dir`.
///
/// ```rust
/// // Generate resources for ./tests dir with file name generated.rs
/// // stored in path defined by OUT_DIR environment variable.
/// // Function name is 'generate'
/// use actix_web_static_files::resource_dir;
///
/// resource_dir("./tests").build().unwrap();
/// ```
pub fn resource_dir<P: AsRef<Path>>(resource_dir: P) -> ResourceDir {
    ResourceDir {
        resource_dir: resource_dir.as_ref().into(),
        ..Default::default()
    }
}

#[derive(Default)]
pub struct ResourceDir {
    pub(crate) resource_dir: PathBuf,
    pub(crate) filter: Option<fn(p: &Path) -> bool>,
    pub(crate) generated_filename: Option<PathBuf>,
    pub(crate) generated_fn: Option<String>,
}

impl ResourceDir {
    pub fn build(&self) -> io::Result<()> {
        let generated_filename = self.generated_filename.clone().unwrap_or_else(|| {
            let out_dir = env::var("OUT_DIR").unwrap();

            Path::new(&out_dir).join("generated.rs")
        });
        let generated_fn = self
            .generated_fn
            .clone()
            .unwrap_or_else(|| "generate".into());

        generate_resources(
            &self.resource_dir,
            self.filter,
            &generated_filename,
            &generated_fn,
        )
    }

    pub fn with_filter(&mut self, filter: fn(p: &Path) -> bool) -> &mut Self {
        self.filter = Some(filter);
        self
    }

    pub fn with_generated_filename<P: AsRef<Path>>(&mut self, generated_filename: P) -> &mut Self {
        self.generated_filename = Some(generated_filename.as_ref().into());
        self
    }

    pub fn with_generated_fn(&mut self, generated_fn: impl Into<String>) -> &mut Self {
        self.generated_fn = Some(generated_fn.into());
        self
    }
}

pub(crate) const DEFAULT_VARIABLE_NAME: &str = "r";

/// Generate resources for `project_dir` using `filter`.
/// Result saved in `generated_filename` and function named as `fn_name`.
///
/// in `build.rs`:
/// ```rust
///
/// use std::{env, path::Path};
/// use actix_web_static_files::generate_resources;
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
/// use actix_web::App;
///
/// include!(concat!(env!("OUT_DIR"), "/generated.rs"));
///
/// fn main() {
///     let generated_file = generate();
///
///     assert_eq!(generated_file.len(), 4);
///
///     let app = App::new()
///         .service(actix_web_static_files::ResourceFiles::new(
///            "/static",
///            generated_file,
///        ));
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
    generate_resource_inserts(&mut f, &project_dir, DEFAULT_VARIABLE_NAME, resources)?;
    generate_variable_return(&mut f, DEFAULT_VARIABLE_NAME)?;

    generate_function_end(&mut f)?;

    Ok(())
}

/// Generate resource mapping for `project_dir` using `filter`.
/// Result saved in `generated_filename` as anonymous block which returns HashMap<&'static str, Resource>.
///
/// in `build.rs`:
/// ```rust
///
/// use std::{env, path::Path};
/// use actix_web_static_files::generate_resources_mapping;
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
/// use actix_web::App;
/// use actix_web_static_files::{Resource, ResourceFiles};
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
///     let app = App::new()
///         .service(ResourceFiles::new(
///            "/static",
///            generated_file,
///        ));
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

    generate_resource_inserts(&mut f, &project_dir, DEFAULT_VARIABLE_NAME, resources)?;

    generate_variable_return(&mut f, DEFAULT_VARIABLE_NAME)?;

    writeln!(f, "}}")?;
    Ok(())
}

pub(crate) fn collect_resources<P: AsRef<Path>>(
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
    resources: Vec<(PathBuf, Metadata)>,
) -> io::Result<()> {
    for resource in &resources {
        generate_resource_insert(f, project_dir, variable_name, resource)?;
    }
    Ok(())
}

pub(crate) fn generate_resource_insert<P: AsRef<Path>, W: Write>(
    f: &mut W,
    project_dir: &P,
    variable_name: &str,
    resource: &(PathBuf, Metadata),
) -> io::Result<()> {
    let (path, metadata) = resource;
    let abs_path = path.canonicalize()?;
    let key_path = path.strip_prefix(&project_dir).unwrap().to_slash().unwrap();

    let modified = if let Ok(Ok(modified)) = metadata
        .modified()
        .map(|x| x.duration_since(SystemTime::UNIX_EPOCH))
    {
        modified.as_secs()
    } else {
        0
    };
    let mime_type = mime_guess::MimeGuess::from_path(&path).first_or_octet_stream();
    writeln!(
        f,
        "{}.insert({:?},n(i!({:?}),{:?},{:?}));",
        variable_name, &key_path, &abs_path, modified, &mime_type,
    )
}

pub(crate) fn generate_function_header<F: Write>(f: &mut F, fn_name: &str) -> io::Result<()> {
    writeln!(
        f,
        "#[allow(clippy::unreadable_literal)] pub fn {}() -> ::std::collections::HashMap<&'static str, ::actix_web_static_files::Resource> {{",
        fn_name
    )
}

pub(crate) fn generate_function_end<F: Write>(f: &mut F) -> io::Result<()> {
    writeln!(f, "}}")
}

pub(crate) fn generate_uses<F: Write>(f: &mut F) -> io::Result<()> {
    writeln!(
        f,
        "use ::actix_web_static_files::new_resource as n;
use ::std::include_bytes as i;",
    )
}

pub(crate) fn generate_variable_header<F: Write>(f: &mut F, variable_name: &str) -> io::Result<()> {
    writeln!(
        f,
        "let mut {} = ::std::collections::HashMap::new();",
        variable_name
    )
}

pub(crate) fn generate_variable_return<F: Write>(f: &mut F, variable_name: &str) -> io::Result<()> {
    writeln!(f, "{}", variable_name)
}
