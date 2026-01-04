use std::{path::PathBuf, rc::Rc};

slint::include_modules!();

fn main() {
    let mainui = MainWindow::new().unwrap();
    let browse_state = mainui.global::<MainBrowsingState>();
    let e = Rc::new(slint::VecModel::from(vec![AlbumItem {
        album: "Album Mommy".into(),
        artist: "Artist Ipsum".into(),
        id: 0,
        title: "Title Lorem".into(),
        album_art: slint::Image::load_from_path(&PathBuf::from("../img.jpg")).unwrap(),
    }]));
    browse_state.set_tracks(slint::ModelRc::from(e));
    drop(browse_state);
    mainui.run().unwrap();
}
