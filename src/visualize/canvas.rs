use egui::{Color32, Pos2, Rect, Vec2};

pub struct Canvas {
    pub size: egui::Vec2,
    pub stroke: egui::Stroke,
    position: egui::Pos2,
}

impl Canvas {
    pub fn new(size: Vec2, stroke: egui::Stroke) -> Canvas {
        Canvas {
            size,
            stroke,
            position: Pos2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
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

    pub fn draw_line(&self, ui: &mut egui::Ui, start: Vec2, end: Vec2) {
        let start = self.position + Vec2::new(0.0, self.size.y) + Vec2::new(start.x, -start.y);
        let end = self.position + Vec2::new(0.0, self.size.y) + Vec2::new(end.x, -end.y);
        ui.painter().line_segment([start, end], self.stroke);
    }

    pub fn draw_circle(&self, ui: &mut egui::Ui, center: Vec2, radius: f32, color: Color32) {
        let center = self.position + center;
        ui.painter().circle(center, radius, color, self.stroke);
    }

    pub fn draw_rectangle(&self, ui: &mut egui::Ui, rect: Rect, color: Color32) {
        let rect = Rect::from_min_size(self.position + Canvas::vec(rect.min), rect.size());
        ui.painter().rect(rect, 0.0, color, self.stroke);
    }

    pub fn draw_rectangle_lines(&self, ui: &mut egui::Ui, rect: Rect) {
        let rect = Rect::from_min_size(self.position + Canvas::vec(rect.min), rect.size());
        ui.painter().rect_stroke(rect, 0.0, self.stroke);
    }
}
