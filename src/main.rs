use slint::{Model, Weak as SlintWeak};
use smol::prelude::*;
use std::{
    path::PathBuf,
    sync::{Arc, LazyLock, Weak as ArcWeak},
};

slint::include_modules!();

mod tag;

static DEFAULT_MUSIC: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("~/Music"));
static ASYNC_RT: smol::Executor<'static> = smol::Executor::new();

#[derive(Debug)]
struct AudioField;

#[derive(Debug)]
struct Item {
    pub path: PathBuf,
    pub audio: Option<AudioField>,
    pub tags: Option<tag::TagSet>,
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
        let mut ret = vec![];
        let mut chunk = vec![];
        for (e, _x) in self.0.iter().enumerate() {
            let x = AlbumItem {
                album: "ALBUM".into(),
                artist: "ARTIST".into(),
                id: e.try_into().unwrap(),
                title: "TITLE".into(),
                album_art: Default::default(),
            };
            chunk.push(x);
            if chunk.len() >= items_per {
                ret.push(std::mem::take(&mut chunk));
            }
        }
        ret
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
                        let path = item.path().clone();
                        let ext = path.extension().and_then(|x| x.to_str());
                        ret.push(Item {
                            path: path.clone(),
                            audio: if let Some(ext) = &ext
                                && check_extension_for_sound_decoder(ext).await
                            {
                                Some(AudioField)
                            } else {
                                None
                            },
                            tags: if let Some(ext) = &ext
                                && check_extension_for_tag_decoder(ext).await
                            {
                                Some(tag::decode_tags(path).await)
                            } else {
                                None
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

async fn check_extension_for_tag_decoder(inp: &str) -> bool {
    // TODO: impl
    true
}

async fn check_extension_for_sound_decoder(inp: &str) -> bool {
    // TODO: impl
    true
}

fn reload_music_files(
    w_state: ArcWeak<smol::lock::RwLock<MioPlaysState>>,
    w_mainui: SlintWeak<MainWindow>,
) {
    // spawn an async task
    let Some(state_lock) = w_state.upgrade() else {
        return;
    };
    ASYNC_RT
        .spawn(async move {
            // reset track list
            let mut state = state_lock.write().await;
            state.tracks.scan().await;
            drop(state);

            w_mainui
                .upgrade_in_event_loop(|mainui| {
                    // then reload the grid
                    mainui
                        .global::<MainBrowsingState>()
                        .invoke_album_view_width_changed();
                })
                .unwrap();
        })
        .detach();
}

// NOTE: the UI can stall on this function, shove it to another thread (if needed)
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
    let main_browsing_state = mainui.global::<MainBrowsingState>();

    // check if row_count is still the same as the requested max-per-row
    let vec_tracks = main_browsing_state.get_tracks();
    let Some(inner) = vec_tracks.iter().nth(0) else {
        // fail on empty vec
        return;
    };
    // there is a single case where this will always result in a refresh:
    // where the total songs is less than the max-per-row.
    // anyways, fast path.
    if inner.row_count() == main_browsing_state.get_max_per_row() as usize {
        return;
    }

    // construct the multidim vec
    let state = state_lock.read_blocking();
    let multi_vec = state
        .tracks
        .clone_to_multi_vec(main_browsing_state.get_max_per_row() as usize);

    // then set the track vec
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
