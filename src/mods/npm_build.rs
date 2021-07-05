/*!
`npm` support.
*/
use std::{
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
    let mut npm_build = NpmBuild::new(resource_dir).install()?;

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
/// ```rust#ignore
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
/// ```rust#ignore
/// include!(concat!(env!("OUT_DIR"), "/generated.rs"));
/// ```
#[derive(Default, Debug)]
pub struct NpmBuild {
    package_json_dir: PathBuf,
    executable: String,
    target_dir: Option<PathBuf>,
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
                eprintln!("Cannot execute {} install: {:?}", &self.executable, err);
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
                eprintln!("Cannot execute {} run {}: {:?}", &self.executable, cmd, err);
                err
            })
            .map(|_| self)
    }

    /// Sets target (default is node_modules).
    pub fn target<P: AsRef<Path>>(mut self, target_dir: P) -> Self {
        self.target_dir = Some(target_dir.as_ref().into());
        self
    }

    /// Sets stderr for the next command.
    ///
    /// You should set it again, if you need also redirect output for the next command.
    pub fn stderr<S: Into<Stdio>>(mut self, stdio: S) -> Self {
        self.stderr = Some(stdio.into());
        self
    }

    /// Sets stdout for the next command.
    ///
    /// You should set it again, if you need also redirect output for the next command.
    pub fn stdout<S: Into<Stdio>>(mut self, stdio: S) -> Self {
        self.stdout = Some(stdio.into());
        self
    }

    /// Converts to `ResourceDir`.
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

        cmd.stderr(self.stderr.take().unwrap_or_else(|| Stdio::inherit()))
            .stdout(self.stdout.take().unwrap_or_else(|| Stdio::inherit()))
            .current_dir(&self.package_json_dir);

        cmd
    }
}

impl From<NpmBuild> for ResourceDir {
    fn from(mut npm_build: NpmBuild) -> Self {
        Self {
            resource_dir: npm_build
                .target_dir
                .take()
                .unwrap_or_else(|| npm_build.package_json_dir.join("node_modules")),
            ..Default::default()
        }
    }
}
