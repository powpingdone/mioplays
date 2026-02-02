use std::{num::NonZeroUsize, ops::Deref, path::PathBuf, sync::LazyLock};

use lofty::{file::TaggedFileExt, tag::ItemKey};
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

    // TODO: impl
    let _lock = READING_THREADS.acquire().await;
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let mut ret = tag_set::TagSet::new();
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
            for item in tag.items() {
                // TODO: possibly use item.lang()
                let (k, v) = (item.key(), item.value());
                let tag = match k {
                    ItemKey::AlbumTitle => todo!(),
                    ItemKey::SetSubtitle => todo!(),
                    ItemKey::ShowName => todo!(),
                    ItemKey::ContentGroup => todo!(),
                    ItemKey::TrackTitle => todo!(),
                    ItemKey::TrackSubtitle => todo!(),
                    ItemKey::OriginalAlbumTitle => todo!(),
                    ItemKey::OriginalArtist => todo!(),
                    ItemKey::OriginalLyricist => todo!(),
                    ItemKey::AlbumTitleSortOrder => todo!(),
                    ItemKey::AlbumArtistSortOrder => todo!(),
                    ItemKey::TrackTitleSortOrder => todo!(),
                    ItemKey::TrackArtistSortOrder => todo!(),
                    ItemKey::ShowNameSortOrder => todo!(),
                    ItemKey::ComposerSortOrder => todo!(),
                    ItemKey::AlbumArtist => todo!(),
                    ItemKey::TrackArtist | ItemKey::TrackArtists => todo!(),
                    ItemKey::Arranger => todo!(),
                    ItemKey::Writer => todo!(),
                    ItemKey::Composer => todo!(),
                    ItemKey::Conductor => todo!(),
                    ItemKey::Director => todo!(),
                    ItemKey::Engineer => todo!(),
                    ItemKey::Lyricist => todo!(),
                    ItemKey::MixDj => todo!(),
                    ItemKey::MixEngineer => todo!(),
                    ItemKey::MusicianCredits => todo!(),
                    ItemKey::Performer => todo!(),
                    ItemKey::Producer => todo!(),
                    ItemKey::Publisher => todo!(),
                    ItemKey::Label => todo!(),
                    ItemKey::InternetRadioStationName => todo!(),
                    ItemKey::InternetRadioStationOwner => todo!(),
                    ItemKey::Remixer => todo!(),
                    ItemKey::DiscNumber => todo!(),
                    ItemKey::DiscTotal => todo!(),
                    ItemKey::TrackNumber => todo!(),
                    ItemKey::TrackTotal => todo!(),
                    ItemKey::Popularimeter => todo!(),
                    ItemKey::ParentalAdvisory => todo!(),
                    ItemKey::RecordingDate => todo!(),
                    ItemKey::Year => todo!(),
                    ItemKey::ReleaseDate => todo!(),
                    ItemKey::OriginalReleaseDate => todo!(),
                    ItemKey::Isrc => todo!(),
                    ItemKey::Barcode => todo!(),
                    ItemKey::CatalogNumber => todo!(),
                    ItemKey::Work => todo!(),
                    ItemKey::Movement => todo!(),
                    ItemKey::MovementNumber => todo!(),
                    ItemKey::MovementTotal => todo!(),
                    ItemKey::MusicBrainzRecordingId => todo!(),
                    ItemKey::MusicBrainzTrackId => todo!(),
                    ItemKey::MusicBrainzReleaseId => todo!(),
                    ItemKey::MusicBrainzReleaseGroupId => todo!(),
                    ItemKey::MusicBrainzArtistId => todo!(),
                    ItemKey::MusicBrainzReleaseArtistId => todo!(),
                    ItemKey::MusicBrainzWorkId => todo!(),
                    ItemKey::FlagCompilation => todo!(),
                    ItemKey::FlagPodcast => todo!(),
                    ItemKey::FileType => todo!(),
                    ItemKey::FileOwner => todo!(),
                    ItemKey::TaggingTime => todo!(),
                    ItemKey::Length => todo!(),
                    ItemKey::OriginalFileName => todo!(),
                    ItemKey::OriginalMediaType => todo!(),
                    ItemKey::EncodedBy => todo!(),
                    ItemKey::EncoderSoftware => todo!(),
                    ItemKey::EncoderSettings => todo!(),
                    ItemKey::EncodingTime => todo!(),
                    ItemKey::ReplayGainAlbumGain => todo!(),
                    ItemKey::ReplayGainAlbumPeak => todo!(),
                    ItemKey::ReplayGainTrackGain => todo!(),
                    ItemKey::ReplayGainTrackPeak => todo!(),
                    ItemKey::AudioFileUrl => todo!(),
                    ItemKey::AudioSourceUrl => todo!(),
                    ItemKey::CommercialInformationUrl => todo!(),
                    ItemKey::CopyrightUrl => todo!(),
                    ItemKey::TrackArtistUrl => todo!(),
                    ItemKey::RadioStationUrl => todo!(),
                    ItemKey::PaymentUrl => todo!(),
                    ItemKey::PublisherUrl => todo!(),
                    ItemKey::Genre => todo!(),
                    ItemKey::InitialKey => todo!(),
                    ItemKey::Color => todo!(),
                    ItemKey::Mood => todo!(),
                    ItemKey::Bpm => todo!(),
                    ItemKey::IntegerBpm => todo!(),
                    ItemKey::CopyrightMessage => todo!(),
                    ItemKey::License => todo!(),
                    ItemKey::PodcastDescription => todo!(),
                    ItemKey::PodcastSeriesCategory => todo!(),
                    ItemKey::PodcastUrl => todo!(),
                    ItemKey::PodcastGlobalUniqueId => todo!(),
                    ItemKey::PodcastKeywords => todo!(),
                    ItemKey::Comment => todo!(),
                    ItemKey::Description => todo!(),
                    ItemKey::Language => todo!(),
                    ItemKey::Script => todo!(),
                    ItemKey::Lyrics => todo!(),
                    ItemKey::AppleXid => todo!(),
                    ItemKey::AppleId3v2ContentGroup => todo!(),

                    ItemKey::Unknown(item) => {
                        ret.push_custom_tag(item, Box::new(v.into_binary().unwrap()));
                        continue;
                    }
                    _ => unreachable!(),
                };
                ret.push_typed_tag(tag);
            }
        }
        tx.send(ret)
            .expect("the outer thread closed the recv, and therefore we cannot send the tags");
    });

    let ret = rx
        .recv()
        .expect("the inner thread did not return, and instead dropped the oneshot");
    drop(_lock);
    ret
}
