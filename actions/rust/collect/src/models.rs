pub struct FormattingConfig {
    pub album: String,
}

pub struct Track {
    pub discnumber: u32,
    pub tracknumber: u32,
    pub title: String,
    pub artist: Option<String>,
}

#[derive(Default)]
pub struct AlbumData {
    pub albumartist: String,
    pub album: String,
    pub date: String,
    pub tracks: Vec<Track>,
}
