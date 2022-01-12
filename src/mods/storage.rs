use std::{collections::HashMap, io::Write, marker::PhantomData, time::SystemTime};

use super::{Resource, ResourceError, ResourceFile, ResourcePrototype, NAMESPACE};

pub trait ResourceStorage<T>: std::fmt::Debug
where
    T: AsRef<str>,
{
    fn get(&self, key: &str) -> Option<&Resource<T>>;
    fn put(&mut self, key: &'static str, resource: Resource<T>);
}

impl<T> ResourceStorage<T> for T
where
    T: AsRef<str> + std::fmt::Debug,
{
    fn get(&self, _key: &str) -> Option<&Resource<T>> {
        None
    }

    fn put(&mut self, _key: &'static str, _resource: Resource<T>) {
        panic!("Unsupported insert into AsRef<str>.");
    }
}

pub trait ResourceStorageType {
    fn namespace(&self) -> &'static str;

    fn storage_type(&self) -> &'static str;

    fn storage_constructor(&self) -> &'static str;

    fn tag_type(&self) -> &'static str {
        "String"
    }

    fn default_variable_name(&self) -> &str {
        "r"
    }

    fn impl_signature(&self) -> String {
        format!(
            "impl ::{}::ResourceStorage<{}>",
            self.namespace(),
            self.tag_type()
        )
    }

    fn write_resource(
        &self,
        f: &mut impl Write,
        resource: ResourcePrototype,
    ) -> Result<(), ResourceError>;
}

pub trait ResourceStorageNamespace {
    fn namespace() -> &'static str {
        NAMESPACE
    }
}

pub struct DefaultResourceStorages;

impl ResourceStorageNamespace for DefaultResourceStorages {}

pub type ResourceStorages = NamespacedResourceStorages<DefaultResourceStorages>;

pub struct NamespacedResourceStorages<NS: ResourceStorageNamespace>(PhantomData<NS>);

impl<NS: ResourceStorageNamespace> NamespacedResourceStorages<NS> {
    pub fn hash_map() -> HashMapResourceStorageType<NS> {
        Default::default()
    }
}

pub struct HashMapResourceStorageType<NS: ResourceStorageNamespace>(PhantomData<NS>);

impl<NS: ResourceStorageNamespace> Default for HashMapResourceStorageType<NS> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<NS: ResourceStorageNamespace> ResourceStorageType for HashMapResourceStorageType<NS> {
    fn namespace(&self) -> &'static str {
        NS::namespace()
    }

    fn storage_type(&self) -> &'static str {
        "::std::collections::HashMap"
    }

    fn storage_constructor(&self) -> &'static str {
        "::std::collections::HashMap::new"
    }

    fn write_resource(
        &self,
        f: &mut impl Write,
        resource: ResourcePrototype,
    ) -> Result<(), ResourceError> {
        match resource {
            ResourcePrototype::Basic {
                resource_file:
                    ResourceFile {
                        url,
                        path,
                        metadata,
                    },
            } => {
                let modified = if let Ok(Ok(modified)) = metadata
                    .modified()
                    .map(|x| x.duration_since(SystemTime::UNIX_EPOCH))
                {
                    modified.as_secs()
                } else {
                    0
                };
                let mime_type = mime_guess::MimeGuess::from_path(&path).first_or_octet_stream();
                let abs_path = path.canonicalize()?;
                writeln!(
                    f,
                    "{}.put({:?},n(i!({:?}),{:?},{:?}));",
                    self.default_variable_name(),
                    &url,
                    &abs_path,
                    modified,
                    &mime_type,
                )
                .map_err(From::from)
            }
            ResourcePrototype::Compressed { compressed_file: _ } => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct HashMapResourceStorage<E: AsRef<str> = String>(HashMap<&'static str, Resource<E>>);

impl <T> HashMapResourceStorage<T> where T: AsRef<str> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl<T> ResourceStorage<T> for HashMapResourceStorage<T>
where
    T: AsRef<str> + std::fmt::Debug,
{
    fn get(&self, key: &str) -> Option<&Resource<T>> {
        self.0.get(key)
    }

    fn put(&mut self, key: &'static str, resource: Resource<T>) {
        self.0.insert(key, resource);
    }
}
