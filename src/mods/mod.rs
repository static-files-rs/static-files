use core::fmt;
use path_slash::PathExt;
use std::{
    error::Error,
    fs::{DirEntry, Metadata},
    io,
    path::{Path, PathBuf},
};

pub mod npm_build;
pub mod resource;
pub mod resource_dir;
pub mod sets;

pub mod compress;
pub mod generate;
pub mod storage;

pub use self::compress::*;
pub use self::generate::*;
pub use self::storage::*;

pub const NAMESPACE: &str = "static_files";

pub struct ResourceFiles {
    path: PathBuf,
    read_dir: Option<Box<dyn Iterator<Item = io::Result<DirEntry>>>>,
}

#[derive(Debug)]
pub enum ResourceError {
    InputOutputError(io::Error),
    WrongOutDir
}

impl Error for ResourceError {}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::InputOutputError(e) => write!(f, "input/output error: {}", e),
            ResourceError::WrongOutDir => write!(f, "OUT_DIR environment variable is not defined or wrong"),
        }
    }
}

impl From<io::Error> for ResourceError {
    fn from(err: io::Error) -> Self {
        Self::InputOutputError(err)
    }
}

type Result<T, Q = ResourceError> = ::std::result::Result<T, Q>;

#[derive(Debug)]
pub struct ResourceFile {
    url: String,
    path: PathBuf,
    metadata: Metadata,
}

impl ResourceFiles {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            read_dir: Some(Box::new(std::fs::read_dir(&path)?)),
        })
    }

    fn process(&mut self, dir_entry: DirEntry) -> Option<Result<ResourcePrototype>> {
        let path = dir_entry.path();
        if path.is_dir() {
            let subdir = match std::fs::read_dir(&path) {
                Ok(subdir) => subdir,
                Err(err) => return Some(Err(err.into())),
            };
            self.read_dir = Some(Box::new(subdir.chain(self.read_dir.take()?)));
            self.next()
        } else {
            let metadata = match dir_entry.metadata() {
                Ok(metadata) => metadata,
                Err(err) => return Some(Err(err.into())),
            };
            let url = path.strip_prefix(&self.path).unwrap().to_slash().unwrap();
            Some(Ok(ResourceFile {
                url,
                path,
                metadata,
            }
            .into()))
        }
    }
}

impl Iterator for ResourceFiles {
    type Item = Result<ResourcePrototype>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_dir
            .as_mut()?
            .next()?
            .map_or_else(|e| Some(Err(e.into())), |d| self.process(d))
    }
}

#[derive(Debug)]
pub enum ResourcePrototype {
    Basic {
        resource_file: ResourceFile,
    },
    Compressed {
        compressed_file: CompressedResourceFile,
    },
}

impl From<ResourceFile> for ResourcePrototype {
    fn from(resource_file: ResourceFile) -> Self {
        Self::Basic { resource_file }
    }
}

#[derive(Debug)]
pub struct Resource<T: AsRef<str> = &'static str> {
    data: Data,
    tag: T,
    mime_type: &'static str,
}

#[derive(Debug)]
pub enum Data {
    Basic(&'static [u8]),
    Compressed(Compressed),
}

pub struct Adapter<C: Convert, IT> {
    iter: IT,
    converter: C,
}

impl<C, IT> Iterator for Adapter<C, IT>
where
    IT: Iterator<Item = Result<ResourcePrototype>>,
    C: Convert,
{
    type Item = Result<ResourcePrototype>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|x| x.and_then(|y| self.converter.convert(y)))
    }
}

pub trait Convert {
    fn convert(&self, input: ResourcePrototype) -> Result<ResourcePrototype>;
}

pub trait ResourceFileAdapters: Iterator<Item = Result<ResourcePrototype>> + Sized {
    fn compress<C>(self, compresser: C) -> Adapter<C, Self>
    where
        C: Convert,
    {
        Adapter {
            iter: self,
            converter: compresser,
        }
    }

    fn generate<S>(self, resource_storage: S) -> Generate<Self, S>
    where
        S: ResourceStorageType,
    {
        Generate {
            iter: self,
            resource_storage,
        }
    }
}

impl<T> ResourceFileAdapters for T where T: Iterator<Item = Result<ResourcePrototype>> {}
