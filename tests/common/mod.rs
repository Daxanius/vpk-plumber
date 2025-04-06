use const_format::concatcp;

pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

// Directories
pub const DIR_TEST_DATA: &str = "tests/data/";
pub const DIR_V1: &str = concatcp!(DIR_TEST_DATA, "v1/");
pub const DIR_V2: &str = concatcp!(DIR_TEST_DATA, "v2/");
pub const DIR_REVPK: &str = concatcp!(DIR_TEST_DATA, "revpk/");

pub const SINGLE_FILE_ARCHIVE: &str = "single_file";
pub const EMPTY_ARCHIVE: &str = "empty";

pub const SINGLE_FILE_NAME: &str = "test/file.txt";

// Data
pub const PAK_V1_EMPTY: &str = concatcp!(DIR_V1, "empty_dir.vpk");
pub const PAK_V1_SINGLE_FILE: &str = concatcp!(DIR_V1, "single_file_dir.vpk");
pub const PAK_V1_ARCHIVE: &str = concatcp!(DIR_V1, "single_file_000.vpk");
pub const PAK_V1_SINGLE_FILE_EOF: &str = concatcp!(DIR_V1, "single_file_eof_dir.vpk");
pub const PAK_V1_PORTAL2: &str = concatcp!(DIR_V1, "portal2/pak01_dir.vpk");

pub const PAK_V2_EMPTY: &str = concatcp!(DIR_V2, "empty_dir.vpk");
pub const PAK_V2_ARCHIVE: &str = concatcp!(DIR_V2, "single_file_000.vpk");
pub const PAK_V2_SINGLE_FILE: &str = concatcp!(DIR_V2, "single_file_dir.vpk");
pub const PAK_V2_PORTAL: &str = concatcp!(DIR_V2, "portal/portal_pak_dir.vpk");

pub const PAK_REVPK_ARCHIVE: &str = concatcp!(DIR_REVPK, "single_file_000.vpk");
pub const PAK_REVPK_SINGLE_FILE: &str = concatcp!(DIR_REVPK, "single_file_dir.vpk");
pub const PAK_REVPK_TITANFALL: &str = concatcp!(
    DIR_REVPK,
    "titanfall/englishclient_mp_colony.bsp.pak000_dir.vpk"
);
