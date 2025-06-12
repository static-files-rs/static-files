/*!
`npm` support.
*/
use std::{
    env, fs,
    io::{self},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use super::resource_dir::ResourceDir;

#[cfg(not(windows))]
const NPM_CMD: &str = "npm";

#[cfg(windows)]
const NPM_CMD: &str = "npm.cmd";

/// Generate resources with run of `npm install` prior to collecting
/// resources in `resource_dir`.
///
/// Resources collected in `node_modules` subdirectory.
pub fn npm_resource_dir<P: AsRef<Path>>(resource_dir: P) -> io::Result<ResourceDir> {
    #[allow(unused_mut)]
    let mut npm_build = NpmBuild::new(resource_dir)
        .node_modules_strategy(NodeModulesStrategy::MoveToOutDir)
        .install()?;

    #[cfg(feature = "change-detection")]
    {
        npm_build = npm_build.change_detection();
    }

    Ok(npm_build.into())
}

/// Executes `npm` commands before collecting resources.
///
/// Example usage:
/// Add `build.rs` with call to bundle resources:
///
/// ```rust, no_run
/// use static_files::NpmBuild;
///
/// fn main() {
///     NpmBuild::new("./web")
///         .install().unwrap() // runs npm install
///         .run("build").unwrap() // runs npm run build
///         .target("./web/dist")
///         .to_resource_dir()
///         .build().unwrap();
/// }
/// ```
/// Include generated code in `main.rs`:
///
/// ```rust, ignore
/// include!(concat!(env!("OUT_DIR"), "/generated.rs"));
/// ```
#[derive(Default, Debug)]
pub struct NpmBuild {
    package_json_dir: PathBuf,
    executable: String,
    target_dir: Option<PathBuf>,
    node_modules_strategy: NodeModulesStrategy,
    stderr: Option<Stdio>,
    stdout: Option<Stdio>,
}

impl NpmBuild {
    pub fn new<P: AsRef<Path>>(package_json_dir: P) -> Self {
        Self {
            package_json_dir: package_json_dir.as_ref().into(),
            executable: String::from(NPM_CMD),
            ..Default::default()
        }
    }

    /// Allow the user to set their own npm-like executable (like yarn, for instance)
    #[must_use]
    pub fn executable(self, executable: &str) -> Self {
        let executable = String::from(executable);
        Self { executable, ..self }
    }

    /// Generates change detection instructions.
    ///
    /// It includes `package.json` directory, ignores by default `node_modules`, `package.json` and `package-lock.json` and target directory.
    /// Each time `npm` changes timestamps on these files, so if we do not ignore them - it runs `npm` each time.
    /// It is recommended to put your dist files one level deeper. For example, if you have `web` with `package.json`
    /// and `dist` just below that, you better generate you index.html somewhere in `web\dist\sub_path\index.html`.
    /// Reason is the same, `npm` touches `dist` each time and it touches the parent directory which in its turn triggers the build each time.
    /// For complete example see: [Angular Router Sample](https://github.com/kilork/actix-web-static-files-example-angular-router).
    /// If default behavior does not work for you, you can use [change-detection](https://crates.io/crates/change-detection) directly.
    #[cfg(feature = "change-detection")]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn change_detection(self) -> Self {
        use ::change_detection::{
            path_matchers::{any, equal, func, starts_with, PathMatcherExt},
            ChangeDetection,
        };

        let package_json_dir = self.package_json_dir.clone();
        let default_exclude_filter = any!(
            equal(package_json_dir.clone()),
            starts_with(self.package_json_dir.join("node_modules")),
            equal(self.package_json_dir.join("package.json")),
            equal(self.package_json_dir.join("package-lock.json")),
            func(move |p| { p.is_file() && p.parent() != Some(package_json_dir.as_path()) })
        );

        {
            // TODO: rework this code to not panic
            let change_detection = if self.target_dir.is_none() {
                ChangeDetection::exclude(default_exclude_filter)
            } else {
                let mut target_dir = self.target_dir.clone().unwrap();

                if let Some(target_dir_parent) = target_dir.parent() {
                    if target_dir_parent.starts_with(&self.package_json_dir) {
                        while target_dir.parent() != Some(&self.package_json_dir) {
                            target_dir = target_dir.parent().unwrap().into();
                        }
                    }
                }

                let exclude_filter = default_exclude_filter.or(starts_with(target_dir));
                ChangeDetection::exclude(exclude_filter)
            };

            change_detection.path(&self.package_json_dir).generate();
        }
        self
    }

    /// Executes `npm install`.
    pub fn install(mut self) -> io::Result<Self> {
        self.package_command()
            .arg("install")
            .status()
            .map_err(|err| {
                eprintln!("Cannot execute {} install: {err:?}", self.executable);
                err
            })
            .map(|_| self)
    }

    /// Executes `npm run CMD`.
    pub fn run(mut self, cmd: &str) -> io::Result<Self> {
        self.package_command()
            .arg("run")
            .arg(cmd)
            .status()
            .map_err(|err| {
                eprintln!("Cannot execute {} run {cmd}: {err:?}", self.executable);
                err
            })
            .map(|_| self)
    }

    /// Sets target (default is `node_modules`).
    ///
    /// The `OUT_DIR` variable is automatically prepended.
    /// Do not forget to adjust your JS side accordingly.
    /// Use absolute path to avoid this behavior.
    #[must_use]
    pub fn target<P: AsRef<Path>>(mut self, target_dir: P) -> Self {
        let target_dir = target_dir.as_ref();
        self.target_dir = Some(if target_dir.is_absolute() {
            target_dir.into()
        } else if let Ok(out_dir) = env::var("OUT_DIR").map(PathBuf::from) {
            out_dir.join(target_dir)
        } else {
            target_dir.into()
        });
        self
    }

    /// Sets stderr for the next command.
    ///
    /// You should set it again, if you need also redirect output for the next command.
    #[must_use]
    pub fn stderr<S: Into<Stdio>>(mut self, stdio: S) -> Self {
        self.stderr = Some(stdio.into());
        self
    }

    /// Sets stdout for the next command.
    ///
    /// You should set it again, if you need also redirect output for the next command.
    #[must_use]
    pub fn stdout<S: Into<Stdio>>(mut self, stdio: S) -> Self {
        self.stdout = Some(stdio.into());
        self
    }

    /// Sets the strategy to executed upon building the `ResourceDir`.
    ///
    /// Default behavior is to clean `node_modules` directory.
    #[must_use]
    pub fn node_modules_strategy(mut self, node_modules_strategy: NodeModulesStrategy) -> Self {
        self.node_modules_strategy = node_modules_strategy;
        self
    }

    /// Converts to `ResourceDir`.
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_resource_dir(self) -> ResourceDir {
        self.into()
    }

    #[cfg(not(windows))]
    fn command(&self) -> Command {
        Command::new(&self.executable)
    }

    #[cfg(windows)]
    fn command(&self) -> Command {
        let mut cmd = Command::new("cmd");

        cmd.arg("/c").arg(&self.executable);

        cmd
    }

    fn package_command(&mut self) -> Command {
        let mut cmd = self.command();

        cmd.stderr(self.stderr.take().unwrap_or_else(Stdio::inherit))
            .stdout(self.stdout.take().unwrap_or_else(Stdio::inherit))
            .current_dir(&self.package_json_dir);

        cmd
    }

    fn to_node_modules_dir(&self) -> PathBuf {
        self.package_json_dir.join("node_modules")
    }

    fn remove_node_modules(&self) -> io::Result<()> {
        let node_modules_dir = self.to_node_modules_dir();

        if node_modules_dir.is_dir() {
            fs::remove_dir_all(node_modules_dir)?;
        }

        Ok(())
    }

    fn move_node_modules_to_out_dir(&self) -> io::Result<PathBuf> {
        let node_modules_dir = self.to_node_modules_dir();

        if !node_modules_dir.is_dir() {
            return Ok(node_modules_dir);
        }

        let Ok(out_node_modules_dir) =
            env::var("OUT_DIR").map(|out_dir| PathBuf::from(out_dir).join("node_modules"))
        else {
            return Ok(node_modules_dir);
        };

        copy_dir_all(&node_modules_dir, &out_node_modules_dir)?;
        fs::remove_dir_all(node_modules_dir)?;

        Ok(out_node_modules_dir)
    }
}

impl From<NpmBuild> for ResourceDir {
    fn from(mut npm_build: NpmBuild) -> Self {
        let resource_dir = npm_build
            .target_dir
            .take()
            .unwrap_or_else(|| npm_build.to_node_modules_dir());

        let resource_dir = npm_build
            .node_modules_strategy
            .execute(resource_dir, &npm_build);

        Self {
            resource_dir,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub enum NodeModulesStrategy {
    #[default]
    Clean,
    MoveToOutDir,
}

impl NodeModulesStrategy {
    fn execute(&self, resource_dir: PathBuf, npm_build: &NpmBuild) -> PathBuf {
        match self {
            Self::Clean => {
                npm_build
                    .remove_node_modules()
                    .expect("remove node_modules dir");
            }
            Self::MoveToOutDir => {
                return npm_build
                    .move_node_modules_to_out_dir()
                    .expect("move node_modules to out dir");
            }
        }
        resource_dir
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
