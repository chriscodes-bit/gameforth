use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init().size(800, 600).title("GameForth").build();

    rl.set_target_fps(60);

    while !rl.window_should_close() {
        let mut draw = rl.begin_drawing(&thread);

        draw.clear_background(Color::new(24, 26, 36, 255));

        draw.draw_text("GameForth", 20, 20, 32, Color::RAYWHITE);

        draw.draw_rectangle(350, 250, 100, 100, Color::GREEN);
    }
}
