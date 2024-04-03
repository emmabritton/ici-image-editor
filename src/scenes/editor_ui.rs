use crate::scenes::editor_ui::MenuId::*;
use crate::{HEIGHT, WIDTH};
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
}

pub(super) fn create_menubar(style: &UiStyle) -> MenuBar<MenuId> {
    MenuBar::new(
        &style.menu,
        Coord::default(),
        (WIDTH, HEIGHT),
        true,
        &[
            MenuBarItem::new_menu(
                MenuFile,
                "File",
                &[
                    (MenuFileNew, "New"),
                    (MenuFileOpen, "Open"),
                    (MenuFileSave, "Save"),
                    (MenuFileSaveAs, "Save As"),
                    (MenuFileQuit, "Quit"),
                ],
            ),
            MenuBarItem::new_menu(
                MenuEdit,
                "Edit",
                &[(MenuEditUndo, "Undo"), (MenuEditRedo, "Redo")],
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
                    MenuBarItem::new_button(MenuImageClear, "Clear"),
                ],
            ),
            MenuBarItem::new(
                MenuPalette,
                "Palette",
                vec![
                    MenuBarItem::new_button(MenuPaletteEdit, "Edit"),
                    MenuBarItem::new_button(MenuPaletteMode, "Set mode"),
                ],
            ),
        ],
    )
}
