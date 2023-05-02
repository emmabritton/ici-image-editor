use fnv::FnvHashSet;
use pixels_graphics_lib::prelude::{IndexedImage, IndexedImageError};

pub fn fill_pixels(image: &IndexedImage, start: (u8, u8)) -> Result<Vec<usize>, IndexedImageError> {
    let i = image.get_pixel_index(start.0, start.1)?;
    let replace_color = image.get_pixel(i)?;
    let to_replace = get_valid_neighbours(image, FnvHashSet::default(), start, replace_color)?;

    Ok(to_replace.into_iter().collect())
}

fn get_valid_neighbours(
    image: &IndexedImage,
    set: FnvHashSet<usize>,
    start: (u8, u8),
    replace_color: u8,
) -> Result<FnvHashSet<usize>, IndexedImageError> {
    let start = (start.0 as isize, start.1 as isize);
    let set = check_and_set(image, set, start, replace_color, (-1, 0))?;
    let set = check_and_set(image, set, start, replace_color, (1, 0))?;
    let set = check_and_set(image, set, start, replace_color, (0, -1))?;
    check_and_set(image, set, start, replace_color, (0, 1))
}

fn check_and_set(
    image: &IndexedImage,
    mut set: FnvHashSet<usize>,
    start: (isize, isize),
    replace_color: u8,
    diff: (isize, isize),
) -> Result<FnvHashSet<usize>, IndexedImageError> {
    let target = (start.0 + diff.0, start.1 + diff.1);
    if target.0 >= 0
        && target.0 < image.width() as isize
        && target.1 >= 0
        && target.1 < image.height() as isize
    {
        let start = (target.0 as u8, target.1 as u8);
        let i = image.get_pixel_index(start.0, start.1)?;
        if !set.contains(&i) {
            let color = image.get_pixel(i)?;
            if color == replace_color {
                set.insert(i);
                return get_valid_neighbours(image, set, start, replace_color);
            }
        }
    }
    Ok(set)
}