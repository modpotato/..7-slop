#![forbid(unsafe_code)]

extern crate alloc;

use alloc::boxed::Box;
#[cfg(feature = "text")]
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub [f32; 4]);

pub trait RenderTarget {
    fn quad(&mut self, bounds: Rect, color: Color);

    #[cfg(feature = "text")]
    fn text(&mut self, _position: [f32; 2], _content: &str, _color: Color) {}
}

#[cfg(feature = "input")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Click { x: f32, y: f32 },
    Move { x: f32, y: f32 },
}

pub trait Widget {
    fn id(&self) -> WidgetId;
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);
    fn draw(&self, target: &mut dyn RenderTarget);

    #[cfg(feature = "input")]
    fn on_input(&mut self, _event: &InputEvent) -> bool {
        false
    }

    #[cfg(feature = "text")]
    fn text_content(&self) -> Option<&str> {
        None
    }
}

#[cfg(feature = "layout")]
pub trait LayoutEngine {
    fn layout(&self, index: usize, current: Rect) -> Rect;
}

pub struct Ui {
    widgets: Vec<Box<dyn Widget>>,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    pub const fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    pub fn push<W: Widget + 'static>(&mut self, widget: W) {
        self.widgets.push(Box::new(widget));
    }

    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    pub fn render(&self, target: &mut dyn RenderTarget) {
        for widget in &self.widgets {
            widget.draw(target);
        }
    }

    pub fn widget_bounds(&self, index: usize) -> Option<Rect> {
        self.widgets.get(index).map(|widget| widget.bounds())
    }

    #[cfg(feature = "layout")]
    pub fn relayout(&mut self, engine: &dyn LayoutEngine) {
        for (index, widget) in self.widgets.iter_mut().enumerate() {
            let next = engine.layout(index, widget.bounds());
            widget.set_bounds(next);
        }
    }

    #[cfg(feature = "input")]
    pub fn dispatch_input(&mut self, event: InputEvent) -> Option<WidgetId> {
        for widget in self.widgets.iter_mut().rev() {
            if let InputEvent::Click { x, y } = event {
                if !widget.bounds().contains(x, y) {
                    continue;
                }
            }

            if widget.on_input(&event) {
                return Some(widget.id());
            }
        }

        None
    }

    #[cfg(feature = "text")]
    pub fn collect_text(&self) -> Vec<(WidgetId, String)> {
        self.widgets
            .iter()
            .filter_map(|widget| widget.text_content().map(|value| (widget.id(), value.into())))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidget {
        id: WidgetId,
        bounds: Rect,
        color: Color,
        #[cfg(feature = "input")]
        captured: bool,
        #[cfg(feature = "text")]
        text: Option<&'static str>,
    }

    impl Widget for TestWidget {
        fn id(&self) -> WidgetId {
            self.id
        }

        fn bounds(&self) -> Rect {
            self.bounds
        }

        fn set_bounds(&mut self, bounds: Rect) {
            self.bounds = bounds;
        }

        fn draw(&self, target: &mut dyn RenderTarget) {
            target.quad(self.bounds, self.color);
            #[cfg(feature = "text")]
            if let Some(text) = self.text {
                target.text([self.bounds.x, self.bounds.y], text, self.color);
            }
        }

        #[cfg(feature = "input")]
        fn on_input(&mut self, _event: &InputEvent) -> bool {
            self.captured
        }

        #[cfg(feature = "text")]
        fn text_content(&self) -> Option<&str> {
            self.text
        }
    }

    #[derive(Default)]
    struct TestTarget {
        quads: usize,
        #[cfg(feature = "text")]
        texts: usize,
    }

    impl RenderTarget for TestTarget {
        fn quad(&mut self, _bounds: Rect, _color: Color) {
            self.quads += 1;
        }

        #[cfg(feature = "text")]
        fn text(&mut self, _position: [f32; 2], _content: &str, _color: Color) {
            self.texts += 1;
        }
    }

    #[test]
    fn renders_all_widgets() {
        let mut ui = Ui::new();
        ui.push(TestWidget {
            id: WidgetId(1),
            bounds: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color([1.0, 0.0, 0.0, 1.0]),
            #[cfg(feature = "input")]
            captured: false,
            #[cfg(feature = "text")]
            text: None,
        });
        ui.push(TestWidget {
            id: WidgetId(2),
            bounds: Rect::new(10.0, 10.0, 10.0, 10.0),
            color: Color([0.0, 1.0, 0.0, 1.0]),
            #[cfg(feature = "input")]
            captured: false,
            #[cfg(feature = "text")]
            text: None,
        });

        let mut target = TestTarget::default();
        ui.render(&mut target);

        assert_eq!(target.quads, 2);
    }

    #[cfg(feature = "layout")]
    #[test]
    fn relayout_updates_widget_bounds() {
        struct ShiftLayout;

        impl LayoutEngine for ShiftLayout {
            fn layout(&self, index: usize, current: Rect) -> Rect {
                Rect::new(current.x + index as f32 + 1.0, current.y, current.width, current.height)
            }
        }

        let mut ui = Ui::new();
        ui.push(TestWidget {
            id: WidgetId(1),
            bounds: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color([1.0, 1.0, 1.0, 1.0]),
            #[cfg(feature = "input")]
            captured: false,
            #[cfg(feature = "text")]
            text: None,
        });

        ui.relayout(&ShiftLayout);
        assert_eq!(
            ui.widget_bounds(0),
            Some(Rect::new(1.0, 0.0, 10.0, 10.0))
        );
    }

    #[cfg(feature = "input")]
    #[test]
    fn input_dispatch_uses_topmost_hit_widget() {
        let mut ui = Ui::new();
        ui.push(TestWidget {
            id: WidgetId(1),
            bounds: Rect::new(0.0, 0.0, 20.0, 20.0),
            color: Color([0.0, 0.0, 0.0, 1.0]),
            captured: false,
            #[cfg(feature = "text")]
            text: None,
        });
        ui.push(TestWidget {
            id: WidgetId(2),
            bounds: Rect::new(0.0, 0.0, 20.0, 20.0),
            color: Color([0.0, 0.0, 0.0, 1.0]),
            captured: true,
            #[cfg(feature = "text")]
            text: None,
        });

        let handled = ui.dispatch_input(InputEvent::Click { x: 5.0, y: 5.0 });
        assert_eq!(handled, Some(WidgetId(2)));
    }

    #[cfg(feature = "text")]
    #[test]
    fn collect_text_includes_text_widgets() {
        let mut ui = Ui::new();
        ui.push(TestWidget {
            id: WidgetId(7),
            bounds: Rect::new(0.0, 0.0, 5.0, 5.0),
            color: Color([1.0, 1.0, 1.0, 1.0]),
            #[cfg(feature = "input")]
            captured: false,
            text: Some("hello"),
        });

        let all_text = ui.collect_text();
        assert_eq!(all_text, vec![(WidgetId(7), String::from("hello"))]);
    }
}
