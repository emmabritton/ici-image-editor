mod palettes;
mod scenes;
mod ui;

use crate::scenes::menu::Menu;
use crate::scenes::new_editor::{Editor, EditorDetails};
use crate::scenes::new_image_dialog::NewImageDialog;
use crate::scenes::palette_dialog::PaletteDialog;
use crate::scenes::save_palette_dialog::SavePaletteDataDialog;
use color_eyre::Result;
use log::LevelFilter;
use pixels_graphics_lib::prelude::*;
use std::fmt::Debug;
use std::path::PathBuf;
use directories::UserDirs;
use serde::{Deserialize, Serialize};

#[allow(clippy::upper_case_acronyms)]
type SUR = SceneUpdateResult<SceneResult, SceneName>;

const WIDTH: usize = 280;
const HEIGHT: usize = 240;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    pub last_used_dir: PathBuf,
    pub last_used_pal_dir: PathBuf,
}

fn settings() -> AppPrefs<Settings> {
    AppPrefs::new("app", "emmabritton", "image_editor", || Settings {
        last_used_dir: UserDirs::new()
            .and_then(|ud| ud.document_dir().map(|p| p.to_path_buf()))
            .unwrap_or(PathBuf::from("/")),
        last_used_pal_dir: UserDirs::new()
            .and_then(|ud| ud.document_dir().map(|p| p.to_path_buf()))
            .unwrap_or(PathBuf::from("/")),
    }).expect("Unable to create prefs file")
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .format_level(false)
        .format_timestamp(None)
        .format_module_path(false)
        .filter_level(LevelFilter::Warn)
        .filter_module("image_editor", LevelFilter::Debug)
        .init();
    color_eyre::install()?;

    let switcher: SceneSwitcher<SceneResult, SceneName> = |style, list, name| {
        let style = style;
        match name {
            SceneName::Editor(details) => list.push(Editor::new(WIDTH, HEIGHT, details, settings(), style)),
            SceneName::NewImage => list.push(NewImageDialog::new(WIDTH, HEIGHT, style)),
            SceneName::Palette(colors, selected) => {
                list.push(PaletteDialog::new(colors, WIDTH, HEIGHT, selected, settings(), &style.dialog))
            }
            SceneName::SavePaletteData => list.push(SavePaletteDataDialog::new(
                WIDTH,
                HEIGHT,
                &style.alert,
                &style.dialog,
            )),
        }
    };

    let mut options = Options::default();
    options.style.dialog.bounds =
        Rect::new_with_size((42, 36), MIN_FILE_DIALOG_SIZE.0, MIN_FILE_DIALOG_SIZE.1);
    run_scenes(
        WIDTH,
        HEIGHT,
        "Image Editor",
        Some(WindowPreferences::new("app", "emmabritton", "image_editor", 1).unwrap()),
        switcher,
        Menu::new(settings(), &options.style.button),
        options,
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum SceneName {
    Editor(EditorDetails),
    NewImage,
    Palette(Vec<Color>, usize),
    #[allow(unused)] //will be one day
    SavePaletteData,
}

#[derive(Debug, Clone, PartialEq)]
enum SceneResult {
    #[allow(unused)] //will be one day
    SavePaletteData(FilePalette),
    Palette(Vec<Color>, usize),
}
