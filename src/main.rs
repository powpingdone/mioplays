use smol::prelude::*;
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, sync::LazyLock};

slint::include_modules!();

static DEFAULT_MUSIC: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("~/Music"));

struct Item {
    path: PathBuf,
    item_type: ItemType,
}

enum ItemType {
    // some ordinary file
    File,
    // an audio file that doesn't contain tags
    AudioFile,
    // a audio file with tags
    TaggedFile(async_lazy::Lazy<HashMap<String, Box<[u8]>>>),
}

#[derive(Default)]
struct Tracks(Vec<Item>);

struct MioPlaysState<'a> {
    rt: smol::Executor<'a>,
    tracks: Tracks,
}

impl Tracks {
    fn new() -> Self {
        Self::default()
    }

    async fn rescan(&mut self) {
        async fn scan_recurse(at: PathBuf, limit: u8) -> Vec<Item> {
            if limit > 0 {
                // normal scan logic
                let mut ret = vec![];
                let mut dir = smol::fs::read_dir(at).await.unwrap();
                while let Some(item) = dir.next().await {
                    let item = item.unwrap();
                    let ftype = item.file_type().await.unwrap();
                    if ftype.is_file() {
                        // file logic
                        ret.push(Item {
                            path: item.path().clone(),
                            item_type: {
                                match item.path().extension() {
                                    Some(ext) => {
                                        if check_extension_for_tag_decoder(ext) {
                                            ItemType::TaggedFile(async_lazy::Lazy::new(|| {
                                                Box::pin({
                                                    let path = item.path().clone();
                                                    async move { todo!() }
                                                })
                                            }))
                                        } else if check_extension_for_sound_decoder(ext) {
                                            ItemType::AudioFile
                                        }
                                    }
                                    None => ItemType::File,
                                }
                            },
                        });
                    } else if ftype.is_dir() {
                        // traverse dir
                        ret.extend(Box::pin(scan_recurse(item.path(), 10)).await);
                    }
                }
                ret
            } else {
                // TODO: indicate that traversal stopped
                vec![]
            }
        }

        let input_dir = &*DEFAULT_MUSIC;
        self.0.clear();
        self.0.extend(scan_recurse(input_dir.clone(), 10).await);
    }
}

async fn check_extension_for_tag_decoder(inp: &OsStr) -> bool {}

async fn check_extension_for_sound_decoder(inp: &OsStr) -> bool {}

fn load_music_files() {}

fn main() {
    let mainui = MainWindow::new().unwrap();
    let browse_state = mainui.global::<MainBrowsingState>();
    let e = slint::ModelRc::new(slint::VecModel::from(
        Vec::<Vec<_>>::new()
            .into_iter()
            .map(|inner| slint::ModelRc::new(slint::VecModel::from(inner)))
            .collect::<Vec<_>>(),
    ));

    browse_state.set_tracks(e);
    drop(browse_state);
    mainui.run().unwrap();
}
