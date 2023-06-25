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
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prefs::WindowPreferences;

use pixels_graphics_lib::prelude::*;
use std::fmt::Debug;

#[allow(clippy::upper_case_acronyms)]
type SUR = SceneUpdateResult<SceneResult, SceneName>;

const WIDTH: usize = 280;
const HEIGHT: usize = 240;

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
            SceneName::Editor(details) => list.push(Editor::new(WIDTH, HEIGHT, details, style)),
            SceneName::NewImage => list.push(NewImageDialog::new(WIDTH, HEIGHT, style)),
            SceneName::Palette(colors) => {
                list.push(PaletteDialog::new(colors, WIDTH, HEIGHT, &style.dialog))
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
        Some(WindowPreferences::new("app", "emmabritton", "image_editor").unwrap()),
        switcher,
        Menu::new(&options.style.button),
        options,
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum SceneName {
    Editor(EditorDetails),
    NewImage,
    Palette(Vec<Color>),
    #[allow(unused)] //will be one day
    SavePaletteData,
}

#[derive(Debug, Clone, PartialEq)]
enum SceneResult {
    #[allow(unused)] //will be one day
    SavePaletteData(FilePalette),
    Palette(Vec<Color>),
}
