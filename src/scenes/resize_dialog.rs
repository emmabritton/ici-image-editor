use crate::{SceneName, SceneResult, Settings, HEIGHT, SUR, WIDTH};
use pixels_graphics_lib::prelude::SceneUpdateResult::*;
use pixels_graphics_lib::prelude::*;
use pixels_graphics_lib::ui::layout::relative::LayoutContext;
use pixels_graphics_lib::ui::prelude::*;
use pixels_graphics_lib::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResizeAnchor {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ResizeAnchor {
    pub fn is_left(&self) -> bool {
        matches!(self, ResizeAnchor::TopLeft | ResizeAnchor::BottomLeft)
    }

    pub fn is_right(&self) -> bool {
        matches!(self, ResizeAnchor::TopRight | ResizeAnchor::BottomRight)
    }

    pub fn is_top(&self) -> bool {
        matches!(self, ResizeAnchor::TopLeft | ResizeAnchor::TopRight)
    }

    pub fn is_bottom(&self) -> bool {
        matches!(self, ResizeAnchor::BottomLeft | ResizeAnchor::BottomRight)
    }
}

pub struct ResizeDialog {
    result: SUR,
    bg: ShapeCollection,
    title: Label,
    current_size: Label,
    new_size: Label,
    anchor: Label,
    x: Label,
    width: TextField,
    height: TextField,
    cancel: Button,
    submit: Button,
    anchors: ToggleIconButtonGroup<ResizeAnchor>,
    alert: Option<Alert>,
    alert_style: AlertStyle,
    prefs: AppPrefs<Settings>,
}

impl ResizeDialog {
    pub fn new(size: (u8, u8), prefs: AppPrefs<Settings>, style: &UiStyle) -> Box<Self> {
        let mut icon_button_style = style.toggle_icon_button.clone();
        icon_button_style.rounding = 0;

        let bg = dialog_background(WIDTH, HEIGHT, &style.dialog);
        let mut title = Label::singleline(
            "Resize canvas",
            (0, 0),
            WHITE,
            PixelFont::Standard8x10,
            style.dialog.bounds.width(),
        );
        let mut current_size = Label::singleline(
            &format!("Current size {}x{}", size.0, size.1),
            (0, 0),
            WHITE,
            PixelFont::Standard6x7,
            style.dialog.bounds.width(),
        );
        let mut new_size = Label::singleline(
            "New size",
            (0, 0),
            WHITE,
            PixelFont::Standard6x7,
            style.dialog.bounds.width(),
        );
        let mut anchor = Label::singleline(
            "Anchor",
            (0, 0),
            WHITE,
            PixelFont::Standard6x7,
            style.dialog.bounds.width(),
        );
        let mut x = Label::singleline(
            "x",
            (0, 0),
            WHITE,
            PixelFont::Standard6x7,
            style.dialog.bounds.width(),
        );
        let mut cancel = Button::new((0, 0), "Cancel", Some(80), &style.button);
        let mut submit = Button::new((0, 0), "Submit", Some(80), &style.button);
        let mut width = TextField::new(
            (0, 0),
            3,
            PixelFont::Standard6x7,
            (None, None),
            &size.0.to_string(),
            &[TextFilter::Numbers],
            &style.text_field,
        );
        let mut height = TextField::new(
            (0, 0),
            3,
            PixelFont::Standard6x7,
            (None, None),
            &size.1.to_string(),
            &[TextFilter::Numbers],
            &style.text_field,
        );
        let mut anchor_tl = ToggleIconButton::new(
            (0, 0),
            "Top left",
            Positioning::RightBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/anchor_tl.ici"))
                .unwrap()
                .0,
            &icon_button_style,
        );
        let mut anchor_tr = ToggleIconButton::new(
            (0, 0),
            "Top right",
            Positioning::LeftBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/anchor_tr.ici"))
                .unwrap()
                .0,
            &icon_button_style,
        );
        let mut anchor_bl = ToggleIconButton::new(
            (0, 0),
            "Bottom left",
            Positioning::RightBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/anchor_bl.ici"))
                .unwrap()
                .0,
            &icon_button_style,
        );
        let mut anchor_br = ToggleIconButton::new(
            (0, 0),
            "Bottom right",
            Positioning::LeftBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/anchor_br.ici"))
                .unwrap()
                .0,
            &icon_button_style,
        );
        let mut anchor_c = ToggleIconButton::new(
            (0, 0),
            "Center",
            Positioning::RightBottom,
            IndexedImage::from_file_contents(include_bytes!("../../assets/icons/anchor_c.ici"))
                .unwrap()
                .0,
            &icon_button_style,
        );

        let context = LayoutContext::new(style.dialog.bounds.clone());

        layout!(context, title, align_top, px!(8));
        layout!(context, title, align_centerh);

        layout!(context, current_size, align_left, px!(6));
        layout!(context, current_size, top_to_bottom_of title, px!(4));

        layout!(context, new_size, align_left, px!(6));
        layout!(context, new_size, top_to_bottom_of current_size, px!(16));

        layout!(context, width, align_left, px!(6));
        layout!(context, width, top_to_bottom_of new_size, px!(4));

        layout!(context, x, bottom_to_bottom_of width);
        layout!(context, x, left_to_right_of  width);

        layout!(context, height, bottom_to_bottom_of x);
        layout!(context, height, left_to_right_of  x);

        layout!(context, anchor, top_to_top_of  new_size);
        layout!(context, anchor, left_to_right_of  height, px!(24));

        layout!(context, submit, align_bottom, px!(4));
        layout!(context, cancel, align_bottom, px!(4));
        layout!(context, submit, align_right, px!(4));
        layout!(context, cancel, align_left, px!(4));

        layout!(context, anchor_tl, left_to_left_of anchor);
        layout!(context, anchor_tl, top_to_bottom_of  anchor);

        layout!(context, anchor_c, left_to_right_of  anchor_tl);
        layout!(context, anchor_c, top_to_bottom_of   anchor_tl);

        layout!(context, anchor_tr, left_to_right_of  anchor_c);
        layout!(context, anchor_tr, top_to_top_of   anchor_tl);

        layout!(context, anchor_bl, top_to_bottom_of   anchor_c);
        layout!(context, anchor_bl, left_to_left_of    anchor_tl);

        layout!(context, anchor_br, left_to_right_of    anchor_c);
        layout!(context, anchor_br, top_to_top_of     anchor_bl);

        let mut anchors = ToggleIconButtonGroup::new(vec![
            (ResizeAnchor::TopLeft, anchor_tl),
            (ResizeAnchor::BottomLeft, anchor_bl),
            (ResizeAnchor::TopRight, anchor_tr),
            (ResizeAnchor::BottomRight, anchor_br),
            (ResizeAnchor::Center, anchor_c),
        ]);

        anchors.set_selected(prefs.data.last_used_anchor);

        Box::new(ResizeDialog {
            result: Nothing,
            bg,
            title,
            current_size,
            width,
            height,
            cancel,
            submit,
            anchor,
            alert: None,
            new_size,
            x,
            anchors,
            alert_style: style.alert.clone(),
            prefs,
        })
    }
}

impl ResizeDialog {
    fn verify(&mut self) {
        let alert = Some(Alert::new_warning(
            &["Invalid width/height, both", "must be between 1 and 64"],
            WIDTH,
            HEIGHT,
            &self.alert_style,
        ));
        let anchor = self.anchors.get_selected();
        let w = self.width.content().parse::<u8>();
        let h = self.height.content().parse::<u8>();
        if w.is_err() || h.is_err() {
            self.alert = alert;
        } else {
            let w = w.unwrap();
            let h = h.unwrap();
            if w == 0 || h == 0 || w > 64 || h > 64 {
                self.alert = alert;
            } else {
                self.prefs.data.last_used_anchor = *anchor;
                self.prefs.save();
                self.result = Pop(Some(SceneResult::ResizeData(w, h, *anchor)));
            }
        }
    }
}

impl Scene<SceneResult, SceneName> for ResizeDialog {
    fn render(&self, graphics: &mut Graphics, mouse: &MouseData, _: &FxHashSet<KeyCode>) {
        self.bg.render(graphics);
        render!(
            graphics,
            mouse,
            self.width,
            self.height,
            self.anchor,
            self.submit,
            self.anchors,
            self.cancel,
            self.title,
            self.x,
            self.current_size,
            self.new_size
        );
        if let Some(alert) = &self.alert {
            alert.render(graphics, mouse);
        }
    }

    fn on_key_up(&mut self, key: KeyCode, _: &MouseData, held: &FxHashSet<KeyCode>) {
        self.width.on_key_press(key, held);
        self.height.on_key_press(key, held);
        if key == KeyCode::Escape {
            self.result = Pop(None);
        }
        if key == KeyCode::Tab && self.width.is_focused() {
            self.width.unfocus();
            self.height.focus();
        }
        if key == KeyCode::Tab
            && (held.contains(&KeyCode::ShiftLeft) || held.contains(&KeyCode::ShiftRight))
            && self.height.is_focused()
        {
            self.width.focus();
            self.height.unfocus();
        }
    }

    fn on_mouse_click(
        &mut self,
        down_at: Coord,
        mouse: &MouseData,
        mouse_button: MouseButton,
        _: &FxHashSet<KeyCode>,
    ) {
        if mouse_button == MouseButton::Left {
            if let Some(alert) = &mut self.alert {
                if alert.on_mouse_click(down_at, mouse.xy).is_some() {
                    self.alert = None;
                }
            } else {
                self.width.on_mouse_click(down_at, mouse.xy);
                self.height.on_mouse_click(down_at, mouse.xy);
                self.anchors.on_mouse_click(down_at, mouse.xy);
                if self.submit.on_mouse_click(down_at, mouse.xy) {
                    self.verify();
                }
                if self.cancel.on_mouse_click(down_at, mouse.xy) {
                    self.result = Pop(None);
                }
            }
        }
    }

    fn update(
        &mut self,
        timing: &Timing,
        _: &MouseData,
        _: &FxHashSet<KeyCode>,
    ) -> SceneUpdateResult<SceneResult, SceneName> {
        self.width.update(timing);
        self.height.update(timing);

        self.result.clone()
    }

    fn is_dialog(&self) -> bool {
        true
    }
}
