use crate::mods::resource::generate_resources;
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
}

impl ResourceDir {
    /// Generates resources for current configuration.
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
    pub fn with_generated_fn(&mut self, generated_fn: impl Into<String>) -> &mut Self {
        self.generated_fn = Some(generated_fn.into());
        self
    }
}
