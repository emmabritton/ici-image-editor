use crate::image::*;
use color_eyre::Result;
use pixels_graphics_lib::prelude::IndexedImage;

use crate::scenes::resize_dialog::ResizeAnchor;

pub fn resize(
    w: u8,
    h: u8,
    resize_anchor: ResizeAnchor,
    img: &IndexedImage,
) -> Result<IndexedImage> {
    let mut img = img.clone();

    let mut alternate = true;

    while w < img.width() {
        if resize_anchor.is_left() {
            img = remove_last_col(img)?;
        } else if resize_anchor.is_right() {
            img = remove_first_col(img)?;
        } else {
            img = if alternate {
                remove_first_col(img)?
            } else {
                remove_last_col(img)?
            };
            alternate = !alternate;
        }
    }

    while h < img.height() {
        if resize_anchor.is_top() {
            img = remove_last_row(img)?;
        } else if resize_anchor.is_bottom() {
            img = remove_first_row(img)?;
        } else {
            img = if alternate {
                remove_first_row(img)?
            } else {
                remove_last_row(img)?
            };
            alternate = !alternate;
        }
    }

    while w > img.width() {
        if resize_anchor.is_left() {
            img = add_last_col(img)?;
        } else if resize_anchor.is_right() {
            img = add_first_col(img)?;
        } else {
            img = if alternate {
                add_first_col(img)?
            } else {
                add_last_col(img)?
            };
            alternate = !alternate;
        }
    }

    while h > img.height() {
        if resize_anchor.is_top() {
            img = add_last_row(img)?;
        } else if resize_anchor.is_bottom() {
            img = add_first_row(img)?;
        } else {
            img = if alternate {
                add_first_row(img)?
            } else {
                add_last_row(img)?
            };
            alternate = !alternate;
        }
    }

    Ok(img)
}

fn add_last_row(image: IndexedImage) -> Result<IndexedImage> {
    let mut pixels = image.get_pixels().to_vec();
    pixels.resize(pixels.len() + image.width() as usize, 0);

    Ok(IndexedImage::new(
        image.width(),
        image.height() + 1,
        image.get_palette().to_vec(),
        pixels.to_vec(),
    )?)
}

fn add_first_row(image: IndexedImage) -> Result<IndexedImage> {
    let mut pixels = image.get_pixels().to_vec();
    for _ in 0..image.width() {
        pixels.insert(0, 0);
    }

    Ok(IndexedImage::new(
        image.width(),
        image.height() + 1,
        image.get_palette().to_vec(),
        pixels.to_vec(),
    )?)
}

fn add_first_col(image: IndexedImage) -> Result<IndexedImage> {
    let idx: Vec<usize> = (0..image.height())
        .map(|i| i as usize * image.width() as usize)
        .rev()
        .collect();
    let mut pixels = image.get_pixels().to_vec();
    for i in idx {
        pixels.insert(i, 0);
    }
    Ok(IndexedImage::new(
        image.width() + 1,
        image.height(),
        image.get_palette().to_vec(),
        pixels.to_vec(),
    )?)
}

fn add_last_col(image: IndexedImage) -> Result<IndexedImage> {
    let idx: Vec<usize> = (0..image.height())
        .map(|i| i as usize * image.width() as usize)
        .rev()
        .collect();
    let mut pixels = image.get_pixels().to_vec();
    for i in idx {
        pixels.insert(i + image.width() as usize, 0);
    }
    Ok(IndexedImage::new(
        image.width() + 1,
        image.height(),
        image.get_palette().to_vec(),
        pixels.to_vec(),
    )?)
}
