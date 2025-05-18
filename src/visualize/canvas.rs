use bevy_egui::egui::{Color32, Pos2, Rect, Vec2};

pub struct Canvas {
    pub size: bevy_egui::egui::Vec2,
    pub stroke: bevy_egui::egui::Stroke,
    position: bevy_egui::egui::Pos2,
}

impl Canvas {
    pub fn new(size: Vec2, stroke: bevy_egui::egui::Stroke) -> Canvas {
        Canvas {
            size,
            stroke,
            position: Pos2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn draw(&mut self, ui: &mut bevy_egui::egui::Ui) {
        bevy_egui::egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            self.position = ui
                .allocate_ui(self.size, |ui| {
                    let (_id, rect) = ui.allocate_space(self.size);
                    rect
                })
                .inner
                .min;
        });
    }

    fn vec(pos: Pos2) -> Vec2 {
        Vec2::new(pos.x, pos.y)
    }

    pub fn draw_line(&self, ui: &mut bevy_egui::egui::Ui, start: Vec2, end: Vec2) {
        let start = self.position + Vec2::new(0.0, self.size.y) + Vec2::new(start.x, -start.y);
        let end = self.position + Vec2::new(0.0, self.size.y) + Vec2::new(end.x, -end.y);
        ui.painter().line_segment([start, end], self.stroke);
    }

    pub fn draw_circle(&self, ui: &mut bevy_egui::egui::Ui, center: Vec2, radius: f32, color: Color32) {
        let center = self.position + center;
        ui.painter().circle(center, radius, color, self.stroke);
    }

    pub fn draw_rectangle(&self, ui: &mut bevy_egui::egui::Ui, rect: Rect, color: Color32) {
        let rect = Rect::from_min_size(self.position + Canvas::vec(rect.min), rect.size());
        ui.painter().rect(rect, 0.0, color, self.stroke);
    }

    pub fn draw_rectangle_lines(&self, ui: &mut bevy_egui::egui::Ui, rect: Rect) {
        let rect = Rect::from_min_size(self.position + Canvas::vec(rect.min), rect.size());
        ui.painter().rect_stroke(rect, 0.0, self.stroke);
    }
}
