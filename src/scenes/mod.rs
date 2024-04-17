use crate::{Settings, HEIGHT, WIDTH};
use directories::UserDirs;
use log::error;
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prelude::AppPrefs;
use pixels_graphics_lib::ui::alert::Alert;
use pixels_graphics_lib::ui::styles::AlertStyle;
use rfd::FileDialog;
use std::path::PathBuf;

pub mod editor;
mod editor_ui;
pub mod menu;
pub mod new_image_dialog;
pub mod palette_dialog;
pub mod resize_dialog;
pub mod save_palette_dialog;
pub mod simplify_dialog;

const BACKGROUND: Color = Color {
    r: 30,
    g: 30,
    b: 140,
    a: 255,
};

fn file_dialog(path: PathBuf, filters: &[(&str, &str)]) -> FileDialog {
    let mut dialog = FileDialog::new();
    for filter in filters {
        dialog = dialog.add_filter(filter.0, &[filter.1]);
    }
    if path.exists() {
        dialog = dialog.set_directory(path);
    } else {
        let docs_dir = UserDirs::new()
            .and_then(|ud| ud.document_dir().map(|p| p.to_path_buf()))
            .unwrap_or(PathBuf::from("/"));
        dialog = dialog.set_directory(docs_dir);
    }
    dialog
}

pub fn import_image(
    alert_style: &AlertStyle,
    settings: &mut AppPrefs<Settings>,
) -> Option<Result<IndexedImage, Alert>> {
    if let Some(path) = file_dialog(
        settings.data.last_used_png_dir.clone(),
        &[
            ("PNG", "png"),
            ("JPG", "jpg"),
            ("JPEG", "jpeg"),
            ("TGA", "tga"),
            ("BMP", "bmp"),
            ("Icon", "ico"),
        ],
    )
    .pick_file()
    {
        return match open_image(&path) {
            Ok(img) => {
                if img.width() > 64 || img.height() > 64 {
                    Some(Err(Alert::new_warning(
                        &["Image is too big", "(max 64x64)"],
                        WIDTH,
                        HEIGHT,
                        alert_style,
                    )))
                } else {
                    let mut buffer = Graphics::create_buffer(img.width(), img.height());
                    let mut graphics = Graphics::new(&mut buffer, img.width(), img.height())
                        .expect("creating graphics for imported image");
                    graphics.draw_image((0, 0), &img);
                    match graphics.copy_to_indexed_image(false) {
                        Ok(ici) => {
                            settings.data.last_used_png_dir = path;
                            settings.save();
                            Some(Ok(ici))
                        }
                        Err(e) => {
                            error!("Error converting image from {path:?}: {e:?}");
                            Some(Err(Alert::new_warning(
                                &["Error importing image"],
                                WIDTH,
                                HEIGHT,
                                alert_style,
                            )))
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error importing image from {path:?}: {e:?}");
                Some(Err(Alert::new_warning(
                    &["Error opening image"],
                    WIDTH,
                    HEIGHT,
                    alert_style,
                )))
            }
        };
    }
    None
}
