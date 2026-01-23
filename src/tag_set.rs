use std::collections::HashSet;

pub enum Tag {
    Title(String),
}

#[repr(u64)]
pub enum TagQuery {
    Title,
}

pub struct TagSet(Vec<Tag>);

impl TagSet {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

// documentation on how this works
// It's going to be a manual implementation of a hashset where TagQueries
// map onto Tags. Something like TagQueries::Title -> Tag::Title(String).
// This is going to require a mapping function for going back and forth between
// each type, then hashing the TagQuery side to determine placement for the Tag.
// This will likely be as simple as `TagQuery as u64`.
