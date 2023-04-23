use crate::scenes::BACKGROUND;
use crate::{SceneName, SceneResult, SUR, WIDTH};
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::scenes::SceneUpdateResult::Nothing;
use pixels_graphics_lib::ui::prelude::TextFilter::{Decimal, Filename};
use pixels_graphics_lib::ui::prelude::*;

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

const TOOL_PENCIL: usize = 0;
const TOOL_LINE: usize = 1;
const TOOL_RECT: usize = 2;

pub struct Editor {
    result: SUR,
    clear: Button,
    tools: ToggleIconButtonGroup<usize>,
    save: Button,
    save_as: Button,
    close: Button,
    palette: Button,
    speed: TextField,
    play_pause: IconButton,
    add_frame: IconButton,
    remove_frame: IconButton,
    copy_frame: IconButton,
    alert: Alert,
    pending_alert_action: Option<AlertAction>,
    file_name: String,
}

impl Editor {
    pub fn new(width: usize, height: usize, style: &UiStyle) -> Box<Editor> {
        let save = Button::new((PADDING, BUTTON_Y), "Save", None, &style.button);
        let save_as = Button::new(
            (save.bounds().bottom_right().x + PADDING, BUTTON_Y),
            "Save As",
            None,
            &style.button,
        );
        let clear = Button::new((180, BUTTON_Y), "Clear", None, &style.button);
        let close = Button::new(
            (
                (width - clear.bounds().width()) as isize - PADDING,
                BUTTON_Y,
            ),
            "Close",
            None,
            &style.button,
        );
        let palette = Button::new(
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
        let mut add_frame = IconButton::new(
            FRAME_CONTROL,
            "Add frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/add.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut remove_frame = IconButton::new(
            FRAME_CONTROL + (FRAME_CONTROL_SPACING, 0),
            "Remove frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/remove.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut copy_frame = IconButton::new(
            FRAME_CONTROL + (FRAME_CONTROL_SPACING * 2, 0),
            "Copy frame",
            Positioning::Center,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/copy.ici"))
                .unwrap()
                .0,
            &style.icon_button,
        );
        let mut play_pause = IconButton::new(
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
            Normal,
            (None, Some(40)),
            "0.1",
            &[Decimal],
            &style.text_field,
        );
        let pencil_tool = ToggleIconButton::new(
            (110, BUTTON_Y),
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
        let tools = ToggleIconButtonGroup::new(vec![
            (TOOL_PENCIL, pencil_tool),
            (TOOL_LINE, line_tool),
            (TOOL_RECT, rect_tool),
        ]);
        add_frame.set_state(ElementState::Disabled);
        remove_frame.set_state(ElementState::Disabled);
        copy_frame.set_state(ElementState::Disabled);
        play_pause.set_state(ElementState::Disabled);
        speed.set_state(ElementState::Disabled);
        Box::new(Self {
            result: SceneUpdateResult::Nothing,
            clear,
            tools,
            save,
            save_as,
            close,
            palette,
            speed,
            play_pause,
            add_frame,
            remove_frame,
            copy_frame,
            alert,
            pending_alert_action: None,
            file_name: String::from("untitled"),
        })
    }
}

impl Scene<SceneResult, SceneName> for Editor {
    fn render(&self, graphics: &mut Graphics, mouse_xy: Coord) {
        graphics.clear(BACKGROUND);

        graphics.draw_text(
            &self.file_name,
            TextPos::px(NAME),
            (WHITE, Normal, WrappingStrategy::Ellipsis(38)),
        );
        graphics.draw_line((0, NAME_LINE_Y), (WIDTH as isize, NAME_LINE_Y), LIGHT_GRAY);

        // self.speed.render(graphics, mouse_xy);
        // self.add_frame.render(graphics, mouse_xy);
        // self.remove_frame.render(graphics, mouse_xy);
        // self.copy_frame.render(graphics, mouse_xy);
        // self.play_pause.render(graphics, mouse_xy);
        self.tools.render(graphics, mouse_xy);
        self.save.render(graphics, mouse_xy);
        self.save_as.render(graphics, mouse_xy);
        self.clear.render(graphics, mouse_xy);
        self.close.render(graphics, mouse_xy);
        self.palette.render(graphics, mouse_xy);
    }

    fn on_key_up(&mut self, key: VirtualKeyCode, _: &Vec<&VirtualKeyCode>) {
        self.speed.on_key_press(key);
    }

    fn on_mouse_up(&mut self, xy: Coord, button: MouseButton, _: &Vec<&VirtualKeyCode>) {
        if button != MouseButton::Left {
            return;
        }
        if let Some(tool_id) = self.tools.on_mouse_click(xy) {
            match tool_id {
                TOOL_PENCIL => {}
                TOOL_LINE => {}
                TOOL_RECT => {}
                _ => {}
            }
        }
        if self.add_frame.on_mouse_click(xy) {}
        if self.remove_frame.on_mouse_click(xy) {}
        if self.copy_frame.on_mouse_click(xy) {}
        if self.close.on_mouse_click(xy) {}
        if self.clear.on_mouse_click(xy) {}
        if self.save.on_mouse_click(xy) {}
        if self.save_as.on_mouse_click(xy) {}
        if self.palette.on_mouse_click(xy) {}
    }

    fn on_scroll(&mut self, xy: Coord, y_diff: isize, x_diff: isize, _: &Vec<&VirtualKeyCode>) {}

    fn update(
        &mut self,
        timing: &Timing,
        _: Coord,
        _: &Vec<&VirtualKeyCode>,
    ) -> SceneUpdateResult<SceneResult, SceneName> {
        self.speed.update(timing);
        self.result.clone()
    }

    fn resuming(&mut self, result: Option<SceneResult>) {
        self.result = Nothing;
    }
}
