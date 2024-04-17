use crate::image::*;
use color_eyre::Result;
use pixels_graphics_lib::prelude::IndexedImage;

pub fn remove_blanks(mut image: IndexedImage) -> Result<IndexedImage> {
    let trans_idxs = get_transparent_colors(&image);
    if trans_idxs.is_empty() {
        return Ok(image);
    }
    if all_transparent(&image, &trans_idxs) {
        return Ok(image);
    }
    while is_row_blank(&image, 0, &trans_idxs) {
        image = remove_first_row(image)?;
    }
    while is_col_blank(&image, 0, &trans_idxs) {
        image = remove_first_col(image)?;
    }
    while is_row_blank(&image, image.size().1 - 1, &trans_idxs) {
        image = remove_last_row(image)?;
    }
    while is_col_blank(&image, image.size().0 - 1, &trans_idxs) {
        image = remove_last_col(image)?;
    }
    Ok(image)
}

fn get_transparent_colors(image: &IndexedImage) -> Vec<u8> {
    let mut output = vec![];
    for (i, color) in image.get_palette().iter().enumerate() {
        if color.a == 0 {
            output.push(i as u8);
        }
    }
    output
}

fn all_transparent(image: &IndexedImage, trans_idxs: &[u8]) -> bool {
    image.get_pixels().iter().all(|i| trans_idxs.contains(i))
}

fn is_row_blank(image: &IndexedImage, row: u8, trans_idxs: &[u8]) -> bool {
    all_pixels_transparent(
        image,
        (0..image.width())
            .map(|i| row as usize * image.width() as usize + i as usize)
            .collect(),
        trans_idxs,
    )
}

fn is_col_blank(image: &IndexedImage, idx: u8, trans_idxs: &[u8]) -> bool {
    all_pixels_transparent(
        image,
        (0..image.height())
            .map(|i| i as usize * image.width() as usize + idx as usize)
            .collect(),
        trans_idxs,
    )
}

fn all_pixels_transparent(image: &IndexedImage, px: Vec<usize>, trans_idxs: &[u8]) -> bool {
    for idx in px {
        if !trans_idxs.contains(&image.get_pixels()[idx]) {
            return false;
        }
    }
    true
}
