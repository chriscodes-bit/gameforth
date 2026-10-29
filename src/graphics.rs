#[derive(Debug)]
pub enum DrawCommand {
    Rectangle {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
}

#[derive(Default)]
pub struct Graphics {
    pub commands: Vec<DrawCommand>,
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
        });
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
