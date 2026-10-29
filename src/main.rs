mod dictionary;
mod graphics;
mod interpreter;
mod lexer;
mod stack;

use raylib::prelude::*;

fn main() {
    let source = std::fs::read_to_string("examples/test.fth").expect("Could not read Forth file");

    let mut interpreter = interpreter::Interpreter::new();

    let (mut rl, thread) = raylib::init().size(800, 600).title("GameForth").build();

    rl.set_target_fps(60);

    while !rl.window_should_close() {
        interpreter.graphics.clear();

        if let Err(err) = interpreter.eval(&source) {
            eprintln!("Forth error: {}", err);
            break;
        }

        let mut draw = rl.begin_drawing(&thread);
        draw.clear_background(Color::new(24, 26, 36, 255));

        for command in &interpreter.graphics.commands {
            match command {
                graphics::DrawCommand::Rectangle {
                    x,
                    y,
                    width,
                    height,
                } => {
                    draw.draw_rectangle(*x, *y, *width, *height, Color::GREEN);
                }
            }
        }
    }
}
