use std::path::PathBuf;

lazy_static::lazy_static! {
    /// Path to the flowerchat's data folder. Takes one of the following values
    /// in the corresponding priority order.
    ///
    /// - `$FLOWERCHAT_DATA_FOLDER`.
    /// - `$XDG_DATA_HOME/flowerchat`.
    /// - `$HOME/.local/share/flowerchat`.
    /// - `<current directory>/flowerchat`.
    pub static ref DATA_FOLDER: PathBuf = std::env::var("FLOWERCHAT_DATA_FOLDER")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("XDG_DATA_HOME")
                .map(|path| PathBuf::from(path).join("flowerchat"))
        })
        .or_else(|_| {
            std::env::var("HOME")
                .map(|path| {
                    PathBuf::from(path)
                        .join(".local")
                        .join("share")
                        .join("flowerchat")
                })
        })
        .map_err(std::io::Error::other)
        .and_then(|_| {
            std::env::current_dir()
                .map(|path| path.join("flowerchat"))
        })
        .expect("failed to choose the data folder path");

    /// Path to the flowerchat database file: `DATA_FOLDER/flowerchat.db`.
    pub static ref DATABASE_PATH: PathBuf = DATA_FOLDER.join("flowerchat.db");

    /// Path to the flowerchat identities file: `DATA_FOLDER/identities.json`.
    pub static ref IDENTITIES_PATH: PathBuf = DATA_FOLDER.join("identities.json");
}
