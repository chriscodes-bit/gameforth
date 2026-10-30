use raylib::prelude::Color;

#[derive(Debug)]
pub enum DrawCommand {
    Rectangle {
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
}

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

    pub fn draw_circle(&mut self, x: i32, y: i32, radius: i32) {
        self.commands.push(DrawCommand::Circle {
            x,
            y,
            radius,
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
