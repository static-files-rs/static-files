pub trait ResourceStorage {}

pub struct ResourceStorages;

impl ResourceStorages {
    pub fn hash_map() -> HashMapResourceStorage {
        Default::default()
    }
}

pub struct HashMapResourceStorage {}

impl Default for HashMapResourceStorage {
    fn default() -> Self {
        Self {}
    }
}

impl ResourceStorage for HashMapResourceStorage {}
