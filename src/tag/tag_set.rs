use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::{self, Debug},
};

/// `Tag`: a sealed marker trait for interacting with the `TagMap` in a
/// typed manner.
mod private {
    pub trait Sealed {}
}
pub trait Tag: private::Sealed + Any + Debug {
    fn to_any(&self) -> &(dyn Any + 'static);
    fn to_any_boxed(self: Box<Self>) -> Box<dyn Any + 'static>;
    fn display_name(&self) -> Option<&str>;
}
macro_rules! tag_impl {
    ($tag:ident {$($name:ident : $typ:ty),+} => $display_name:expr) => {
        #[derive(Debug)]
        pub struct $tag {
            $( $name: $typ, )+
        }

        tag_trait_impl!($tag: $display_name);
    };
    ($tag:ident as $inner:ty => $display_name:expr) => {
        #[derive(Debug)]
        pub struct $tag(pub $inner);

        tag_trait_impl!($tag: $display_name);
    };

}
macro_rules! tag_trait_impl {
    ($tag:ident: $display_name:expr) => {
        impl private::Sealed for $tag {}
        impl Tag for $tag {
            fn to_any(&self) -> &(dyn Any + 'static) {
                self
            }

            fn to_any_boxed(self: Box<Self>) -> Box<dyn Any + 'static> {
                self
            }

            fn display_name(&self) -> Option<&str> {
                Some($display_name)
            }
        }
    };
}

// List of all tag structs being mapped to the appropriate item

// some notes before this arduous hell:
//
// * any tags that holds an artist name is expected to hold multiple artist names
//
// * any tags that involve sort order will insert it into the struct of the same name
// (AlbumTitleSortOrder -> AlbumTitle {inner: String, sort_order: Option<String>})

// Titles/Subtitles
tag_impl!(AlbumTitle {inner: String, sort_order: String} => "Album Title");
tag_impl!(TrackTitle {inner: String, sort_order: String} => "Track Title");
tag_impl!(DiscTitle as String => "Disc Title"); // title of individual disk

// Creators (Artists) & Credits
tag_impl!(AlbumArtist {inner: Vec<String>, sort_order: Vec<String>} => "Album Artist");
tag_impl!(TrackArtist {inner: Vec<String>, sort_order: Vec<String>} => "Track Artist");
tag_impl!(Composer {inner: String, sort_order: String} => "Composer");
tag_impl!(Performer as Vec<String> => "Performer");
tag_impl!(Remixer as Vec<String> => "Remixer");

// Music information
//ReplayGainAlbumGain
//ReplayGainAlbumPeak
//ReplayGainTrackGain
//ReplayGainTrackPeak
//Genre
//InitialKey
//Color
//Mood
//Bpm
//IntegerBpm

// Other
tag_impl!(DiscPos as u32 => "Disk Number");
tag_impl!(DiscTotal as u32 => "Number of Disks");
tag_impl!(TrackPos as u32 => "Track Number");
tag_impl!(TrackTotal as u32 => "Number of Tracks");
tag_impl!(ReleaseDate as jiff::Timestamp => "Release Date");

//Comment
//Description
//Language
//Script
//Lyrics

// Special tag for the string insert
#[derive(Debug)]
pub struct UnknownItem(Box<dyn Any + Send + Sync + 'static>);
impl private::Sealed for UnknownItem {}
impl Tag for UnknownItem {
    fn to_any(&self) -> &(dyn Any + 'static) {
        self
    }

    fn to_any_boxed(self: Box<Self>) -> Box<dyn Any + 'static> {
        self
    }

    fn display_name(&self) -> Option<&str> {
        None
    }
}

// Special tag for the Cover Art
#[derive(Debug)]
pub struct EncodedCoverArt(Box<[u8]>);
impl private::Sealed for EncodedCoverArt {}
impl Tag for EncodedCoverArt {
    fn to_any(&self) -> &(dyn Any + 'static) {
        self
    }

    fn to_any_boxed(self: Box<Self>) -> Box<dyn Any + 'static> {
        self
    }

    fn display_name(&self) -> Option<&str> {
        None
    }
}

/// A private enum for containing both a custom `String` id and
/// a `TypeId`. Used for allowing typed HashMap accesses along with
/// untyped, custom tags.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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
/// string will only ever return a `dyn Tag` which can be cast into a
/// `dyn Any` using the `Tag::to_any` and `Tag::to_any_boxed` methods.
#[derive(Debug)]
pub struct TagSet {
    map: HashMap<TIDOrCustom, Box<dyn Tag + Send + Sync + 'static>>,
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
    pub fn push_typed_tag<K: Tag + Send + Sync + 'static>(&mut self, tag: K) -> Result<(), K> {
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
    pub fn get_typed_tag<K: Tag + Send + Sync + 'static>(&self) -> Option<&K> {
        let type_id = TypeId::of::<K>();
        self.map.get(&type_id.into())?.to_any().downcast_ref()
    }

    /// Fetch and return a typed `Tag`, removing it from the `TagMap`.
    pub fn drop_typed_tag<K: Tag + Send + Sync + 'static>(&mut self) -> Option<K> {
        let type_id = TypeId::of::<K>();
        let ret = self
            .map
            .remove(&type_id.into())?
            .to_any_boxed()
            .downcast::<K>();
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
        value: Box<dyn Any + Send + Sync + 'static>,
    ) -> Result<(), Box<dyn Tag + Send + Sync + 'static>> {
        let key = key.as_ref().into();
        let value = Box::new(UnknownItem(value));
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
    pub fn get_custom_tag(
        &self,
        key: impl AsRef<str>,
    ) -> Option<&Box<dyn Tag + Send + Sync + 'static>> {
        self.map.get(&key.as_ref().into())
    }

    /// Fetch and return an associated custom tag object, removing it from the `TagMap`.
    pub fn drop_custom_tag(
        &mut self,
        key: impl AsRef<str>,
    ) -> Option<Box<dyn Tag + Send + Sync + 'static>> {
        self.map.remove(&key.as_ref().into())
    }
}
