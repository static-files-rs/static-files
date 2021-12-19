use std::{collections::HashMap, marker::PhantomData};

use super::{Resource, NAMESPACE};

pub trait ResourceStorage<T>: std::fmt::Debug
where
    T: AsRef<str>,
{
    fn get(&self, key: &str) -> Option<&Resource<T>>;
}

impl<T> ResourceStorage<T> for T
where
    T: AsRef<str> + std::fmt::Debug,
{
    fn get(&self, _key: &str) -> Option<&Resource<T>> {
        None
    }
}

pub trait ResourceStorageType {
    fn namespace(&self) -> &'static str;

    fn storage_type(&self) -> &'static str;

    fn tag_type(&self) -> &'static str {
        "String"
    }
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
}

#[derive(Debug)]
pub struct HashMapResourceStorage<E: AsRef<str> = String>(HashMap<&'static str, Resource<E>>);

impl<T> ResourceStorage<T> for HashMapResourceStorage<T>
where
    T: AsRef<str> + std::fmt::Debug,
{
    fn get(&self, key: &str) -> Option<&Resource<T>> {
        self.0.get(key)
    }
}
