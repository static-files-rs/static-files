use super::{Convert, ResourceFile, ResourcePrototype, Result};

#[derive(Debug)]
pub enum CompressedResourceFile {
    Brotli(ResourceFile),
    Compress(ResourceFile),
    Deflate(ResourceFile),
    Exi(ResourceFile),
    Gzip(ResourceFile),
    Identity(ResourceFile),
    Pack200(ResourceFile),
    Zstd(ResourceFile),
    Noop(ResourceFile),
}

#[derive(Debug)]
pub enum Compressed {
    Brotli(&'static [u8]),
    Compress(&'static [u8]),
    Deflate(&'static [u8]),
    Exi(&'static [u8]),
    Gzip(&'static [u8]),
    Identity(&'static [u8]),
    Pack200(&'static [u8]),
    Zstd(&'static [u8]),
    Noop(&'static [u8]),
}

pub struct NoopCompressConverter;

impl Convert for NoopCompressConverter {
    fn convert(&self, input: ResourcePrototype) -> Result<ResourcePrototype> {
        Ok(input.into())
    }
}
