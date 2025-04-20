use std::{
    fs::{DirEntry, File},
    io::BufReader,
    path::PathBuf,
};

use ::image::{EncodableLayout, ImageReader, RgbaImage};
use arboard::{Clipboard, ImageData};
use bittenhumans::ByteSizeFormatter;
use directories::ProjectDirs;
use iced::{
    Alignment, Element, Font, Length, Task,
    alignment::Horizontal,
    clipboard,
    font::Weight,
    widget::{Space, button, column, image, row, scrollable, text, text_input},
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

mod strings;

// UI Constants
const PADDING_STANDARD: u16 = 30;
const PADDING_SMALL: u16 = 20;
const PADDING_BUTTON: [u16; 2] = [10, 20];
const PADDING_BUTTON_SMALL: [u16; 2] = [5, 10];

const SPACING_STANDARD: u16 = 20;
const SPACING_SMALL: u16 = 10;
const SPACING_TINY: u16 = 5;

const FONT_SIZE_TITLE: u16 = 32;

#[derive(Debug, Default, Clone)]
struct Index {
    roms: Vec<Rom>,
    collections: Vec<Collection>,
}

#[derive(Debug, Clone)]
struct Rom {
    name: String,
    boxart_path: PathBuf,
    boxart_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistentConfig {
    roms_path: PathBuf,
}

#[derive(Debug, Clone)]
struct Collection {
    name: String,
    rom_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
enum Message {
    NoOp,
    OpenRomDirectoryPicker,
    OpenRomList(String, Vec<usize>),
    SelectRom(usize),
    CompletedIndexing(State),
    RomDirectoryChosen(PathBuf),
    OpenCollectionList,
    OpenErrorList,
    SetupDone(PathBuf),
    SetClipboardText(String),
    SetClipboardImage(PathBuf),
    ReplacementImageFromClip(PathBuf, usize),
    ViewError(String),
    RecordError(String),
    SetRomInfoImage(u32, u32, Vec<u8>),
    WroteNewImage(usize, u64),
    ChooseReplacementImage(PathBuf, usize),
    ResetState,
}

#[derive(Debug, Clone)]
struct State {
    roms_folder: PathBuf,
    index: Index,
    errors: Vec<String>,
}

impl State {
    pub fn index_roms(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let read_dir = std::fs::read_dir(&self.roms_folder).map_err(|e| {
            format!(
                "{}{}': {}",
                strings::ERROR_PREFIX_DIR_READ,
                self.roms_folder.display(),
                e
            )
        })?;

        for entry_result in read_dir {
            if let Ok(entry) = entry_result {
                let entry_path = entry.path();
                match entry.file_type() {
                    Ok(file_type) => {
                        if file_type.is_dir() {
                            if let Err(e) = self.index_collection_folder(entry) {
                                self.errors.push(format!(
                                    "{}{}': {}",
                                    strings::ERROR_PREFIX_INDEX_COLLECTION,
                                    entry_path.display(),
                                    e
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        self.errors.push(format!(
                            "{}{}': {}",
                            strings::ERROR_PREFIX_DIR_TYPE,
                            entry.path().display(),
                            e
                        ));
                    }
                }
            } else if let Err(e) = entry_result {
                self.errors
                    .push(format!("{}{}", strings::ERROR_PREFIX_DIR_ENTRY, e));
            }
        }

        self.index.collections = self
            .index
            .collections
            .iter()
            .filter(|x| !x.rom_indices.is_empty())
            .cloned()
            .collect();

        Ok(())
    }

    fn index_collection_folder(
        &mut self,
        collection_direntry: DirEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection_path = collection_direntry.path();
        let collection_name_os = collection_direntry.file_name();
        let collection_name = collection_name_os.to_string_lossy();

        let mut collection = Collection {
            name: collection_name.to_string(),
            rom_indices: Vec::new(),
        };

        let read_dir = std::fs::read_dir(&collection_path).map_err(|e| {
            format!(
                "{}{}': {}",
                strings::ERROR_PREFIX_READ_COLLECTION,
                collection_path.display(),
                e
            )
        })?;

        for entry in read_dir {
            let mut media_folder = collection_direntry.path().clone();
            media_folder.push(".media");
            if !media_folder.exists() {
                std::fs::create_dir(&media_folder).map_err(|e| {
                    format!(
                        "{}{}': {}",
                        strings::ERROR_PREFIX_MEDIA_DIR,
                        media_folder.display(),
                        e
                    )
                })?;
            }

            if let Ok(entry) = entry {
                if !entry.file_type()?.is_file() {
                    continue;
                }

                let mut boxart_path = media_folder.clone();
                boxart_path.push(&format!(
                    "{}.png",
                    entry
                        .path()
                        .file_stem()
                        .ok_or(format!("{}{:#?}", strings::ERROR_PREFIX_FILE_STEM, entry))?
                        .display()
                ));

                let mut rom = Rom {
                    name: entry
                        .path()
                        .file_stem()
                        .ok_or(format!("Failed to extract file stem: {entry:#?}"))?
                        .to_string_lossy()
                        .into(),
                    boxart_path: boxart_path.clone(),
                    boxart_size: 0,
                };

                match std::fs::exists(&boxart_path) {
                    Ok(exists) => {
                        if exists {
                            if let Ok(metadata) = std::fs::metadata(&boxart_path) {
                                rom.boxart_size = metadata.len();
                            } else {
                                self.errors.push(format!(
                                    "{}{}'",
                                    strings::ERROR_PREFIX_GET_METADATA,
                                    boxart_path.display()
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        self.errors.push(format!(
                            "{}{}: {}'",
                            strings::ERROR_PREFIX_GET_METADATA,
                            boxart_path.display(),
                            e
                        ));
                    }
                }

                self.index.roms.push(rom);
                collection.rom_indices.push(self.index.roms.len() - 1);
            }
        }

        self.index.collections.push(collection);

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum NextArtView {
    Setup {
        chosen_path: Option<PathBuf>,
        error: Option<String>,
    },
    Loading {
        state: State,
        message: String,
    },
    CollectionList {
        state: State,
    },
    RomList {
        state: State,
        title: String,
        selected_index: Option<usize>,
        selected_image: Option<image::Handle>,
        rom_indices: Vec<usize>,
    },
    FatalError {
        error_description: String,
    },
    ErrorList {
        state: State,
    },
}

impl Default for NextArtView {
    fn default() -> Self {
        Self::Setup {
            chosen_path: None,
            error: None,
        }
    }
}

impl NextArtView {
    pub fn view(&self) -> Element<Message> {
        match self {
            Self::Setup { chosen_path, error } => column![
                text(strings::UI_TITLE_SETUP).font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
                text(strings::UI_SETUP_WELCOME),
                row![
                    text_input(
                        "Path to Roms/",
                        &chosen_path
                            .clone()
                            .map_or("".to_owned(), |x| x.to_string_lossy().to_string())
                    )
                    .width(Length::Fill),
                    button(strings::LABEL_PICK)
                        .padding(PADDING_BUTTON_SMALL)
                        .on_press(Message::OpenRomDirectoryPicker),
                ]
                .spacing(SPACING_SMALL),
                row![
                    Space::with_width(Length::Fill),
                    button(strings::LABEL_DONE)
                        .padding(PADDING_BUTTON)
                        .on_press(if let Some(path) = chosen_path {
                            Message::SetupDone(path.clone())
                        } else {
                            Message::ViewError(strings::ERROR_NO_PATH.into())
                        })
                ],
                if let Some(error) = error {
                    text(error)
                } else {
                    text("")
                }
            ]
            .spacing(SPACING_STANDARD)
            .padding(PADDING_STANDARD)
            .into(),

            Self::Loading { message, .. } => column![
                text(strings::UI_TITLE_LOADING).font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
                text(message),
            ]
            .spacing(20)
            .padding(30)
            .into(),

            Self::CollectionList { state } => scrollable(
                column![
                    text(strings::UI_TITLE_MAIN)
                        .font(Font {
                            weight: Weight::Light,
                            ..Default::default()
                        })
                        .size(FONT_SIZE_TITLE)
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    column(state.index.collections.iter().map(|x| {
                        row![
                            button(strings::LABEL_OPEN).on_press(Message::OpenRomList(
                                x.name.clone(),
                                x.rom_indices.clone()
                            )),
                            column![
                                text(x.name.clone()).font(Font {
                                    weight: Weight::Bold,
                                    ..Default::default()
                                }),
                                text!("{} {}", x.rom_indices.len(), strings::LABEL_ROMS)
                            ],
                        ]
                        .spacing(SPACING_SMALL)
                        .into()
                    }))
                    .spacing(SPACING_STANDARD)
                    .padding(PADDING_STANDARD),
                    if state.errors.len() != 0 {
                        Element::from(
                            button(strings::LABEL_SHOW_ERRORS)
                                .on_press(Message::OpenErrorList)
                                .style(|theme: &iced::Theme, status| button::Style {
                                    background: if let button::Status::Hovered = status {
                                        Some(iced::Background::Color(
                                            theme.extended_palette().danger.strong.color,
                                        ))
                                    } else {
                                        Some(iced::Background::Color(
                                            theme.extended_palette().danger.base.color,
                                        ))
                                    },
                                    ..Default::default()
                                }),
                        )
                    } else {
                        text(strings::LABEL_NO_ERRORS)
                            .font(Font {
                                weight: Weight::Light,
                                ..Default::default()
                            })
                            .into()
                    }
                ]
                .padding(30),
            )
            .into(),

            Self::RomList {
                state,
                title,
                selected_index,
                selected_image,
                rom_indices,
            } => {
                let mut rom_indice_tuples: Vec<(usize, &Rom)> = rom_indices
                    .iter()
                    .filter_map(|rom_index| {
                        if let Some(rom) = state.index.roms.get(*rom_index) {
                            Some((*rom_index, rom))
                        } else {
                            None
                        }
                    })
                    .collect();
                rom_indice_tuples.sort_by_key(|x| &x.1.name);

                column![
                    row![
                        button(strings::LABEL_BACK).on_press(Message::OpenCollectionList),
                        text(title)
                            .font(Font {
                                weight: Weight::Light,
                                ..Default::default()
                            })
                            .size(32)
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                    ],
                    row![
                        scrollable(
                            column(rom_indice_tuples.iter().map(|(index, rom)| {
                                row![
                                    button(strings::LABEL_MANAGE)
                                        .on_press(Message::SelectRom(*index)),
                                    column![
                                        text(rom.name.clone()).font(Font {
                                            weight: Weight::Bold,
                                            ..Default::default()
                                        }),
                                        if rom.boxart_size == 0 {
                                            text(strings::LABEL_NO_BOX_ART)
                                        } else {
                                            text!(
                                                "{} {}",
                                                ByteSizeFormatter::format_auto(
                                                    rom.boxart_size,
                                                    bittenhumans::consts::System::Binary
                                                ),
                                                strings::LABEL_BOX_ART
                                            )
                                        }
                                    ],
                                ]
                                .spacing(SPACING_SMALL)
                                .into()
                            }),)
                            .spacing(SPACING_STANDARD)
                            .padding(PADDING_STANDARD),
                        ),
                        if let Some(selected_index) = selected_index {
                            Self::rom_info_column(
                                state.index.roms.get(*selected_index).expect(
                                    "This should not be reachable! selected_index did not exist!",
                                ),
                                *selected_index,
                                selected_image,
                            )
                        } else {
                            column![
                                text(strings::LABEL_NO_ROM_SELECTED)
                                    .width(Length::Fill)
                                    .align_x(Horizontal::Center)
                            ]
                            .into()
                        }
                    ]
                    .padding(PADDING_SMALL)
                ]
                .spacing(20)
                .padding(30)
                .into()
            }

            Self::ErrorList { state } => column![
                row![
                    button(strings::LABEL_BACK).on_press(Message::OpenCollectionList),
                    text(strings::UI_TITLE_ERRORS)
                        .size(32)
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                ]
                .spacing(10),
                scrollable(column(state.errors.iter().map(|x| {
                    row![
                        button(strings::LABEL_COPY).on_press(Message::SetClipboardText(x.clone())),
                        text(x)
                    ]
                    .spacing(10)
                    .into()
                })))
            ]
            .spacing(20)
            .padding(30)
            .into(),

            Self::FatalError { error_description } => column![
                text(strings::UI_TITLE_ERROR).font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
                text(error_description).style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.palette().text.scale_alpha(0.5))
                }),
                row![
                    button(strings::LABEL_RESTART).on_press(Message::ResetState),
                    Space::with_width(Length::Fill),
                    button(strings::LABEL_COPY)
                        .on_press(Message::SetClipboardText(error_description.clone()))
                ]
            ]
            .spacing(20)
            .padding(30)
            .into(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => {}

            Message::SetRomInfoImage(width, height, byte_vec) => {
                if let NextArtView::RomList { selected_image, .. } = self {
                    *selected_image = Some(image::Handle::from_rgba(width, height, byte_vec));
                }
            }

            Message::OpenCollectionList => match std::mem::replace(self, NextArtView::default()) {
                NextArtView::RomList { state, .. } | NextArtView::ErrorList { state } => {
                    *self = NextArtView::CollectionList { state };
                }
                other => {
                    *self = other;
                    return Task::perform(
                        async { String::from(strings::ERROR_CANNOT_NAVIGATE) },
                        Message::RecordError,
                    );
                }
            },

            Message::OpenErrorList => match std::mem::replace(self, NextArtView::default()) {
                NextArtView::RomList { state, .. } | NextArtView::CollectionList { state } => {
                    *self = NextArtView::ErrorList { state };
                }
                other => {
                    *self = other;
                    return Task::perform(
                        async { String::from(strings::ERROR_CANNOT_NAVIGATE_COLLECTIONS) },
                        Message::RecordError,
                    );
                }
            },

            Message::ReplacementImageFromClip(boxart_path, rom_index) => {
                return Task::perform(
                    async move {
                        let mut clip = Clipboard::new().map_err(|e| {
                            format!("{}{}", strings::ERROR_PREFIX_ACCESS_CLIPBOARD, e)
                        })?;
                        let image = clip.get_image().map_err(|e| {
                            format!("{}{}", strings::ERROR_PREFIX_CLIPBOARD_IMAGE, e)
                        })?;
                        let rgba_image = RgbaImage::from_vec(
                            image.width as u32,
                            image.height as u32,
                            image.bytes.to_vec(),
                        )
                        .ok_or_else(|| strings::ERROR_FAILED_CLIPBOARD_IMAGE_OTHER)?;
                        rgba_image
                            .save_with_format(&boxart_path, ::image::ImageFormat::Png)
                            .map_err(|e| {
                                format!(
                                    "{}{}': {}",
                                    strings::ERROR_PREFIX_SAVE_IMAGE,
                                    boxart_path.display(),
                                    e
                                )
                            })?;

                        std::fs::metadata(&boxart_path)
                            .map_err(|e| {
                                format!(
                                    "{}{}': {}",
                                    strings::ERROR_PREFIX_GET_METADATA_SAVED,
                                    boxart_path.display(),
                                    e
                                )
                            })
                            .map(|m| m.len())
                    },
                    move |result| match result {
                        Ok(size) => Message::WroteNewImage(rom_index, size),
                        Err(e) => Message::RecordError(e),
                    },
                );
            }

            Message::SetClipboardImage(image_path) => {
                return Task::perform(
                    async move {
                        let img = ImageReader::open(&image_path)
                            .map_err(|x| {
                                format!(
                                    "{}{}': {}",
                                    strings::ERROR_PREFIX_OPEN_IMAGE,
                                    image_path.display(),
                                    x
                                )
                            })?
                            .decode()
                            .map_err(|x| {
                                format!("Failed to decode image '{}': {}", image_path.display(), x)
                            })?
                            .into_rgba8();
                        let mut clip = Clipboard::new().map_err(|x| {
                            format!("{}{}", strings::ERROR_PREFIX_ACCESS_CLIPBOARD, x)
                        })?;
                        let img_data = ImageData {
                            width: img.width() as usize,
                            height: img.height() as usize,
                            bytes: img.as_bytes().into(),
                        };

                        clip.set_image(img_data).map_err(|x| {
                            format!("{}{}", strings::ERROR_PREFIX_COPY_TO_CLIPBOARD, x)
                        })
                    },
                    |x: Result<(), String>| {
                        if let Err(e) = x {
                            Message::RecordError(e)
                        } else {
                            Message::NoOp
                        }
                    },
                );
            }

            Message::WroteNewImage(rom_index, size) => {
                if let NextArtView::RomList {
                    state,
                    selected_image,
                    ..
                } = self
                {
                    state.index.roms[rom_index].boxart_size = size;
                    *selected_image = None;

                    return Self::load_image_task(state.index.roms[rom_index].boxart_path.clone());
                }
            }

            Message::ResetState => {
                *self = NextArtView::Setup {
                    chosen_path: None,
                    error: None,
                };
            }

            Message::OpenRomList(title, rom_indices) => {
                match std::mem::replace(self, NextArtView::default()) {
                    NextArtView::CollectionList { state } | NextArtView::ErrorList { state } => {
                        *self = NextArtView::RomList {
                            state,
                            title,
                            selected_index: None,
                            selected_image: None,
                            rom_indices,
                        };
                    }
                    other => {
                        *self = other;
                        return Task::perform(
                            async { String::from("Cannot navigate: No state available") },
                            Message::RecordError,
                        );
                    }
                }
            }

            Message::RecordError(error_description) => {
                if let NextArtView::RomList { state, .. } = self {
                    state.errors.push(error_description);
                } else if let NextArtView::Loading { state, .. } = self {
                    state.errors.push(error_description);
                } else if let NextArtView::CollectionList { state } = self {
                    state.errors.push(error_description);
                }
            }

            Message::ViewError(error_description) => {
                *self = NextArtView::FatalError { error_description };
            }

            Message::SetClipboardText(value) => {
                return clipboard::write(value);
            }

            Message::ChooseReplacementImage(path, rom_index) => {
                return Task::perform(
                    async move {
                        let dialog = FileDialog::new().add_filter("PNG", &["png"]);
                        if let Some(picked) = dialog.pick_file() {
                            let written = std::fs::copy(&picked, &path);
                            if let Ok(written) = written {
                                return Ok(written);
                            } else {
                                return Err(format!(
                                    "{}{}' to '{}': {}",
                                    strings::ERROR_PREFIX_COPY_FILE,
                                    picked.display(),
                                    path.display(),
                                    written.unwrap_err()
                                ));
                            }
                        } else {
                            return Err(strings::ERROR_NO_FILE_SELECTED.into());
                        }
                    },
                    move |x| match x {
                        Ok(x) => Message::WroteNewImage(rom_index, x),
                        Err(e) => Message::RecordError(e.to_string()),
                    },
                );
            }

            Message::OpenRomDirectoryPicker => {
                return Task::perform(
                    async move {
                        let dialog = FileDialog::new();
                        dialog.pick_folder()
                    },
                    |x| {
                        if let Some(x) = x {
                            Message::RomDirectoryChosen(x)
                        } else {
                            Message::NoOp
                        }
                    },
                );
            }

            Message::RomDirectoryChosen(path) => {
                if let NextArtView::Setup { chosen_path, .. } = self {
                    *chosen_path = Some(path);
                }
            }

            Message::SelectRom(index) => {
                if let NextArtView::RomList {
                    selected_index,
                    state,
                    ..
                } = self
                {
                    *selected_index = Some(index);

                    if state.index.roms[index].boxart_size != 0 {
                        return Self::load_image_task(state.index.roms[index].boxart_path.clone());
                    }
                }
            }

            Message::SetupDone(path) => {
                *self = NextArtView::Loading {
                    state: State {
                        roms_folder: path,
                        errors: Vec::new(),
                        index: Index::default(),
                    },
                    message: strings::UI_SETUP_INDEXING.into(),
                };
                if let Self::Loading { state, .. } = self {
                    let mut state = state.clone();
                    return Task::perform(
                        async move {
                            if let Some(dirs) =
                                ProjectDirs::from("", strings::DIR_ORG, strings::DIR_APP)
                            {
                                let config_dir = dirs.config_dir();

                                if let Err(e) = std::fs::create_dir_all(config_dir) {
                                    state.errors.push(format!(
                                        "{}: {}",
                                        strings::ERROR_PREFIX_CONFIG_DIR_CREATE,
                                        e
                                    ));
                                } else {
                                    let config = PersistentConfig {
                                        roms_path: state.roms_folder.clone(),
                                    };

                                    let config_path = config_dir.join("config.json");

                                    if let Err(e) = serde_json::to_string(&config)
                                        .map_err(|e| {
                                            format!(
                                                "{}: {}",
                                                strings::ERROR_PREFIX_CONFIG_FILE_CREATE,
                                                e
                                            )
                                        })
                                        .and_then(|serialized| {
                                            std::fs::write(&config_path, serialized).map_err(|e| {
                                                format!(
                                                    "{}: {}",
                                                    strings::ERROR_PREFIX_CONFIG_FILE_CREATE,
                                                    e
                                                )
                                            })
                                        })
                                    {
                                        state.errors.push(e);
                                    }
                                }
                            } else {
                                state.errors.push(strings::ERROR_NO_HOME_DIRECTORY.into());
                            }

                            // Index ROMs
                            if let Err(e) = state.index_roms() {
                                state.errors.push(e.to_string());
                            }

                            state
                        },
                        Message::CompletedIndexing,
                    );
                }
            }

            Message::CompletedIndexing(state) => {
                *self = NextArtView::CollectionList { state };
            }
        }

        Task::none()
    }

    fn rom_info_column<'a>(
        rom: &'a Rom,
        rom_index: usize,
        rom_image: &'a Option<image::Handle>,
    ) -> Element<'a, Message> {
        scrollable(
            column![
                text(&rom.name)
                    .font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    })
                    .size((FONT_SIZE_TITLE as f32 / 1.5).ceil()),
                if rom.boxart_size == 0 {
                    column![
                        text(strings::LABEL_NO_IMAGE).font(Font {
                            weight: Weight::Light,
                            ..Default::default()
                        }),
                        row![
                            button(strings::LABEL_COPY_PATH).on_press(Message::SetClipboardText(
                                rom.boxart_path.to_string_lossy().into()
                            )),
                            button(strings::LABEL_CHOOSE_IMAGE).on_press(
                                Message::ChooseReplacementImage(rom.boxart_path.clone(), rom_index)
                            ),
                            button(strings::LABEL_PASTE_IMAGE).on_press(
                                Message::ReplacementImageFromClip(
                                    rom.boxart_path.clone(),
                                    rom_index
                                )
                            ),
                        ]
                        .spacing(SPACING_TINY)
                    ]
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .spacing(10)
                } else {
                    column![
                        if let Some(handle) = rom_image {
                            Element::from(image(handle))
                        } else {
                            text(strings::LABEL_LOADING_IMAGE).into()
                        },
                        row![
                            button(strings::LABEL_COPY_PATH).on_press(Message::SetClipboardText(
                                rom.boxart_path.to_string_lossy().into()
                            )),
                            button(strings::LABEL_CHOOSE_IMAGE).on_press(
                                Message::ChooseReplacementImage(rom.boxart_path.clone(), rom_index)
                            ),
                            button(strings::LABEL_COPY_IMAGE)
                                .on_press(Message::SetClipboardImage(rom.boxart_path.clone())),
                            button(strings::LABEL_PASTE_IMAGE).on_press(
                                Message::ReplacementImageFromClip(
                                    rom.boxart_path.clone(),
                                    rom_index
                                )
                            ),
                        ]
                        .spacing(5)
                    ]
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .spacing(10)
                }
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill),
        )
        .into()
    }

    fn load_image_task(image_path: PathBuf) -> Task<Message> {
        Task::perform(
            async move {
                let file = File::open(&image_path).map_err(|e| {
                    format!(
                        "{}{}': {}",
                        strings::ERROR_PREFIX_OPEN_IMAGE_FILE,
                        image_path.display(),
                        e
                    )
                })?;

                let img = ImageReader::new(BufReader::new(file))
                    .with_guessed_format()
                    .map_err(|e| {
                        format!(
                            "{}{}': {}",
                            strings::ERROR_PREFIX_GUESS_FORMAT,
                            image_path.display(),
                            e
                        )
                    })?
                    .decode()
                    .map_err(|e| {
                        format!(
                            "{}{}': {}",
                            strings::ERROR_PREFIX_DECODE_IMAGE,
                            image_path.display(),
                            e
                        )
                    })?;

                Ok((img.width(), img.height(), img.to_rgba8().to_vec()))
            },
            |result: Result<(u32, u32, Vec<u8>), String>| match result {
                Ok((width, height, bytes)) => Message::SetRomInfoImage(width, height, bytes),
                Err(e) => Message::RecordError(e),
            },
        )
    }
}

#[tokio::main]
async fn main() {
    iced::application("NextArt", NextArtView::update, NextArtView::view)
        .run_with(
            || match ProjectDirs::from("", strings::DIR_ORG, strings::DIR_APP) {
                Some(dirs) => {
                    let mut config_file = dirs.config_dir().to_path_buf();
                    config_file.push("config.json");

                    match std::fs::read_to_string(&config_file) {
                        Ok(content) => match serde_json::from_str::<PersistentConfig>(&content) {
                            Ok(config) => (
                                NextArtView::Setup {
                                    chosen_path: Some(config.roms_path),
                                    error: None,
                                },
                                Task::none(),
                            ),
                            Err(e) => (
                                NextArtView::Setup {
                                    chosen_path: None,
                                    error: Some(format!(
                                        "{}: {}",
                                        strings::ERROR_PREFIX_CONFIG_FILE_READ,
                                        e
                                    )),
                                },
                                Task::none(),
                            ),
                        },
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (
                            NextArtView::Setup {
                                chosen_path: None,
                                error: None,
                            },
                            Task::none(),
                        ),
                        Err(e) => (
                            NextArtView::Setup {
                                chosen_path: None,
                                error: Some(format!(
                                    "{}: {}",
                                    strings::ERROR_PREFIX_CONFIG_FILE_READ,
                                    e
                                )),
                            },
                            Task::none(),
                        ),
                    }
                }
                None => (
                    NextArtView::Setup {
                        chosen_path: None,
                        error: Some(strings::ERROR_NO_HOME_DIRECTORY.into()),
                    },
                    Task::none(),
                ),
            },
        )
        .expect("Error while running GUI");
}
