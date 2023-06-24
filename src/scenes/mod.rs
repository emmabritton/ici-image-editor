use directories::UserDirs;
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use rfd::FileDialog;
use std::path::PathBuf;

// pub mod editor;
pub mod menu;
pub mod new_editor;
pub mod new_image_dialog;
pub mod palette_dialog;
pub mod save_palette_dialog;

const BACKGROUND: Color = Color {
    r: 30,
    g: 30,
    b: 140,
    a: 255,
};

fn file_dialog(path: &Option<String>, filters: &[(&str, &str)]) -> FileDialog {
    let mut dialog = FileDialog::new();
    for filter in filters {
        dialog = dialog.add_filter(filter.0, &[filter.1]);
    }
    if let Some(filepath) = path {
        let path = PathBuf::from(filepath);
        let parent_dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(PathBuf::from("/"));
        dialog = dialog.set_directory(parent_dir);
    } else {
        let docs_dir = UserDirs::new()
            .and_then(|ud| ud.document_dir().map(|p| p.to_path_buf()))
            .unwrap_or(PathBuf::from("/"));
        dialog = dialog.set_directory(docs_dir);
    }
    dialog
}
