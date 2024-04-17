use crate::scenes::editor::BackgroundColors;
use crate::scenes::editor_ui::MenuId::*;
use crate::{Settings, HEIGHT, WIDTH};
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::prelude::*;

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuId {
    MenuFile,
    MenuEdit,
    MenuImage,
    MenuPalette,
    MenuFileNew,
    MenuFileOpen,
    MenuFileSave,
    MenuFileSaveAs,
    MenuFileQuit,
    MenuEditUndo,
    MenuEditRedo,
    MenuImageFlipV,
    MenuImageFlipH,
    MenuImageClear,
    MenuImageRotCw,
    MenuImageRotCcw,
    MenuImageRotCw90,
    MenuImageRotCw180,
    MenuImageRotCw270,
    MenuImageRotCcw90,
    MenuImageRotCcw180,
    MenuImageRotCcw270,
    MenuImageShift,
    MenuImageShiftLeft,
    MenuImageShiftUp,
    MenuImageShiftRight,
    MenuImageShiftDown,
    MenuPaletteEdit,
    MenuPaletteMode,
    MenuCanvas,
    MenuCanvasResize,
    MenuCanvasTrim,
    MenuCanvasBackground,
    MenuCanvasBackgroundGreyCheck,
    MenuCanvasBackgroundPurpleCheck,
    MenuCanvasBackgroundSolidLightGrey,
    MenuCanvasBackgroundSolidDarkGrey,
    MenuCanvasBackgroundSolidBlack,
    MenuCanvasBackgroundSolidWhite,
    MenuImageDoubleSize,
    MenuFileExport,
    MenuFileExportPng,
    MenuFileExportJpeg,
    MenuFileExportTga,
    MenuFileExportBmp,
    MenuFileExportIco,
    MenuFileImport,
    MenuPaletteSimplify,
}

pub(super) fn create_menubar(style: &UiStyle, settings: &AppPrefs<Settings>) -> MenuBar<MenuId> {
    MenuBar::new(
        &style.menu,
        Coord::default(),
        (WIDTH, HEIGHT),
        true,
        &[
            MenuBarItem::new(
                MenuFile,
                "File",
                vec![
                    MenuBarItem::new_button(MenuFileNew, "New"),
                    MenuBarItem::new_button(MenuFileOpen, "Open"),
                    MenuBarItem::new_button(MenuFileSave, "Save"),
                    MenuBarItem::new_button(MenuFileSaveAs, "Save As"),
                    MenuBarItem::new_button(MenuFileImport, "Import"),
                    MenuBarItem::new_menu(
                        MenuFileExport,
                        "Export",
                        &[
                            (MenuFileExportPng, "PNG"),
                            (MenuFileExportJpeg, "JPEG"),
                            (MenuFileExportTga, "TGA"),
                            (MenuFileExportBmp, "BMP"),
                            (MenuFileExportIco, "Icon"),
                        ],
                    ),
                    MenuBarItem::new_button(MenuFileQuit, "Quit"),
                ],
            ),
            MenuBarItem::new_menu(
                MenuEdit,
                "Edit",
                &[(MenuEditUndo, "Undo"), (MenuEditRedo, "Redo")],
            ),
            MenuBarItem::new(
                MenuCanvas,
                "Canvas",
                vec![
                    MenuBarItem::new_button(MenuCanvasResize, "Resize"),
                    MenuBarItem::new_button(MenuCanvasTrim, "Trim"),
                    MenuBarItem::new_options(
                        MenuCanvasBackground,
                        "Background",
                        &BackgroundColors::menu_items(),
                        settings.data.background_color.idx(),
                    ),
                ],
            ),
            MenuBarItem::new(
                MenuImage,
                "Image",
                vec![
                    MenuBarItem::new_button(MenuImageFlipH, "Flip H"),
                    MenuBarItem::new_button(MenuImageFlipV, "Flip V"),
                    MenuBarItem::new_menu(
                        MenuImageRotCw,
                        "Rotate CW",
                        &[
                            (MenuImageRotCw90, "90°"),
                            (MenuImageRotCw180, "180°"),
                            (MenuImageRotCw270, "270°"),
                        ],
                    ),
                    MenuBarItem::new_menu(
                        MenuImageRotCcw,
                        "Rotate CCW",
                        &[
                            (MenuImageRotCcw90, "90°"),
                            (MenuImageRotCcw180, "180°"),
                            (MenuImageRotCcw270, "270°"),
                        ],
                    ),
                    MenuBarItem::new_menu(
                        MenuImageShift,
                        "Shift",
                        &[
                            (MenuImageShiftUp, "Up"),
                            (MenuImageShiftDown, "Down"),
                            (MenuImageShiftLeft, "Left"),
                            (MenuImageShiftRight, "Right"),
                        ],
                    ),
                    MenuBarItem::new_button(MenuImageDoubleSize, "Double size"),
                    MenuBarItem::new_button(MenuImageClear, "Clear"),
                ],
            ),
            MenuBarItem::new_menu(
                MenuPalette,
                "Palette",
                &[
                    (MenuPaletteEdit, "Edit"),
                    (MenuPaletteMode, "Set mode"),
                    (MenuPaletteSimplify, "Simplify"),
                ],
            ),
        ],
    )
}
