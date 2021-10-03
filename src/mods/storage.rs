use std::{collections::HashMap, marker::PhantomData};

use super::{NAMESPACE, Resource};

pub trait ResourceStorage: std::fmt::Debug {
    fn get<T: AsRef<str>>(key: &str) -> Option<&Resource<T>>;
}

impl<T> ResourceStorage for T where T: AsRef<str> + std::fmt::Debug {
    fn get<E: AsRef<str>>(_key: &str) -> Option<&Resource<E>> {
        None
    }
}

pub trait ResourceStorageType {
    fn namespace(&self) -> &'static str;
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
}

#[derive(Debug)]
pub struct HashMapResourceStorage<E: AsRef<str> = String>(HashMap<&'static str, Resource<E>>);

impl<T> ResourceStorage for HashMapResourceStorage<T> where T: AsRef<str> + std::fmt::Debug {
    fn get<E: AsRef<str>>(_key: &str) -> Option<&Resource<E>> {
        None
    }
}
