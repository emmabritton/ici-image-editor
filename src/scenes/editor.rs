use crate::scenes::editor::AlertAction::{Clear, Close};
use crate::scenes::BACKGROUND;
use crate::ui::prelude::AlertResult::Positive;
use crate::ui::prelude::*;
use crate::SceneName::{Palette, SaveFile};
use crate::SceneUpdateResult::*;
use crate::{Scene, SceneName, SceneResult, HEIGHT, SUR, WIDTH};
use log::error;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::styles::{AlertStyle, ButtonStyle};
use std::fs;
use std::path::PathBuf;

const CANVAS_WIDTH: usize = 200;
const CANVAS_HEIGHT: usize = 200;
const PAL_SIZE: usize = 8;
const PAL_SPACE: usize = 4;
const PAL_SPACED: usize = PAL_SIZE + PAL_SPACE;
const PAL_WIDTH: usize = PAL_SPACED * PALETTE_COLS;
const PAL_HEIGHT: usize = PAL_SPACED * PALETTE_ROWS;
const PALETTE_ROWS: usize = 6;
const PALETTE_COLS: usize = 6;
const PALETTE_POS: Coord = Coord::new(4, 130);
const FILENAME_POS: Coord = Coord::new(2, 2);
const CANVAS_POS: Coord = Coord::new(76, 36);
const SAVE_POS: Coord = Coord::new(2, 16);
const SAVE_AS_POS: Coord = Coord::new(40, 16);
const CLOSE_POS: Coord = Coord::new(234, 16);
const CLEAR_POS: Coord = Coord::new(140, 16);
const PENCIL_POS: Coord = Coord::new(8, 42);
const LINE_POS: Coord = Coord::new(8, 64);
const RECT_POS: Coord = Coord::new(8, 86);
const ERASE_POS: Coord = Coord::new(8, 108);
const PAL_EDIT_POS: Coord = Coord::new(8, (HEIGHT - 20) as isize);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum AlertAction {
    Clear,
    Close,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum DrawingMode {
    Pencil,
    // Line,
    Rect,
    Eraser,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Editor {
    result: SUR,
    file_path: Option<String>,
    file_name: String,
    image: IndexedImage,
    canvas_rect: Rect,
    save: Button,
    clear: Button,
    close: Button,
    save_as: Button,
    pencil: ToggleButton,
    line: ToggleButton,
    rect: ToggleButton,
    erase: ToggleButton,
    tile_size: usize,
    show_grid: bool,
    image_rect: Rect,
    grid_color: Color,
    canvas_color: Color,
    selected_color_idx: usize,
    pal_edit: Button,
    changes: Vec<(Coord, Color)>,
    change_idx: usize,
    alert: Alert,
    pending_alert_action: Option<AlertAction>,
    drawing_mode: DrawingMode,
    shape_start: Option<Coord>,
    transparent_placeholder: ShapeCollection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorDetails {
    Open(String),
    New(usize, usize),
}

impl Editor {
    pub fn new(
        details: EditorDetails,
        width: usize,
        height: usize,
        alert_style: &AlertStyle,
        button_style: &ButtonStyle,
        toggle_button_style: &ToggleButtonStyle,
    ) -> Box<Self> {
        let (file_path, image): (Option<String>, (IndexedImage, FilePalette)) = match details {
            EditorDetails::Open(path) => match fs::read(&path) {
                Ok(data) => (Some(path), IndexedImage::from_file_contents(&data).unwrap()),
                Err(e) => panic!("opening {path}:  {e}"),
            },
            EditorDetails::New(w, h) => {
                let image = IndexedImage::new(
                    w as u8,
                    h as u8,
                    vec![
                        IciColor::transparent(),
                        IciColor::new(0, 0, 0, 255),
                        IciColor::new(255, 255, 255, 255),
                        IciColor::new(255, 0, 0, 255),
                        IciColor::new(0, 255, 0, 255),
                        IciColor::new(0, 0, 255, 255),
                    ],
                    vec![0; w * h],
                )
                .unwrap();
                (None, (image, FilePalette::Colors))
            }
        };
        let image = image.0;
        let canvas_rect = Rect::new_with_size(CANVAS_POS, CANVAS_WIDTH, CANVAS_HEIGHT);
        let save = Button::new(SAVE_POS, "Save", None, button_style);
        let save_as = Button::new(SAVE_AS_POS, "Save as", None, button_style);
        let clear = Button::new(CLEAR_POS, "Clear", None, button_style);
        let close = Button::new(CLOSE_POS, "Close", None, button_style);
        let mut pencil = ToggleButton::new(PENCIL_POS, "Pencil", Some(56), toggle_button_style);
        let line = ToggleButton::new(LINE_POS, "Line", Some(56), toggle_button_style);
        let rect = ToggleButton::new(RECT_POS, "Rect", Some(56), toggle_button_style);
        let erase = ToggleButton::new(ERASE_POS, "Eraser", Some(56), toggle_button_style);
        let file_name = file_path
            .as_ref()
            .map(|p| {
                PathBuf::from(p)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or(String::from("Untitled"));
        let tile_size = (CANVAS_WIDTH / (image.width() as usize).max(image.height() as usize))
            .min(16)
            .max(4);
        let image_width = tile_size * (image.width() as usize);
        let image_height = tile_size * (image.height() as usize);
        let image_rect = Rect::new_with_size(
            CANVAS_POS
                + (
                    (CANVAS_WIDTH - image_width) / 2,
                    (CANVAS_HEIGHT - image_height) / 2,
                ),
            image_width,
            image_height,
        );
        let canvas_color = BLACK;
        let grid_color = DARK_GRAY;
        let pal_edit = Button::new(PAL_EDIT_POS, "Palette", Some(56), button_style);
        let alert = Alert::new_question(
            &["Are you sure?", "All changes will be lost"],
            "Cancel",
            "Yes",
            width,
            height,
            alert_style,
        );
        let mut transparent_placeholder = ShapeCollection::new();
        let trans_size = tile_size / 2;
        InsertShape::insert_above(
            &mut transparent_placeholder,
            Rect::new_with_size((0, 0), trans_size, trans_size),
            fill(LIGHT_GRAY),
        );
        InsertShape::insert_above(
            &mut transparent_placeholder,
            Rect::new_with_size((trans_size, 0), trans_size, trans_size),
            fill(DARK_GRAY),
        );
        InsertShape::insert_above(
            &mut transparent_placeholder,
            Rect::new_with_size((0, trans_size), trans_size, trans_size),
            fill(DARK_GRAY),
        );
        InsertShape::insert_above(
            &mut transparent_placeholder,
            Rect::new_with_size((trans_size, trans_size), trans_size, trans_size),
            fill(LIGHT_GRAY),
        );
        pencil.set_selected(true);
        Box::new(Self {
            result: Nothing,
            alert,
            file_name,
            file_path,
            image,
            canvas_rect,
            save,
            clear,
            close,
            save_as,
            pencil,
            line,
            rect,
            erase,
            tile_size,
            show_grid: false,
            image_rect,
            grid_color,
            canvas_color,
            selected_color_idx: 3,
            pal_edit,
            changes: vec![],
            change_idx: 0,
            pending_alert_action: None,
            drawing_mode: DrawingMode::Pencil,
            shape_start: None,
            transparent_placeholder,
        })
    }

    fn save_image(&self) {
        if let Some(path) = &self.file_path {
            let bytes = self.image.to_file_contents(&FilePalette::Colors).unwrap();
            if let Err(err) = fs::write(path, bytes) {
                error!("saving image to {path}: {}", err);
            }
        }
    }
}

impl Scene<SceneResult, SceneName> for Editor {
    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        graphics.clear(BACKGROUND);

        graphics.draw_text(
            &self.file_name,
            TextPos::px(FILENAME_POS),
            (WHITE, Normal, WrappingStrategy::Ellipsis(38)),
        );
        graphics.draw_line(
            (0, FILENAME_POS.y + 10),
            (WIDTH as isize, FILENAME_POS.y + 10),
            LIGHT_GRAY,
        );

        let mut x = 0;
        let mut y = 0;
        for (i, color) in self.image.get_palette().iter().enumerate() {
            let xy = PALETTE_POS + (x * PAL_SPACED, y * PAL_SPACED);
            graphics.draw_rect(
                Rect::new_with_size(xy, PAL_SIZE, PAL_SIZE),
                fill(Color::rgba(color.r, color.g, color.b, color.a)),
            );
            if self.selected_color_idx == i {
                graphics.draw_rect(Rect::new_with_size(xy, PAL_SIZE, PAL_SIZE), stroke(WHITE));
            }
            x += 1;
            if x >= PALETTE_COLS {
                y += 1;
                x = 0;
            }
        }

        for x in 0..self.image.width() {
            for y in 0..self.image.height() {
                let xy = self.image_rect.top_left()
                    + (x as usize * self.tile_size, y as usize * self.tile_size);
                let i = self.image.get_pixel_index(x, y).unwrap();
                let color_idx = self.image.get_pixel(i).unwrap();
                let color = self.image.get_color(color_idx).unwrap();
                if color.a != 255 {
                    graphics.draw_offset(xy, &self.transparent_placeholder);
                }
                if color != IciColor::transparent() {
                    graphics.draw_rect(
                        Rect::new_with_size(xy, self.tile_size, self.tile_size),
                        fill(Color::rgba(color.r, color.g, color.b, color.a)),
                    );
                }
            }
        }

        if self.image_rect.contains(mouse_xy) {
            let xy = (mouse_xy - self.image_rect.top_left()) / self.tile_size;
            let temp_clr = self.image.get_palette()[self.selected_color_idx];
            graphics.draw_rect(
                Rect::new_with_size(
                    xy * self.tile_size + self.image_rect.top_left(),
                    self.tile_size,
                    self.tile_size,
                ),
                stroke(Color::rgba(temp_clr.r, temp_clr.g, temp_clr.b, temp_clr.a)),
            );
        }

        if self.image_rect.contains(mouse_xy) {
            let xy = (mouse_xy - self.image_rect.top_left()) / self.tile_size;
            let mut color = self.image.get_palette()[self.selected_color_idx];
            color.a = 155;
            match self.drawing_mode {
                DrawingMode::Pencil => graphics.draw_rect(
                    Rect::new_with_size(
                        xy * self.tile_size + self.image_rect.top_left(),
                        self.tile_size,
                        self.tile_size,
                    ),
                    stroke(Color::rgba(color.r, color.g, color.b, color.a)),
                ),
                // DrawingMode::Line => match self.shape_start {
                //     None => graphics.draw_rect(Rect::new_with_size(xy * self.tile_size + self.image_rect.top_left(), self.tile_size, self.tile_size), stroke(color)),
                //     Some(_) => {
                //
                //     }
                // }
                DrawingMode::Rect => match self.shape_start {
                    None => graphics.draw_rect(
                        Rect::new_with_size(
                            xy * self.tile_size + self.image_rect.top_left(),
                            self.tile_size,
                            self.tile_size,
                        ),
                        stroke(Color::rgba(color.r, color.g, color.b, color.a)),
                    ),
                    Some(start) => {
                        let offset = self.image_rect.top_left();
                        let top_left = (xy.x.min(start.x) as usize, xy.y.min(start.y) as usize);
                        let bottom_right = (xy.x.max(start.x) as usize, xy.y.max(start.y) as usize);
                        for x in top_left.0..=bottom_right.0 {
                            graphics.draw_rect(
                                Rect::new_with_size(
                                    offset + (x * self.tile_size, top_left.1 * self.tile_size),
                                    self.tile_size,
                                    self.tile_size,
                                ),
                                stroke(Color::rgba(color.r, color.g, color.b, color.a)),
                            );
                            graphics.draw_rect(
                                Rect::new_with_size(
                                    offset + (x * self.tile_size, bottom_right.1 * self.tile_size),
                                    self.tile_size,
                                    self.tile_size,
                                ),
                                stroke(Color::rgba(color.r, color.g, color.b, color.a)),
                            );
                        }
                        for y in top_left.1..=bottom_right.1 {
                            graphics.draw_rect(
                                Rect::new_with_size(
                                    offset + (top_left.0 * self.tile_size, y * self.tile_size),
                                    self.tile_size,
                                    self.tile_size,
                                ),
                                stroke(Color::rgba(color.r, color.g, color.b, color.a)),
                            );
                            graphics.draw_rect(
                                Rect::new_with_size(
                                    offset + (bottom_right.0 * self.tile_size, y * self.tile_size),
                                    self.tile_size,
                                    self.tile_size,
                                ),
                                stroke(Color::rgba(color.r, color.g, color.b, color.a)),
                            );
                        }
                    }
                },
                DrawingMode::Eraser => graphics.draw_rect(
                    Rect::new_with_size(
                        xy * self.tile_size + self.image_rect.top_left(),
                        self.tile_size,
                        self.tile_size,
                    ),
                    stroke(BLACK),
                ),
            }
        }

        self.save.render(graphics, mouse_xy);
        self.save_as.render(graphics, mouse_xy);
        self.close.render(graphics, mouse_xy);
        self.clear.render(graphics, mouse_xy);
        // self.line.render(graphics, mouse_xy);
        self.rect.render(graphics, mouse_xy);
        self.pencil.render(graphics, mouse_xy);
        self.erase.render(graphics, mouse_xy);
        self.pal_edit.render(graphics, mouse_xy);

        graphics.draw_rect(self.image_rect.clone(), stroke(LIGHT_GRAY));

        if self.pending_alert_action.is_some() {
            self.alert.render(graphics, mouse_xy);
        }
    }

    fn on_key_up(&mut self, key: VirtualKeyCode, held_keys: &Vec<&VirtualKeyCode>) {
        if (held_keys.contains(&&VirtualKeyCode::LControl)
            || held_keys.contains(&&VirtualKeyCode::LWin))
            && key == VirtualKeyCode::Z
        {
            todo!("undo");
        } else if (held_keys.contains(&&VirtualKeyCode::LControl)
            || held_keys.contains(&&VirtualKeyCode::LWin))
            && key == VirtualKeyCode::Y
        {
            todo!("redo");
        }
    }

    fn on_mouse_up(&mut self, xy: Coord, button: MouseButton, _: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left { return; }
        if self.pending_alert_action.is_some() {
            if let Some(result) = self.alert.on_mouse_click(xy) {
                if result == Positive {
                    match &self.pending_alert_action {
                        None => error!("Alert was shown but no pending result"),
                        Some(result) => match result {
                            Clear => {
                                self.image = IndexedImage::new(
                                    self.image.width(),
                                    self.image.height(),
                                    self.image.get_palette().to_vec(),
                                    vec![
                                        0;
                                        self.image.width() as usize * self.image.height() as usize
                                    ],
                                )
                                .unwrap();
                                self.shape_start = None;
                            }
                            Close => {
                                self.result = Pop(None);
                            }
                        },
                    }
                }
                self.pending_alert_action = None;
            }
            return;
        }
        if self.pencil.on_mouse_click(xy) {
            self.line.set_selected(false);
            self.rect.set_selected(false);
            self.erase.set_selected(false);
            self.drawing_mode = DrawingMode::Pencil;
        }
        // if self.line.on_mouse_click(xy) {
        //     self.pencil.set_selected(false);
        //     self.rect.set_selected(false);
        //     self.erase.set_selected(false);
        //     self.drawing_mode = DrawingMode::Line;
        // }
        if self.rect.on_mouse_click(xy) {
            self.pencil.set_selected(false);
            self.line.set_selected(false);
            self.erase.set_selected(false);
            self.drawing_mode = DrawingMode::Rect;
        }
        if self.erase.on_mouse_click(xy) {
            self.pencil.set_selected(false);
            self.line.set_selected(false);
            self.rect.set_selected(false);
            self.drawing_mode = DrawingMode::Eraser;
        }
        if self.image_rect.contains(xy) {
            let xy = (xy - self.image_rect.top_left()) / self.tile_size;
            let i = xy.y as usize * self.image.width() as usize + xy.x as usize;
            match self.drawing_mode {
                DrawingMode::Pencil => self
                    .image
                    .set_pixel(i, self.selected_color_idx as u8)
                    .unwrap(),
                // DrawingMode::Line => match self.shape_start {
                //     None => self.shape_start = Some(xy),
                //     Some(_) => {
                //
                //     }
                // }
                DrawingMode::Rect => match self.shape_start {
                    None => self.shape_start = Some(xy),
                    Some(start) => {
                        let width = self.image.width() as isize;
                        let color = self.selected_color_idx as u8;
                        let top_left = (xy.x.min(start.x), xy.y.min(start.y));
                        let bottom_right = (xy.x.max(start.x), xy.y.max(start.y));
                        for x in top_left.0..=bottom_right.0 {
                            let top = top_left.1 * width + x;
                            let bottom = bottom_right.1 * width + x;
                            self.image.set_pixel(top as usize, color).unwrap();
                            self.image.set_pixel(bottom as usize, color).unwrap();
                        }
                        for y in top_left.1..=bottom_right.1 {
                            let left = y * width + top_left.0;
                            let right = y * width + bottom_right.0;
                            self.image.set_pixel(left as usize, color).unwrap();
                            self.image.set_pixel(right as usize, color).unwrap();
                        }
                        self.shape_start = None;
                    }
                },
                DrawingMode::Eraser => self.image.set_pixel(i, 0).unwrap(),
            }
        }
        if self.close.on_mouse_click(xy) {
            self.pending_alert_action = Some(Close);
        }
        if self.clear.on_mouse_click(xy) {
            self.pending_alert_action = Some(Clear);
        }
        if self.save.on_mouse_click(xy) {
            if self.file_path.is_some() {
                self.save_image();
            } else {
                self.result = Push(false, SaveFile(String::from("ici"), None))
            }
        }
        if self.save_as.on_mouse_click(xy) {
            self.result = Push(false, SaveFile(String::from("ici"), None))
        }
        if self.pal_edit.on_mouse_click(xy) {
            let mut colors = vec![];
            for color in self.image.get_palette() {
                colors.push(Color::rgba(color.r, color.g, color.b, color.a));
            }
            self.result = Push(false, Palette(colors))
        }
        if Rect::new_with_size(PALETTE_POS, PAL_WIDTH, PAL_HEIGHT).contains(xy) {
            let x = (xy.x - PALETTE_POS.x) / PAL_SPACED as isize;
            let y = (xy.y - PALETTE_POS.y) / PAL_SPACED as isize;
            let idx = (x + y * PALETTE_COLS as isize) as usize;
            if idx < self.image.get_palette().len() {
                self.selected_color_idx = idx;
            }
        }
    }

    fn update(&mut self, _: &Timing, _: Coord, _: &Vec<&VirtualKeyCode>) -> SUR {
        self.result.clone()
    }

    fn resuming(&mut self, result: Option<SceneResult>) {
        match result {
            Some(SceneResult::SaveFilePath(path)) => {
                self.file_name = PathBuf::from(&path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                self.file_path = Some(path);
                self.save_image();
            }
            Some(SceneResult::Palette(colors)) => {
                let mut ici_colors = vec![];
                for color in &colors {
                    ici_colors.push(IciColor::new(color.r, color.g, color.b, color.a))
                }
                self.image.set_palette(&ici_colors).unwrap();
                self.selected_color_idx = self
                    .selected_color_idx
                    .min(self.image.get_palette().len() - 1);
            }
            _ => {}
        }
        self.result = Nothing;
    }
}
