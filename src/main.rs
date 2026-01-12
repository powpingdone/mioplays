use smol::prelude::*;
use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

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
    TaggedFile(HashMap<String, Box<[u8]>>),
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
                        todo!()
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
