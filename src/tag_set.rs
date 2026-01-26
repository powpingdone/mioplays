use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub trait TagInner {}

pub trait Tag: TagInner + Any {}

pub struct Title {
    title: String,
}

impl TagInner for Title {}

pub struct TagSet {
    known: HashMap<TypeId, Box<dyn Any + 'static>>,
}

impl TagSet {
    pub fn new() -> Self {
        Self {
            known: HashMap::new(),
        }
    }

    // Add tag to set. If the associated tag is already in the set,
    // return the argument as an error.
    pub fn push_typed_tag<K: Tag + 'static>(&mut self, tag: K) -> Result<(), K> {
        if self.known.contains_key(&tag.type_id()) {
            Err(tag)
        } else {
            let ret = self.known.insert(tag.type_id(), Box::new(tag));
            if ret.is_some() {
                panic!("somehow, we don't contain a key we have access to now")
            }
            Ok(())
        }
    }

    pub fn get_typed_tag<K: Any>(&self) -> Option<&K> {
        let type_id = TypeId::of::<K>();
        self.known.get(&type_id)?.downcast_ref()
    }
}
