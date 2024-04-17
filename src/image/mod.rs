use color_eyre::Result;
use pixels_graphics_lib::prelude::IndexedImage;

pub mod resize;
pub mod trim;

fn remove_first_row(image: IndexedImage) -> Result<IndexedImage> {
    let new_pixels = &image.get_pixels()[image.width() as usize..image.get_pixels().len()];
    Ok(IndexedImage::new(
        image.width(),
        image.height() - 1,
        image.get_palette().to_vec(),
        new_pixels.to_vec(),
    )?)
}

fn remove_last_row(image: IndexedImage) -> Result<IndexedImage> {
    let new_pixels = &image.get_pixels()[0..image.get_pixels().len() - image.width() as usize];
    Ok(IndexedImage::new(
        image.width(),
        image.height() - 1,
        image.get_palette().to_vec(),
        new_pixels.to_vec(),
    )?)
}

pub(super) fn remove_first_col(image: IndexedImage) -> Result<IndexedImage> {
    let idx: Vec<usize> = (0..image.height())
        .map(|i| i as usize * image.width() as usize)
        .rev()
        .collect();
    let mut pixels = image.get_pixels().to_vec();
    for i in idx {
        pixels.remove(i);
    }
    Ok(IndexedImage::new(
        image.width() - 1,
        image.height(),
        image.get_palette().to_vec(),
        pixels.to_vec(),
    )?)
}

fn remove_last_col(image: IndexedImage) -> Result<IndexedImage> {
    let idx: Vec<usize> = (0..image.height())
        .map(|i| i as usize * image.width() as usize)
        .rev()
        .collect();
    let mut pixels = image.get_pixels().to_vec();
    for i in idx {
        pixels.remove(i + image.width() as usize - 1);
    }
    Ok(IndexedImage::new(
        image.width() - 1,
        image.height(),
        image.get_palette().to_vec(),
        pixels.to_vec(),
    )?)
}
