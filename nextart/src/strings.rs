pub const ERROR_CANNOT_NAVIGATE: &str = "Cannot navigate: No state available";
pub const ERROR_NO_FILE_SELECTED: &str = "No file selected";
pub const ERROR_NO_PATH: &str = "No path selected.";
pub const ERROR_CANNOT_NAVIGATE_COLLECTIONS: &str =
    "Cannot navigate to collections: Current view doesn't contain a valid state";
pub const ERROR_PREFIX_ACCESS_CLIPBOARD: &str = "Failed to access clipboard: ";
pub const ERROR_PREFIX_CLIPBOARD_IMAGE: &str = "Failed to get image from clipboard: ";
pub const ERROR_FAILED_CLIPBOARD_IMAGE_OTHER: &str =
    "Failed to create image from clipboard data: Invalid image dimensions or data.";
pub const ERROR_NO_HOME_DIRECTORY: &str = "Failed to retrieve home directory from operating system. Roms path will not be pre-filled on restart.";
pub const ERROR_PREFIX_CONFIG_DIR_CREATE: &str =
    "Failed to create config directory. Roms path will not be pre-filled on restart";
pub const ERROR_PREFIX_CONFIG_FILE_CREATE: &str =
    "Failed to create config file. Roms path will not be pre-filled on restart";
pub const ERROR_PREFIX_CONFIG_FILE_READ: &str = "Failed to read config file";
pub const ERROR_PREFIX_DELETE_FILE: &str = "Failed to delete file '";
pub const ERROR_PREFIX_COPY_FILE: &str = "Failed to copy file from '";
pub const ERROR_PREFIX_DECODE_IMAGE: &str = "Failed to decode image '";
pub const ERROR_PREFIX_DIR_ENTRY: &str = "Failed to read directory entry: ";
pub const ERROR_PREFIX_DIR_READ: &str = "Failed to read directory '";
pub const ERROR_PREFIX_DIR_TYPE: &str = "Failed to determine file type for '";
pub const ERROR_PREFIX_FILE_STEM: &str = "Failed to extract file stem: ";
pub const ERROR_PREFIX_GET_METADATA: &str = "Failed to get metadata for '";
pub const ERROR_PREFIX_GET_METADATA_SAVED: &str = "Failed to get metadata for saved image '";
pub const ERROR_PREFIX_GUESS_FORMAT: &str = "Failed to guess format for '";
pub const ERROR_PREFIX_INDEX_COLLECTION: &str = "Failed to index collection '";
pub const ERROR_PREFIX_MEDIA_DIR: &str = "Failed to create media folder '";
pub const ERROR_PREFIX_OPEN_IMAGE: &str = "Failed to open image '";
pub const ERROR_PREFIX_OPEN_IMAGE_FILE: &str = "Failed to open image file '";
pub const ERROR_PREFIX_READ_COLLECTION: &str = "Failed to read collection directory '";
pub const ERROR_PREFIX_SAVE_IMAGE: &str = "Failed to save image to '";
pub const ERROR_PREFIX_COPY_TO_CLIPBOARD: &str = "Failed to copy image to clipboard: ";

pub const LABEL_BACK: &str = "Back";
pub const LABEL_BOX_ART: &str = "Box Art";
pub const LABEL_CHOOSE_IMAGE: &str = "Choose Image";
pub const LABEL_COPY: &str = "Copy";
pub const LABEL_COPY_IMAGE: &str = "Copy Image";
pub const LABEL_COPY_PATH: &str = "Copy Path";
pub const LABEL_DONE: &str = "Done";
pub const LABEL_MANAGE: &str = "Manage";
pub const LABEL_NO_BOX_ART: &str = "No box art";
pub const LABEL_NO_ERRORS: &str = "All good! No errors encountered.";
pub const LABEL_NO_IMAGE: &str = "No image";
pub const LABEL_NO_ROM_SELECTED: &str = "No ROM selected";
pub const LABEL_OPEN: &str = "Open";
pub const LABEL_PASTE_IMAGE: &str = "Paste Image";
pub const LABEL_DELETE: &str = "Delete";
pub const LABEL_PICK: &str = "Pick";
pub const LABEL_RESTART: &str = "Restart";
pub const LABEL_ROMS: &str = "Roms";
pub const LABEL_SHOW_ERRORS: &str = "Show Errors";
pub const LABEL_LOADING_IMAGE: &str = "Loading image...";

pub const UI_SETUP_INDEXING: &str = "Your collection is being indexed, please be patient.";
pub const UI_SETUP_WELCOME: &str = "Welcome to NextArt, please provide the path to the Roms folder located at the root of your SD Card.";

pub const UI_TITLE_ERROR: &str = "NextArt: Error";
pub const UI_TITLE_ERRORS: &str = "Errors";
pub const UI_TITLE_LOADING: &str = "NextArt: Loading...";
pub const UI_TITLE_MAIN: &str = "NextArt";
pub const UI_TITLE_SETUP: &str = "NextArt: Setup";

pub const DIR_ORG: &str = "sysrqmagician";
pub const DIR_APP: &str = "nextart";
