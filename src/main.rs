use std::{path::PathBuf, rc::Rc};

slint::include_modules!();

fn main() {
    let mainui = MainWindow::new().unwrap();
    let browse_state = mainui.global::<MainBrowsingState>();
    let e = slint::ModelRc::new(slint::VecModel::from(
        vec![vec![AlbumItem {
            album: "Album Mommy".into(),
            artist: "Artist Ipsum".into(),
            id: 0,
            title: "Title Lorem".into(),
            album_art: slint::Image::load_from_path(&PathBuf::from("../img.jpg")).unwrap(),
        }]]
        .into_iter()
        .map(|inner| slint::ModelRc::new(slint::VecModel::from(inner)))
        .collect::<Vec<_>>(),
    ));

    browse_state.set_tracks(e);
    drop(browse_state);
    mainui.run().unwrap();
}
