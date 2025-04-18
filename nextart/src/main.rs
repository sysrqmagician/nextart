use std::{fs::DirEntry, os::unix::fs::MetadataExt, path::PathBuf};

use ::image::{ImageDecoder, ImageReader, RgbaImage};
use arboard::{Clipboard, ImageData};
use bittenhumans::ByteSizeFormatter;
use iced::{
    Alignment, Element, Font, Length, Task,
    alignment::Horizontal,
    clipboard,
    font::Weight,
    widget::{Space, button, column, image, row, scrollable, text, text_input},
};
use rfd::FileDialog;

mod strings;

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

#[derive(Debug, Clone)]
struct Collection {
    name: String,
    rom_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
enum Message {
    NoOp,
    OpenDirectoryPicker,
    OpenRomList(NextArt, Vec<usize>),
    SelectRom(usize),
    CompletedIndexing(NextArt),
    DirectoryChosen(PathBuf),
    SetupDone(PathBuf),
    SetClipboardText(String),
    SetClipboardImage(PathBuf),
    ReplacementImageFromClip(PathBuf, usize),
    ViewError(String),
    RecordError(String),
    NewImageSize(usize, u64),
    ChooseReplacementImage(PathBuf, usize),
    ResetState,
}

#[derive(Debug, Clone)]
struct NextArt {
    roms_folder: PathBuf,
    index: Index,
    errors: Vec<String>,
}

impl NextArt {
    pub fn index_roms(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(&self.roms_folder)? {
            if let Ok(entry) = entry {
                if entry.file_type()?.is_dir() {
                    self.index_collection_folder(entry)?;
                }
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
        let mut collection = Collection {
            name: collection_direntry.file_name().to_string_lossy().into(),
            rom_indices: Vec::new(),
        };

        for entry in std::fs::read_dir(&collection_direntry.path())? {
            let mut media_folder = collection_direntry.path().clone();
            media_folder.push(".media");

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
                        .ok_or(format!("Failed to extract file stem: {entry:#?}"))?
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

                if std::fs::exists(&boxart_path)? {
                    if let Ok(metadata) = std::fs::metadata(&boxart_path) {
                        rom.boxart_size = metadata.size();
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
    },
    Loading {
        state: NextArt,
        message: String,
    },
    CollectionList {
        state: NextArt,
    },
    RomList {
        state: NextArt,
        selected_index: Option<usize>,
        rom_indices: Vec<usize>,
    },
    FatalError {
        error_description: String,
    },
    ErrorList {
        state: NextArt,
    },
}

impl Default for NextArtView {
    fn default() -> Self {
        Self::Setup { chosen_path: None }
    }
}

impl NextArtView {
    pub fn view(&self) -> Element<Message> {
        match self {
            Self::Setup { chosen_path } => column![
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
                    button("Pick")
                        .padding([5, 10])
                        .on_press(Message::OpenDirectoryPicker),
                ]
                .spacing(10),
                row![
                    Space::with_width(Length::Fill),
                    button("Done")
                        .padding([10, 20])
                        .on_press(if let Some(path) = chosen_path {
                            Message::SetupDone(path.clone())
                        } else {
                            Message::ViewError(strings::ERROR_NO_PATH.into())
                        })
                ]
            ]
            .spacing(20)
            .padding(30)
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
                column(state.index.collections.iter().map(|x| {
                    row![
                        button("Open")
                            .on_press(Message::OpenRomList(state.clone(), x.rom_indices.clone())),
                        column![
                            text(x.name.clone()).font(Font {
                                weight: Weight::Bold,
                                ..Default::default()
                            }),
                            text!("{} Roms", x.rom_indices.len())
                        ],
                    ]
                    .spacing(10)
                    .into()
                }))
                .spacing(20)
                .padding(30),
            )
            .into(),

            Self::RomList {
                state,
                selected_index,
                rom_indices,
            } => {
                row![
                    scrollable(
                        column(
                            rom_indices
                                .iter()
                                .filter_map(|rom_index| {
                                    if let Some(rom) = state.index.roms.get(*rom_index) {
                                        Some((*rom_index, rom))
                                    } else {
                                        None
                                    }
                                })
                                .map(|(index, rom)| {
                                    row![
                                        button("Manage").on_press(Message::SelectRom(index)),
                                        column![
                                            text(rom.name.clone()).font(Font {
                                                weight: Weight::Bold,
                                                ..Default::default()
                                            }),
                                            if rom.boxart_size == 0 {
                                                text("No box art")
                                            } else {
                                                text!(
                                                    "{} Box Art",
                                                    ByteSizeFormatter::format_auto(
                                                        rom.boxart_size,
                                                        bittenhumans::consts::System::Binary
                                                    )
                                                )
                                            }
                                        ],
                                    ]
                                    .spacing(10)
                                    .into()
                                }),
                        )
                        .spacing(20)
                        .padding(30),
                    ),
                    if let Some(selected_index) = selected_index {
                        Self::rom_info_column(
                            state.index.roms.get(*selected_index).expect(
                                "This should not be reachable! selected_index did not exist!",
                            ),
                            *selected_index,
                        )
                    } else {
                        column![
                            text("No rom selected")
                                .width(Length::Fill)
                                .align_x(Horizontal::Center)
                        ]
                        .into()
                    }
                ]
                .padding(20)
                .into()
            }

            Self::ErrorList { state } => todo!(),

            Self::FatalError { error_description } => column![
                text(strings::UI_TITLE_ERROR).font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }),
                text(error_description).style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.palette().text.scale_alpha(0.5))
                }),
                row![
                    button("Restart").on_press(Message::ResetState),
                    Space::with_width(Length::Fill),
                    button("Copy").on_press(Message::SetClipboardText(error_description.clone()))
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

            Message::ReplacementImageFromClip(boxart_path, rom_index) => {
                return Task::perform(
                    async move {
                        let mut clip = Clipboard::new().map_err(|e| e.to_string())?;
                        let image = clip.get_image().map_err(|e| e.to_string())?;
                        let rgba_image = RgbaImage::from_vec(
                            image.width as u32,
                            image.height as u32,
                            image.bytes.to_vec(),
                        )
                        .ok_or_else(|| "Failed to create image from clipboard data".to_string())?;
                        rgba_image
                            .save_with_format(&boxart_path, ::image::ImageFormat::Png)
                            .map_err(|e| e.to_string())?;

                        std::fs::metadata(&boxart_path)
                            .map_err(|e| e.to_string())
                            .map(|m| m.size())
                    },
                    move |result| match result {
                        Ok(size) => Message::NewImageSize(rom_index, size),
                        Err(e) => Message::RecordError(e),
                    },
                );
            }

            Message::SetClipboardImage(image_path) => {
                return Task::perform(
                    async move {
                        let img = ImageReader::open(image_path)
                            .map_err(|x| x.to_string())?
                            .decode()
                            .map_err(|x| x.to_string())?;
                        let mut clip = Clipboard::new().map_err(|x| x.to_string())?;
                        let img_data = ImageData {
                            width: img.width() as usize,
                            height: img.height() as usize,
                            bytes: img.as_bytes().into(),
                        };

                        clip.set_image(img_data).map_err(|x| x.to_string())
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

            Message::NewImageSize(rom_index, size) => {
                if let NextArtView::RomList { state, .. } = self {
                    state.index.roms[rom_index].boxart_size = size;
                }
            }

            Message::ResetState => {
                *self = NextArtView::Setup { chosen_path: None };
            }

            Message::OpenRomList(state, rom_indices) => {
                *self = NextArtView::RomList {
                    state,
                    selected_index: None,
                    rom_indices,
                };
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
                                    "Failed to copy file: {}",
                                    written.unwrap_err()
                                ));
                            }
                        } else {
                            return Err("No file selected".into());
                        }
                    },
                    move |x| match x {
                        Ok(x) => Message::NewImageSize(rom_index, x),
                        Err(e) => Message::RecordError(e.to_string()),
                    },
                );
            }

            Message::OpenDirectoryPicker => {
                return Task::perform(
                    async move {
                        let dialog = FileDialog::new();
                        dialog.pick_folder()
                    },
                    |x| {
                        if let Some(x) = x {
                            Message::DirectoryChosen(x)
                        } else {
                            Message::NoOp
                        }
                    },
                );
            }

            Message::DirectoryChosen(path) => {
                if let NextArtView::Setup { chosen_path } = self {
                    *chosen_path = Some(path);
                }
            }

            Message::SelectRom(index) => {
                if let NextArtView::RomList { selected_index, .. } = self {
                    *selected_index = Some(index);
                }
            }

            Message::SetupDone(path) => {
                *self = NextArtView::Loading {
                    state: NextArt {
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
                            if let Err(e) = state.index_roms() {
                                state.errors.push(e.to_string());
                            }
                            state
                        },
                        |new_state| Message::CompletedIndexing(new_state),
                    );
                }
            }

            Message::CompletedIndexing(state) => {
                *self = NextArtView::CollectionList { state };
            }
        }

        Task::none()
    }

    fn rom_info_column(rom: &Rom, rom_index: usize) -> Element<Message> {
        column![
            text(&rom.name)
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                })
                .size(32),
            if rom.boxart_size == 0 {
                column![
                    text("No image").font(Font {
                        weight: Weight::Light,
                        ..Default::default()
                    }),
                    row![
                        button("Copy Path").on_press(Message::SetClipboardText(
                            rom.boxart_path.to_string_lossy().into()
                        )),
                        button("Choose Image").on_press(Message::ChooseReplacementImage(
                            rom.boxart_path.clone(),
                            rom_index
                        )),
                        button("Paste Image").on_press(Message::ReplacementImageFromClip(
                            rom.boxart_path.clone(),
                            rom_index
                        )),
                    ]
                    .spacing(5)
                ]
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .spacing(10)
            } else {
                column![
                    image(&rom.boxart_path),
                    row![
                        button("Copy Path").on_press(Message::SetClipboardText(
                            rom.boxart_path.to_string_lossy().into()
                        )),
                        button("Choose Image").on_press(Message::ChooseReplacementImage(
                            rom.boxart_path.clone(),
                            rom_index
                        )),
                        button("Copy Image")
                            .on_press(Message::SetClipboardImage(rom.boxart_path.clone())),
                        button("Paste Image").on_press(Message::ReplacementImageFromClip(
                            rom.boxart_path.clone(),
                            rom_index
                        )),
                    ]
                    .spacing(5)
                ]
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .spacing(10)
            }
        ]
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .into()
    }
}

#[tokio::main]
async fn main() {
    iced::application("NextArt", NextArtView::update, NextArtView::view)
        .run()
        .expect("Error while running GUI");
}
