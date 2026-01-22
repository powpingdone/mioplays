use slint::Weak as SlintWeak;
use smol::prelude::*;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::PathBuf,
    sync::{Arc, LazyLock, Weak as ArcWeak},
};

slint::include_modules!();

static DEFAULT_MUSIC: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("~/Music"));
static ASYNC_RT: smol::Executor<'static> = smol::Executor::new();

struct Item {
    pub path: PathBuf,
    pub item_type: ItemType,
}

type TagMap = HashMap<String, Box<[u8]>>;
enum ItemType {
    // some ordinary file
    File,
    // an audio file that doesn't contain tags
    AudioFile,
    // a audio file with tags
    TaggedFile(TagMap),
}

#[derive(Default)]
struct Tracks(Vec<Item>);

#[derive(Default)]
struct MioPlaysState {
    pub tracks: Tracks,
}

impl MioPlaysState {
    fn new() -> Self {
        Self::default()
    }
}

impl Tracks {
    fn clone_to_multi_vec(&self, items_per: usize) -> Vec<Vec<AlbumItem>> {
        todo!()
    }

    async fn scan(&mut self) {
        // TODO: error handling
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
                                        if check_extension_for_tag_decoder(ext).await {
                                            ItemType::TaggedFile(decode_tags(item.path()).await)
                                        } else if check_extension_for_sound_decoder(ext).await {
                                            ItemType::AudioFile
                                        } else {
                                            ItemType::File
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

async fn check_extension_for_tag_decoder(inp: &OsStr) -> bool {
    // TODO: impl
    true
}

async fn check_extension_for_sound_decoder(inp: &OsStr) -> bool {
    // TODO: impl
    true
}

async fn decode_tags(inp: PathBuf) -> TagMap {
    // TODO: impl
    TagMap::default()
}

fn reload_music_files(
    w_state: ArcWeak<smol::lock::RwLock<MioPlaysState>>,
    w_mainui: SlintWeak<MainWindow>,
) {
    let Some(state_lock) = w_state.upgrade() else {
        return;
    };
    ASYNC_RT
        .spawn(async move {
            let mut state = state_lock.write().await;
            state.tracks.scan().await;
            drop(state);

            w_mainui
                .upgrade_in_event_loop(|mainui| {
                    mainui
                        .global::<MainBrowsingState>()
                        .invoke_album_view_width_changed();
                })
                .unwrap();
        })
        .detach();
}

// if the UI locks up on this function, shove it to another thread
fn width_changed(
    w_state: ArcWeak<smol::lock::RwLock<MioPlaysState>>,
    w_mainui: SlintWeak<MainWindow>,
) {
    let Some(state_lock) = w_state.upgrade() else {
        return;
    };
    let Some(mainui) = w_mainui.upgrade() else {
        return;
    };
    let state = state_lock.read_blocking();
    let main_browsing_state = mainui.global::<MainBrowsingState>();
    let multi_vec = state
        .tracks
        .clone_to_multi_vec(main_browsing_state.get_max_per_row() as usize);

    main_browsing_state.set_tracks(slint::ModelRc::new(slint::VecModel::from(
        multi_vec
            .into_iter()
            .map(|inner| slint::ModelRc::new(slint::VecModel::from(inner)))
            .collect::<Vec<_>>(),
    )));
}

fn main() {
    let state = Arc::new(smol::lock::RwLock::new(MioPlaysState::new()));
    let mainui = MainWindow::new().unwrap();
    let browse_state = mainui.global::<MainBrowsingState>();

    browse_state.on_begin_reload_all_tracks({
        let w_state = Arc::downgrade(&state);
        let w_mainui = mainui.as_weak();
        move || reload_music_files(w_state.clone(), w_mainui.clone())
    });

    browse_state.on_album_view_width_changed({
        let w_state = Arc::downgrade(&state);
        let w_mainui = mainui.as_weak();
        move || width_changed(w_state.clone(), w_mainui.clone())
    });

    drop(browse_state);

    let rt_thread = slint::spawn_local(async {
        loop {
            ASYNC_RT.tick().await;
        }
    })
    .unwrap();
    mainui.run().unwrap();
    rt_thread.abort();
}
