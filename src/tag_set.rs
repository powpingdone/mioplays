use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// `Tag`: a sealed marker trait for interacting with the `TagMap` in a
/// typed manner.
mod private {
    pub trait Sealed {}
}
pub trait Tag: private::Sealed {}
macro_rules! tag_impl {
    ($tag:ty) => {
        impl private::Sealed for $tag {}
        impl Tag for $tag {}
    };
}

pub struct Title(pub String);
tag_impl!(Title);

pub struct Artist(pub String);
tag_impl!(Artist);

pub struct Album(pub String);
tag_impl!(Album);

pub struct EncodedCoverArt(pub Box<[u8]>);
tag_impl!(EncodedCoverArt);

/// A private enum for containing both a custom `String` id and
/// a `TypeId`. Used for allowing typed HashMap accesses along with
/// untyped, custom tags.
#[derive(Hash, PartialEq, Eq, Clone)]
enum TIDOrCustom {
    TypeId(TypeId),
    Custom(String),
}

impl From<TypeId> for TIDOrCustom {
    fn from(value: TypeId) -> Self {
        Self::TypeId(value)
    }
}

impl From<String> for TIDOrCustom {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}

impl From<&str> for TIDOrCustom {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_owned())
    }
}

/// A Mapping of tags to custom structures. The tags may be
/// defined by a specific struct with the `Tag` trait, or a string.
/// Fetching a typed item will return it's associated struct, while a
/// string will only ever return a `dyn Any`
pub struct TagSet {
    map: HashMap<TIDOrCustom, Box<dyn Any + 'static>>,
}

// Not related to accesses.
impl TagSet {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

// Typed accessing of fields
impl TagSet {
    /// Add tag to set. If the associated tag is already in the set,
    /// return the argument as an error.
    pub fn push_typed_tag<K: Tag + 'static>(&mut self, tag: K) -> Result<(), K> {
        if self.map.contains_key(&tag.type_id().into()) {
            Err(tag)
        } else {
            let ret = self.map.insert(tag.type_id().into(), Box::new(tag));
            if ret.is_some() {
                panic!("somehow, we don't contain a key we have access to now")
            }
            Ok(())
        }
    }

    /// Fetch a immutable reference to a typed `Tag`.
    pub fn get_typed_tag<K: Tag + 'static>(&self) -> Option<&K> {
        let type_id = TypeId::of::<K>();
        self.map.get(&type_id.into())?.downcast_ref()
    }

    /// Fetch and return a typed `Tag`, removing it from the `TagMap`.
    pub fn drop_typed_tag<K: Tag + 'static>(&mut self) -> Option<K> {
        let type_id = TypeId::of::<K>();
        let ret = self.map.remove(&type_id.into())?.downcast::<K>();
        if ret.is_err() {
            panic!(
                "tag map type mismatch: expected {:?} (aka {}) but the key was not that",
                type_id,
                std::any::type_name::<K>(),
            );
        } else {
            Some(*ret.unwrap())
        }
    }
}

// Stringly accessing fields
impl TagSet {
    /// Add tag to set which does not have an associated type, but a custom string.
    /// If the associated tag is already in the set, return the argument as an error.
    pub fn push_custom_tag(
        &mut self,
        key: impl AsRef<str>,
        value: Box<dyn Any + 'static>,
    ) -> Result<(), Box<dyn Any + 'static>> {
        let key = key.as_ref().into();
        if self.map.contains_key(&key) {
            Err(value)
        } else {
            let ret = self.map.insert(key, value);
            if ret.is_some() {
                panic!("somehow, we don't contain a *string* key that exists during insertion")
            }
            Ok(())
        }
    }

    /// Fetch a reference to an associated custom tag object.
    pub fn get_custom_tag(&self, key: impl AsRef<str>) -> Option<&Box<dyn Any + 'static>> {
        self.map.get(&key.as_ref().into())
    }

    /// Fetch and return an associated custom tag object, removing it from the `TagMap`.
    pub fn drop_custom_tag(&mut self, key: impl AsRef<str>) -> Option<Box<dyn Any + 'static>> {
        self.map.remove(&key.as_ref().into())
    }
}
