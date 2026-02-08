use std::{path::PathBuf, sync::LazyLock};

use lofty::prelude::*;
use lofty::tag::ItemKey;
use smol::lock::Semaphore;

use crate::tag::tag_set;

pub async fn decode_tags(inp: PathBuf) -> tag_set::TagSet {
    // shared state to prevent multiple decodes at once
    //
    // POSS TODO: make this configurable by the user?
    static READING_THREADS: LazyLock<Semaphore> = LazyLock::new(|| {
        Semaphore::new(
            std::thread::available_parallelism()
                .map(|x| x.get())
                .unwrap_or_else(|_| 1), // if parallelism cannot be determined, assume we have minimum one core.
        )
    });

    // spawn worker thread
    let _lock = READING_THREADS.acquire().await;
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || tag_probe_worker_thread(inp, tx));

    // await return
    let ret = rx
        .await
        .expect("the inner thread did not return, and instead dropped the oneshot");
    drop(_lock);
    ret
}

// boilerplate macros to set fields on the tag structs
macro_rules! set_string {
    ($map:ident, $key:ty {$value:ident: $field:ident}) => {{
        // check if we can extract a string
        if let Some(value) = $value.text() {
            let value = value.to_owned();
            // try getting a mut ref to the tagmap
            let tag = $map.get_typed_tag_mut::<$key>();
            match tag {
                Some(tag) => {
                    // if exists, set field
                    tag.$field = Some(value);
                }
                None => {
                    // if not, create from default
                    //
                    // this is weird syntax for what I want, but tldr
                    // I can't just $map.push_typed_tag($key { inner });
                    // so I manually specify the type from the default, modify it,
                    // then push it back to the map
                    //
                    // the other macros use this pattern
                    let mut inp: $key = Default::default();
                    inp.$field = Some(value);
                    drop($map.push_typed_tag(inp));
                }
            }
        }
    }};
}

macro_rules! push_string {
    ($map:ident, $key:ty {$value:ident: $field:ident}) => {{
        // read set_string above for comments.
        // the only difference is that this expects to operate on a vec
        if let Some(value) = $value.text() {
            let value = value.to_owned();
            let tag = $map.get_typed_tag_mut::<$key>();
            match tag {
                Some(tag) => {
                    tag.$field.push(value);
                }
                None => {
                    let mut inp: $key = Default::default();
                    inp.$field.push(value);
                    drop($map.push_typed_tag(inp));
                }
            }
        }
    }};
}

macro_rules! set_parsed {
        ($map:ident, $key:ty {$value:ident: $field:ident}) => {{
            // check if key exists
            let check_key = $map.drop_typed_tag::<$key>();
            if check_key.is_none()
                // check if string exists and is parseable
                && let Some(text) = $value.text()
                && let Ok(parsed) = text.parse()
            {
                // set the tag
                let mut inp: $key = Default::default();
                inp.$field = parsed;
                drop($map.push_typed_tag(inp));
            }
        }};
    }

/// The worker thread for extracting the item. While this is supposted to be async,
/// because of lofty not being async itself, just use this worker thread that can
/// block instead.
fn tag_probe_worker_thread(inp: PathBuf, tx: oneshot::Sender<tag_set::TagSet>) {
    use crate::tag;

    let mut map = tag_set::TagSet::new();

    // probe the item
    let probe = lofty::probe::Probe::open(inp)
        .expect("file cannot be opened")
        .options(
            lofty::config::ParseOptions::new()
                .parsing_mode(lofty::config::ParsingMode::Relaxed)
                .max_junk_bytes(4096)
                .read_cover_art(true)
                .read_tags(true)
                .read_properties(false),
        )
        .guess_file_type()
        .expect("unable to guess file type")
        .read()
        .expect("unable to read file tags");

    for tag in probe.tags() {
        // string tags
        for item in tag.items() {
            // TODO: possibly use item.lang()
            let (key, value) = (item.key(), item.value());
            match key {
                ItemKey::AlbumTitle => set_string!(map, tag::AlbumTitle { value: inner }),
                ItemKey::AlbumTitleSortOrder => {
                    set_string!(map, tag::AlbumTitle { value: sort_order })
                }
                ItemKey::AlbumArtist => push_string!(map, tag::AlbumArtist { value: inner }),
                ItemKey::AlbumArtistSortOrder => {
                    push_string!(map, tag::AlbumArtist { value: sort_order })
                }
                ItemKey::Composer => set_string!(map, tag::Composer { value: inner }),
                ItemKey::ComposerSortOrder => {
                    set_string!(map, tag::Composer { value: sort_order })
                }
                ItemKey::DiscNumber => set_parsed!(map, tag::DiscPos { value: inner }),
                ItemKey::DiscTotal => set_parsed!(map, tag::DiscTotal { value: inner }),
                ItemKey::Performer => push_string!(map, tag::Performer { value: inner }),
                ItemKey::TrackArtist | ItemKey::TrackArtists => {
                    push_string!(map, tag::TrackArtist { value: inner })
                }
                ItemKey::TrackArtistSortOrder => {
                    push_string!(map, tag::TrackArtist { value: sort_order })
                }
                ItemKey::TrackNumber => set_parsed!(map, tag::TrackPos { value: inner }),
                ItemKey::TrackTitle => set_string!(map, tag::TrackTitle { value: inner }),
                ItemKey::TrackTitleSortOrder => {
                    set_string!(map, tag::TrackTitle { value: sort_order })
                }
                ItemKey::TrackTotal => set_parsed!(map, tag::TrackTotal { value: inner }),
                ItemKey::ReleaseDate => set_parsed!(map, tag::ReleaseDate { value: inner }),
                ItemKey::Remixer => push_string!(map, tag::Remixer { value: inner }),
                _ => continue,
            }
        }

        // cover art
        if let Some(img) = tag.get_picture_type(lofty::picture::PictureType::CoverFront)
            && map.get_typed_tag::<tag::EncodedCoverArt>().is_none()
        {
            let buf = img.data().to_owned().into();
            drop(map.push_typed_tag(tag::EncodedCoverArt(buf)));
        }
    }

    tx.send(map)
        .expect("the outer thread closed the recv, and therefore we cannot send the tags");
}
