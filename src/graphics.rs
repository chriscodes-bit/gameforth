use raylib::prelude::Color;

#[derive(Debug, Clone)]
pub enum DrawCommand {
    Rectangle {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Color,
    },
    RectangleLines {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Color,
    },
    Circle {
        x: i32,
        y: i32,
        radius: i32,
        color: Color,
    },
    CircleLines {
        x: i32,
        y: i32,
        radius: i32,
        color: Color,
    },
    Line {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: Color,
    },
    Text {
        text: String,
        x: i32,
        y: i32,
        size: i32,
        color: Color,
    },
}

#[derive(Debug, Clone)]
pub struct Graphics {
    pub commands: Vec<DrawCommand>,
    pub current_color: Color,
}

impl Default for Graphics {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            current_color: Color::WHITE,
        }
    }
}

impl Graphics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw_rectangle(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.commands.push(DrawCommand::Rectangle {
            x,
            y,
            width,
            height,
            color: self.current_color,
        });
    }

    pub fn draw_rectangle_lines(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.commands.push(DrawCommand::RectangleLines {
            x,
            y,
            width,
            height,
            color: self.current_color,
        });
    }

    pub fn draw_circle(&mut self, x: i32, y: i32, radius: i32) {
        self.commands.push(DrawCommand::Circle {
            x,
            y,
            radius,
            color: self.current_color,
        });
    }

    pub fn draw_circle_lines(&mut self, x: i32, y: i32, radius: i32) {
        self.commands.push(DrawCommand::CircleLines {
            x,
            y,
            radius,
            color: self.current_color,
        });
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.commands.push(DrawCommand::Line {
            x1,
            y1,
            x2,
            y2,
            color: self.current_color,
        });
    }

    pub fn draw_text(&mut self, text: String, x: i32, y: i32, size: i32) {
        self.commands.push(DrawCommand::Text {
            text,
            x,
            y,
            size,
            color: self.current_color,
        });
    }

    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
