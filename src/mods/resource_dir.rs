use super::sets::{generate_resources_sets, SplitByCount};
use std::{
    env, io,
    path::{Path, PathBuf},
};

/// Generate resources for `resource_dir`.
///
/// ```rust
/// // Generate resources for ./tests dir with file name generated.rs
/// // stored in path defined by OUT_DIR environment variable.
/// // Function name is 'generate'
/// use static_files::resource_dir;
///
/// resource_dir("./tests").build().unwrap();
/// ```
pub fn resource_dir<P: AsRef<Path>>(resource_dir: P) -> ResourceDir {
    ResourceDir {
        resource_dir: resource_dir.as_ref().into(),
        ..Default::default()
    }
}

/// Resource dir.
///
/// A builder structure allows to change default settings for:
/// - file filter
/// - generated file name
/// - generated function name
#[derive(Default)]
pub struct ResourceDir {
    pub(crate) resource_dir: PathBuf,
    pub(crate) filter: Option<fn(p: &Path) -> bool>,
    pub(crate) generated_filename: Option<PathBuf>,
    pub(crate) generated_fn: Option<String>,
    pub(crate) module_name: Option<String>,
    pub(crate) count_per_module: Option<usize>,
}

pub const DEFAULT_MODULE_NAME: &str = "sets";
pub const DEFAULT_COUNT_PER_MODULE: usize = 256;

impl ResourceDir {
    /// Generates resources for current configuration.
    pub fn build(self) -> io::Result<()> {
        let generated_filename = self.generated_filename.unwrap_or_else(|| {
            let out_dir = env::var("OUT_DIR").unwrap();

            Path::new(&out_dir).join("generated.rs")
        });
        let generated_fn = self.generated_fn.unwrap_or_else(|| "generate".into());

        let module_name = self
            .module_name
            .unwrap_or_else(|| format!("{}_{}", &generated_fn, DEFAULT_MODULE_NAME));

        let count_per_module = self.count_per_module.unwrap_or(DEFAULT_COUNT_PER_MODULE);

        generate_resources_sets(
            &self.resource_dir,
            self.filter,
            &generated_filename,
            module_name.as_str(),
            &generated_fn,
            &mut SplitByCount::new(count_per_module),
        )
    }

    /// Sets the file filter.
    pub fn with_filter(&mut self, filter: fn(p: &Path) -> bool) -> &mut Self {
        self.filter = Some(filter);
        self
    }

    /// Sets the generated filename.
    pub fn with_generated_filename<P: AsRef<Path>>(&mut self, generated_filename: P) -> &mut Self {
        self.generated_filename = Some(generated_filename.as_ref().into());
        self
    }

    /// Sets the generated function name.
    pub fn with_generated_fn<S>(&mut self, generated_fn: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.generated_fn = Some(generated_fn.into());
        self
    }

    /// Sets the generated module name.
    ///
    /// Default value is based on generated function name and the suffix "sets".
    /// Generated module would be overriden by each call.
    pub fn with_module_name<S>(&mut self, module_name: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.module_name = Some(module_name.into());
        self
    }

    /// Sets maximal count of files per module.
    pub fn with_count_per_module(&mut self, count_per_module: usize) -> &mut Self {
        self.count_per_module = Some(count_per_module);
        self
    }
}
