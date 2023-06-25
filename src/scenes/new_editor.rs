use crate::palettes::Palette;
use crate::scenes::{file_dialog, BACKGROUND};
use crate::ui::canvas::{Canvas, Tool};
use crate::ui::palette::PaletteView;
use crate::{SceneName, SceneResult, SUR, WIDTH};

use log::error;
use pixels_graphics_lib::buffer_graphics_lib::prelude::*;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::scenes::SceneUpdateResult::{Nothing, Pop};
use pixels_graphics_lib::ui::prelude::TextFilter::Decimal;
use pixels_graphics_lib::ui::prelude::*;

use crate::ui::edit_history::EditHistory;
use crate::ui::preview::Preview;
use crate::ui::timeline::Timeline;
use std::fs;
use std::ops::Add;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const PER_UNDO: u64 = 200;

#[allow(unused)] //will be soon
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum AlertAction {
    Clear,
    Close,
}

const NAME: Coord = Coord::new(2, 2);
const NAME_LINE_Y: isize = 12;
const PADDING: isize = 4;
const BUTTON_Y: isize = NAME_LINE_Y + PADDING;
const FRAME_CONTROL: Coord = Coord::new(4, 200);
const FRAME_CONTROL_SPACING: isize = 20;
const PALETTE_HEIGHT: usize = 64;

const TOOL_PENCIL: usize = 0;
const TOOL_LINE: usize = 1;
const TOOL_RECT: usize = 2;
const TOOL_FILL: usize = 3;

const UNTITLED: &str = "untitled";
const CORRUPT: &str = "???";

#[derive(Debug, Clone, PartialEq)]
pub enum EditorDetails {
    Open(String),
    New(u8, u8),
}

pub struct Editor {
    result: SUR,
    clear: Button,
    tools: ToggleIconButtonGroup<usize>,
    save: Button,
    save_as: Button,
    close: Button,
    edit_palette: Button,
    speed: TextField,
    play_pause: IconButton,
    add_frame: IconButton,
    remove_frame: IconButton,
    copy_frame: IconButton,
    #[allow(unused)] //will be soon
    alert: Alert,
    #[allow(unused)] //will be soon
    pending_alert_action: Option<AlertAction>,
    filepath: Option<String>,
    canvas: Canvas,
    palette: PaletteView,
    last_undo: Instant,
    preview: Preview,
    timeline: Timeline,
    history: EditHistory,
    is_playing: bool,
    next_frame_swap: f64,
    anim_frame_idx: usize,
}

impl Editor {
    pub fn new(
        width: usize,
        height: usize,
        details: EditorDetails,
        style: &UiStyle,
    ) -> Box<Editor> {
        let save = Button::new((PADDING, BUTTON_Y), "Save", None, &style.button);
        let save_as = Button::new(
            (save.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Save As",
            None,
            &style.button,
        );
        let clear = Button::new((188, BUTTON_Y), "Clear", None, &style.button);
        let close = Button::new(
            (
                (width - clear.bounds().width()) as isize - PADDING,
                BUTTON_Y,
            ),
            "Close",
            None,
            &style.button,
        );
        let edit_palette = Button::new(
            (PADDING, save.bounds().bottom_right().y + PADDING),
            "Palette",
            None,
            &style.button,
        );
        let alert = Alert::new_question(
            &["Are you sure?", "All changes will be lost"],
            "Cancel",
            "Yes",
            width,
            height,
            &style.alert,
        );
        let add_frame = IconButton::new(
            FRAME_CONTROL,
            "Add frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/add.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let remove_frame = IconButton::new(
            FRAME_CONTROL + (FRAME_CONTROL_SPACING, 0),
            "Remove frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/remove.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let copy_frame = IconButton::new(
            FRAME_CONTROL + (FRAME_CONTROL_SPACING * 2, 0),
            "Copy frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/copy.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let play_pause = IconButton::new(
            FRAME_CONTROL + (0, FRAME_CONTROL_SPACING),
            "Play/Pause",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/play_pause.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut speed = TextField::new(
            FRAME_CONTROL + (FRAME_CONTROL_SPACING, FRAME_CONTROL_SPACING),
            6,
            TextSize::Normal,
            (None, Some(40)),
            "0.1",
            &[Decimal],
            &style.text_field,
        );
        let pencil_tool = ToggleIconButton::new(
            (105, BUTTON_Y),
            "Pencil",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/pencil.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let line_tool = ToggleIconButton::new(
            (pencil_tool.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Line",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/line.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let rect_tool = ToggleIconButton::new(
            (line_tool.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Rect",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/rect.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let fill_tool = ToggleIconButton::new(
            (rect_tool.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Fill",
            Positioning::CenterBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/fill.ici"))
                .unwrap()
                .0,
            &style.toggle_icon_button,
        );
        let tools = ToggleIconButtonGroup::new(vec![
            (TOOL_PENCIL, pencil_tool),
            (TOOL_LINE, line_tool),
            (TOOL_RECT, rect_tool),
            (TOOL_FILL, fill_tool),
        ]);
        let mut filepath = None;
        let frames = match details {
            EditorDetails::Open(path) => {
                let is_animated = path.contains(".ica");
                filepath = Some(path.clone());
                let bytes = fs::read(path).expect("Reading image from file");
                let (image, pal) = if is_animated {
                    let (image, pal) = AnimatedIndexedImage::from_file_contents(&bytes)
                        .expect("Reading animated image data");
                    speed.set_content(&image.get_per_frame().to_string());
                    (image.as_images(), pal)
                } else {
                    let (image, pal) =
                        IndexedImage::from_file_contents(&bytes).expect("Reading image data");
                    (vec![image], pal)
                };
                if pal != FilePalette::Colors {
                    panic!("Currently {pal:?} isn't supported");
                }
                image
            }
            EditorDetails::New(w, h) => {
                vec![IndexedImage::new(
                    w,
                    h,
                    Palette::default()
                        .colors
                        .iter()
                        .map(|c| c.to_ici())
                        .collect(),
                    vec![0; w as usize * h as usize],
                )
                .unwrap()]
            }
        };

        let mut canvas = Canvas::new(
            Coord::new(
                edit_palette.bounds().bottom_right().x + PADDING,
                edit_palette.bounds().top_left().y,
            ),
            (210, 200),
        );

        canvas.set_color_index(1);
        canvas.set_image(frames[0].clone());
        let mut palette = PaletteView::new(
            Coord::new(
                edit_palette.bounds().top_left().x,
                edit_palette.bounds().bottom_right().y + PADDING,
            ),
            (edit_palette.bounds().width(), PALETTE_HEIGHT),
        );
        palette.set_palette(canvas.get_image().get_palette());
        palette.set_color_index(1);
        let mut preview = Preview::new(Rect::new_with_size((4, 122), 64, 73));
        let mut timeline = Timeline::new(Rect::new_with_size((-1, -1), 0, 0));
        timeline.set_frames(frames.clone(), 0);
        let history = EditHistory::new(frames.clone());
        preview.set_image(history.get_current_image().clone());
        let mut editor = Self {
            timeline,
            result: Nothing,
            clear,
            tools,
            save,
            save_as,
            close,
            edit_palette,
            speed,
            play_pause,
            add_frame,
            remove_frame,
            copy_frame,
            alert,
            palette,
            pending_alert_action: None,
            filepath,
            canvas,
            preview,
            last_undo: Instant::now(),
            history,
            is_playing: false,
            next_frame_swap: 0.0,
            anim_frame_idx: 0,
        };
        editor.relayout_canvas(frames.len() > 1);
        Box::new(editor)
    }

    fn open_save_as(&mut self, frame_idx: Option<usize>) {
        let filters = if frame_idx.is_some() {
            &[("IndexedImage", "ici")]
        } else {
            &[("AnimatedIndexedImage", "ica")]
        };
        if let Some(path) = file_dialog(&self.filepath, filters).save_file() {
            self.filepath = Some(path.to_string_lossy().to_string());
            self.save_file(frame_idx);
        }
    }

    fn save_file(&self, frame_idx: Option<usize>) {
        if let Some(filepath) = &self.filepath {
            if let Some(idx) = frame_idx {
                let bytes = self
                    .history
                    .get_image(idx)
                    .to_file_contents(&FilePalette::Colors)
                    .expect("Unable to save ici file (converting)");
                fs::write(filepath, bytes).expect("Unable to save ici file (writing)");
            } else {
                let frames = self.history.get_images();
                let pixels = frames
                    .iter()
                    .flat_map(|i| i.get_pixels().to_vec())
                    .collect();
                let image = AnimatedIndexedImage::new(
                    frames[0].width(),
                    frames[1].height(),
                    self.speed.content().parse::<f64>().unwrap_or(1.0),
                    frames.len() as u8,
                    frames[0].get_palette().to_vec(),
                    pixels,
                    PlayType::Loops,
                )
                .unwrap();
                let bytes = image
                    .to_file_contents(&FilePalette::Colors)
                    .expect("Unable to save ica file (converting)");
                fs::write(filepath, bytes).expect("Unable to save ica file (writing)");
            }
        } else {
            error!("save_file called but no filepath set")
        }
    }

    fn relayout_canvas(&mut self, show_timeline: bool) {
        let state = self.canvas.get_usage_state();
        if show_timeline {
            let image_height = self.history.get_current_image().height() as usize;
            self.timeline = Timeline::new(Rect::new_with_size(
                (
                    self.preview.bounds().right() + PADDING,
                    self.play_pause.bounds().bottom() - (image_height as isize + PADDING),
                ),
                200,
                image_height + 4,
            ));
            self.canvas = Canvas::new(
                Coord::new(
                    self.edit_palette.bounds().bottom_right().x + PADDING,
                    self.edit_palette.bounds().top_left().y,
                ),
                (
                    210,
                    (self.timeline.bounds().top()
                        - self.edit_palette.bounds().top_left().y
                        - PADDING) as usize,
                ),
            );
            self.play_pause.set_state(ElementState::Normal);
        } else {
            self.timeline = Timeline::new(Rect::new_with_size((-1, -1), 0, 0));
            self.canvas = Canvas::new(
                Coord::new(
                    self.edit_palette.bounds().bottom_right().x + PADDING,
                    self.edit_palette.bounds().top_left().y,
                ),
                (210, 200),
            );
            self.play_pause.set_state(ElementState::Disabled);
        }
        self.canvas.set_color_index(self.palette.get_selected_idx());
        self.canvas
            .set_image(self.history.get_current_image().clone());
        self.timeline
            .set_frames(self.history.get_images(), self.history.active_frame());
        self.canvas.set_usage_state(state);
    }
}

impl Scene<SceneResult, SceneName> for Editor {
    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        graphics.clear(BACKGROUND);

        let name = if let Some(path) = &self.filepath {
            PathBuf::from(path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .map(|s| {
                    if s.ends_with(".ici") || s.ends_with(".ica") {
                        s.chars().take(s.chars().count() - 4).collect()
                    } else {
                        s
                    }
                })
                .unwrap_or(CORRUPT.to_string())
        } else {
            UNTITLED.to_string()
        };
        graphics.draw_text(
            &name,
            TextPos::px(NAME),
            (WHITE, TextSize::Normal, WrappingStrategy::Ellipsis(38)),
        );
        graphics.draw_line((0, NAME_LINE_Y), (WIDTH as isize, NAME_LINE_Y), LIGHT_GRAY);

        self.speed.render(graphics, mouse_xy);
        self.add_frame.render(graphics, mouse_xy);
        self.remove_frame.render(graphics, mouse_xy);
        self.copy_frame.render(graphics, mouse_xy);
        self.play_pause.render(graphics, mouse_xy);
        self.tools.render(graphics, mouse_xy);
        self.save.render(graphics, mouse_xy);
        self.save_as.render(graphics, mouse_xy);
        self.clear.render(graphics, mouse_xy);
        self.close.render(graphics, mouse_xy);
        self.palette.render(graphics, mouse_xy);
        self.edit_palette.render(graphics, mouse_xy);
        self.canvas.render(graphics, mouse_xy);
        self.preview.render(graphics, mouse_xy);
        self.timeline.render(graphics, mouse_xy);
    }

    fn on_key_down(&mut self, key: VirtualKeyCode, _: Coord, held: &Vec<&VirtualKeyCode>) {
        if self.last_undo < Instant::now() {
            if key == VirtualKeyCode::Z
                && !held.contains(&&VirtualKeyCode::LShift)
                && (held.contains(&&VirtualKeyCode::LControl)
                    || held.contains(&&VirtualKeyCode::LWin))
            {
                self.history.undo().unwrap();
                self.last_undo = Instant::now().add(Duration::from_millis(PER_UNDO));
                self.palette.set_palette(self.canvas.get_palette());
                self.canvas
                    .set_image(self.history.get_current_image().clone());
                self.timeline
                    .set_frames(self.history.get_images(), self.history.active_frame());
                self.preview
                    .set_image(self.history.get_current_image().clone());
            }
            if ((key == VirtualKeyCode::Z && held.contains(&&VirtualKeyCode::LShift))
                || key == VirtualKeyCode::Y)
                && (held.contains(&&VirtualKeyCode::LControl)
                    || held.contains(&&VirtualKeyCode::LWin))
            {
                self.history.redo().unwrap();
                self.last_undo = Instant::now().add(Duration::from_millis(PER_UNDO));
                self.palette.set_palette(self.canvas.get_palette());
                self.canvas
                    .set_image(self.history.get_current_image().clone());
                self.timeline
                    .set_frames(self.history.get_images(), self.history.active_frame());
                self.preview
                    .set_image(self.history.get_current_image().clone());
            }
        }
    }

    fn on_key_up(&mut self, key: VirtualKeyCode, _: Coord, held: &Vec<&VirtualKeyCode>) {
        self.speed.on_key_press(key, held);
    }

    fn on_mouse_down(&mut self, xy: Coord, button: MouseButton, _: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left {
            return;
        }
        if self.canvas.on_mouse_down(xy, &mut self.history) {
            self.canvas
                .set_image(self.history.get_current_image().clone());
            self.preview.set_image(self.canvas.get_image().clone());
            self.timeline
                .update_frame(self.history.get_current_image().clone());
            if self.history.is_first_event_light_pixel() {
                let color = self.preview.select_dark_background();
                self.timeline.set_background(color)
            }
        }
    }

    fn on_mouse_up(&mut self, xy: Coord, button: MouseButton, keys: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left {
            return;
        }
        if let Some(tool_id) = self.tools.on_mouse_click(xy) {
            match tool_id {
                TOOL_PENCIL => self.canvas.set_tool(Tool::Pencil),
                TOOL_LINE => self.canvas.set_tool(Tool::Line),
                TOOL_RECT => self.canvas.set_tool(Tool::Rect),
                TOOL_FILL => self.canvas.set_tool(Tool::Fill),
                _ => {}
            }
        }
        if self.play_pause.on_mouse_click(xy) {
            if self.is_playing {
                self.is_playing = false;
                self.add_frame.set_state(ElementState::Normal);
                self.speed.set_state(ElementState::Normal);
                self.remove_frame.set_state(ElementState::Normal);
                self.copy_frame.set_state(ElementState::Normal);
                self.timeline.set_state(ElementState::Normal);
                self.palette.set_state(ElementState::Normal);
                self.canvas.set_state(ElementState::Normal);
                self.clear.set_state(ElementState::Normal);
                self.edit_palette.set_state(ElementState::Normal);
            } else {
                self.is_playing = true;
                self.anim_frame_idx = 0;
                self.next_frame_swap = self.speed.content().parse::<f64>().unwrap_or(1.0);
                self.add_frame.set_state(ElementState::Disabled);
                self.speed.set_state(ElementState::Disabled);
                self.remove_frame.set_state(ElementState::Disabled);
                self.copy_frame.set_state(ElementState::Disabled);
                self.timeline.set_state(ElementState::Disabled);
                self.palette.set_state(ElementState::Disabled);
                self.canvas.set_state(ElementState::Disabled);
                self.clear.set_state(ElementState::Disabled);
                self.edit_palette.set_state(ElementState::Disabled);
            }
        }
        if self.add_frame.on_mouse_click(xy) {
            self.history.add_blank_frame().unwrap();
            self.timeline
                .set_frames(self.history.get_images(), self.history.active_frame());
            if self.history.frame_count() == 2 {
                self.relayout_canvas(true);
            }
        }
        if self.remove_frame.on_mouse_click(xy) {
            self.history.remove_frame().unwrap();
            self.timeline
                .set_frames(self.history.get_images(), self.history.active_frame());
            if self.history.frame_count() == 1 {
                self.relayout_canvas(false);
            }
        }
        if self.copy_frame.on_mouse_click(xy) {
            self.history.add_duplicate_frame().unwrap();
            self.timeline
                .set_frames(self.history.get_images(), self.history.active_frame());
            if self.history.frame_count() == 2 {
                self.relayout_canvas(true);
            }
        }
        if self.close.on_mouse_click(xy) {
            self.result = Pop(None);
        }
        if self.clear.on_mouse_click(xy) {
            self.history.add_clear().unwrap();
        }
        if self.save.on_mouse_click(xy) {
            let idx = if keys.contains(&&VirtualKeyCode::LShift) || self.history.frame_count() == 1
            {
                Some(self.history.active_frame())
            } else {
                None
            };
            if self.filepath.is_some() {
                self.save_file(idx);
            } else {
                self.open_save_as(idx);
            }
        }
        if self.save_as.on_mouse_click(xy) {
            let idx = if keys.contains(&&VirtualKeyCode::LShift) || self.history.frame_count() == 1
            {
                Some(self.history.active_frame())
            } else {
                None
            };
            self.open_save_as(idx);
        }
        self.speed.on_mouse_click(xy);
        if self.edit_palette.on_mouse_click(xy) {
            let colors = self
                .canvas
                .get_image()
                .get_palette()
                .iter()
                .map(|c| c.to_color())
                .collect();
            self.result = SceneUpdateResult::Push(false, SceneName::Palette(colors));
        }
        if self.palette.on_mouse_click(xy) {
            self.canvas.set_color_index(self.palette.get_selected_idx());
        }
        self.canvas.on_mouse_up(xy, &mut self.history);
        let background_color = self.preview.on_mouse_click(xy);
        self.timeline.set_background(background_color);
        if let Some(frame) = self.timeline.on_mouse_click(xy) {
            self.history.add_frame_select(frame).unwrap();
        }

        self.canvas
            .set_image(self.history.get_current_image().clone());
        self.preview.set_image(self.canvas.get_image().clone());
        self.timeline
            .update_frame(self.history.get_current_image().clone());
    }

    fn on_scroll(&mut self, xy: Coord, x_diff: isize, y_diff: isize, _: &Vec<&VirtualKeyCode>) {
        self.palette.on_scroll(xy, y_diff);
        self.timeline.on_scroll(xy, x_diff);
    }

    fn update(
        &mut self,
        timing: &Timing,
        _xy: Coord,
        _: &Vec<&VirtualKeyCode>,
    ) -> SceneUpdateResult<SceneResult, SceneName> {
        self.speed.update(timing);

        if self.is_playing {
            self.next_frame_swap -= timing.fixed_time_step;
            if self.next_frame_swap <= 0.0 {
                self.next_frame_swap = self.speed.content().parse::<f64>().unwrap_or(1.0);
                self.anim_frame_idx += 1;
                if self.anim_frame_idx >= self.history.frame_count() {
                    self.anim_frame_idx = 0;
                }
                let image = self.history.get_image(self.anim_frame_idx);
                self.canvas.set_image(image.clone());
                self.preview.set_image(image.clone());
                self.timeline.set_active(self.anim_frame_idx)
            }
        } else {
            let frame_count = self.history.frame_count();
            if frame_count == 1 {
                self.remove_frame.set_state(ElementState::Disabled);
            } else {
                self.remove_frame.set_state(ElementState::Normal);
            }
            if frame_count > 254 {
                self.add_frame.set_state(ElementState::Disabled);
                self.copy_frame.set_state(ElementState::Disabled);
            } else {
                self.add_frame.set_state(ElementState::Normal);
                self.copy_frame.set_state(ElementState::Normal);
            }
        }

        self.result.clone()
    }

    fn resuming(&mut self, result: Option<SceneResult>) {
        if let Some(SceneResult::Palette(colors)) = result {
            let colors: Vec<IciColor> = colors.iter().map(|c| c.to_ici()).collect();
            self.palette.set_palette(&colors);
            self.palette.set_color_index(0);
            if let Err(e) = self.history.add_palette_change(&colors) {
                panic!("Failed to update palette: {e} (please raise issue on github)");
            }
            self.timeline
                .set_frames(self.history.get_images(), self.history.active_frame());
            self.preview
                .set_image(self.history.get_current_image().clone());
            self.canvas
                .set_image(self.history.get_current_image().clone());
            self.canvas.set_color_index(0);
        }
        self.result = Nothing;
    }
}
