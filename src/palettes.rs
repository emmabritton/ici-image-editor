use pixels_graphics_lib::buffer_graphics_lib::prelude::*;

const PAL_DOS: &str = include_str!("../assets/palettes/dos.pal");
const PAL_GB: &str = include_str!("../assets/palettes/gb.pal");
const PAL_PICO: &str = include_str!("../assets/palettes/pico.pal");
const PAL_VIC20: &str = include_str!("../assets/palettes/vic_20.pal");
const PAL_ZX: &str = include_str!("../assets/palettes/zx_spectrum.pal");

pub fn palette_dos() -> JascPalette {
    JascPalette::from_file_contents(PAL_DOS).unwrap()
}

pub fn palette_gb() -> JascPalette {
    JascPalette::from_file_contents(PAL_GB).unwrap()
}

pub fn palette_pico() -> JascPalette {
    JascPalette::from_file_contents(PAL_PICO).unwrap()
}

pub fn palette_zx() -> JascPalette {
    JascPalette::from_file_contents(PAL_ZX).unwrap()
}

pub fn palette_vic() -> JascPalette {
    JascPalette::from_file_contents(PAL_VIC20).unwrap()
}

pub fn palette_default() -> JascPalette {
    JascPalette::new(
        vec![
            TRANSPARENT,
            WHITE,
            BLACK,
            LIGHT_GRAY,
            DARK_GRAY,
            RED,
            GREEN,
            BLUE,
            CYAN,
            MAGENTA,
            YELLOW,
            ORANGE,
            PURPLE,
            BROWN,
        ]
        .iter()
        .map(|c| c.to_ici())
        .collect(),
    )
}
