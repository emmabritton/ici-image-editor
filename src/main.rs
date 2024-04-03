mod palettes;
mod scenes;
mod ui;

use crate::scenes::editor;
use crate::scenes::editor::{Editor, EditorDetails};
use crate::scenes::menu::Menu;
use crate::scenes::new_image_dialog::NewImageDialog;
use crate::scenes::palette_dialog::PaletteDialog;
use crate::scenes::save_palette_dialog::SavePaletteDataDialog;
use color_eyre::Result;
use directories::UserDirs;
use log::LevelFilter;
use pixels_graphics_lib::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;
use std::{env, fs};

#[allow(clippy::upper_case_acronyms)]
type SUR = SceneUpdateResult<SceneResult, SceneName>;

const WIDTH: usize = 280;
const HEIGHT: usize = 240;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    pub last_used_dir: PathBuf,
    pub last_used_pal_dir: PathBuf,
    pub use_colors: bool,
}

fn settings() -> AppPrefs<Settings> {
    AppPrefs::new("app", "emmabritton", "image_editor", || Settings {
        last_used_dir: UserDirs::new()
            .and_then(|ud| ud.document_dir().map(|p| p.to_path_buf()))
            .unwrap_or(PathBuf::from("/")),
        last_used_pal_dir: UserDirs::new()
            .and_then(|ud| ud.document_dir().map(|p| p.to_path_buf()))
            .unwrap_or(PathBuf::from("/")),
        use_colors: true,
    })
    .expect("Unable to create prefs file")
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

    let switcher: SceneSwitcher<SceneResult, SceneName> = |style, list, name| match name {
        SceneName::Editor(details) => {
            list.retain(|s| s.id() != editor::ID);
            list.push(Editor::new(
                WIDTH,
                HEIGHT,
                details,
                settings(),
                load_default_palette(),
                style,
            ))
        }
        SceneName::NewImage => list.push(NewImageDialog::new(WIDTH, HEIGHT, style)),
        SceneName::Palette(colors, selected) => list.push(PaletteDialog::new(
            colors,
            WIDTH,
            HEIGHT,
            selected,
            settings(),
            &style.dialog,
        )),
        SceneName::SavePaletteData(pal) => list.push(SavePaletteDataDialog::new(
            WIDTH,
            HEIGHT,
            pal,
            settings(),
            &style.alert,
            &style.dialog,
        )),
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
        Menu::new(settings(), load_default_palette(), &options.style.button),
        options,
        empty_pre_post(),
    )?;
    Ok(())
}

enum DefaultPalette {
    NoPalette,
    Error(String),
    Palette(String, Vec<Color>),
}

fn load_default_palette() -> DefaultPalette {
    if env::args().len() > 1 {
        let palette_option = env::args().nth(1).unwrap_or_default();
        return match fs::read_to_string(&palette_option) {
            Ok(data) => match JascPalette::from_file_contents(&data) {
                Ok(palette) => {
                    if palette.colors.is_empty() {
                        DefaultPalette::Error(format!("Palette {palette_option} was empty"))
                    } else {
                        DefaultPalette::Palette(palette_option, palette.colors)
                    }
                }
                Err(e) => {
                    DefaultPalette::Error(format!("Invalid palette file: {palette_option}: {e}"))
                }
            },
            Err(e) => DefaultPalette::Error(format!(
                "Unable to read palette file: {palette_option}: {e}"
            )),
        };
    }
    DefaultPalette::NoPalette
}

#[derive(Debug, Clone, PartialEq)]
enum SceneName {
    Editor(EditorDetails),
    NewImage,
    Palette(Vec<Color>, usize),
    SavePaletteData(Option<FilePalette>),
}

#[derive(Debug, Clone, PartialEq)]
enum SceneResult {
    SavePaletteData(FilePalette),
    Palette(Vec<Color>, usize),
}
