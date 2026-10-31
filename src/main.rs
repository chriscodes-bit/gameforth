use std::path::{Path, PathBuf};
use std::time::SystemTime;

use raylib::consts::MouseButton;
use raylib::core::input::key_from_i32;
use raylib::prelude::*;

use gameforth::graphics::DrawCommand;
use gameforth::input::{InputState, TRACKED_KEYS};
use gameforth::interpreter::Interpreter;

struct Options {
    source: PathBuf,
    headless: bool,
    frames: Option<u64>,
    watch: bool,
}

fn main() {
    let options = match parse_args() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("error: {}", error);
            print_help();
            std::process::exit(2);
        }
    };

    let result = if options.headless {
        run_headless(&options)
    } else {
        run_windowed(&options)
    };

    if let Err(error) = result {
        eprintln!("error: {}", error);
        std::process::exit(1);
    }
}

fn parse_args() -> Result<Options, String> {
    let mut source = PathBuf::from("examples/demo/main.fth");
    let mut headless = false;
    let mut frames = None;
    let mut watch = true;

    let mut args = std::env::args().skip(1);

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--headless" => headless = true,
            "--no-watch" => watch = false,
            "--frames" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--frames requires a number".to_string())?;
                frames = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| format!("invalid frame count: {}", value))?,
                );
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option: {}", other));
            }
            other => source = PathBuf::from(other),
        }
    }

    if headless && frames.is_none() {
        frames = Some(1);
    }

    Ok(Options {
        source,
        headless,
        frames,
        watch,
    })
}

fn print_help() {
    println!(
        "GameForth\n\
         \n\
         Usage: gameforth [source.fth] [options]\n\
         \n\
         Options:\n\
           --headless     Run without opening a window\n\
           --frames N     Stop after N frames (headless defaults to 1)\n\
           --no-watch     Disable hot reload\n\
           -h, --help     Show this help"
    );
}

fn load_interpreter(path: &Path) -> Result<Interpreter, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read '{}': {}", path.display(), error))?;

    let mut interpreter = Interpreter::new();
    interpreter
        .load(&source, Some(path.to_path_buf()))
        .map_err(|error| error.to_string())?;

    Ok(interpreter)
}

fn run_headless(options: &Options) -> Result<(), String> {
    let mut interpreter = load_interpreter(&options.source)?;
    let frames = options.frames.unwrap_or(1);
    let dt = 16;

    for frame in 0..frames {
        interpreter.set_time(dt, (frame as i64 + 1) * dt, frame as i64);
        interpreter.set_input(InputState::default());
        interpreter.run_frame().map_err(|error| error.to_string())?;
    }

    println!("ran {} frame(s)", frames);
    Ok(())
}

fn run_windowed(options: &Options) -> Result<(), String> {
    let mut interpreter = load_interpreter(&options.source)?;

    let (mut rl, thread) = raylib::init().size(800, 600).title("GameForth").build();

    rl.set_target_fps(60);

    let mut last_modified = modified_time(&options.source);
    let mut last_elapsed = 0.0;
    let mut frame = 0i64;

    while !rl.window_should_close() {
        if options.watch {
            if let Some(modified) = modified_time(&options.source) {
                if Some(modified) != last_modified {
                    last_modified = Some(modified);

                    match load_interpreter(&options.source) {
                        Ok(new_interpreter) => {
                            interpreter = new_interpreter;
                            println!("reloaded {}", options.source.display());
                        }
                        Err(error) => eprintln!("reload failed: {}", error),
                    }
                }
            }
        }

        let elapsed = rl.get_time();
        let dt = if last_elapsed == 0.0 {
            0
        } else {
            ((elapsed - last_elapsed) * 1000.0).round() as i64
        };
        last_elapsed = elapsed;

        interpreter.set_time(dt, (elapsed * 1000.0) as i64, frame);
        interpreter.set_input(snapshot_input(&mut rl));

        if let Err(error) = interpreter.run_frame() {
            eprintln!("Forth error: {}", error);
            break;
        }

        let mut draw = rl.begin_drawing(&thread);
        draw.clear_background(Color::new(24, 26, 36, 255));

        for command in &interpreter.context.graphics.commands {
            match command {
                DrawCommand::Rectangle {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => draw.draw_rectangle(*x, *y, *width, *height, *color),

                DrawCommand::RectangleLines {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => draw.draw_rectangle_lines(*x, *y, *width, *height, *color),

                DrawCommand::Circle {
                    x,
                    y,
                    radius,
                    color,
                } => draw.draw_circle(*x, *y, *radius as f32, *color),

                DrawCommand::CircleLines {
                    x,
                    y,
                    radius,
                    color,
                } => draw.draw_circle_lines(*x, *y, *radius as f32, *color),

                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    color,
                } => draw.draw_line(*x1, *y1, *x2, *y2, *color),

                DrawCommand::Text {
                    text,
                    x,
                    y,
                    size,
                    color,
                } => draw.draw_text(text, *x, *y, *size, *color),
            }
        }

        frame += 1;
    }

    Ok(())
}

fn snapshot_input(rl: &mut RaylibHandle) -> InputState {
    let mut input = InputState::default();

    for &code in TRACKED_KEYS {
        if let Some(key) = key_from_i32(code) {
            if rl.is_key_down(key) {
                input.keys_down.insert(code);
            }

            if rl.is_key_pressed(key) {
                input.keys_pressed.insert(code);
            }
        }
    }

    let mouse = rl.get_mouse_position();
    input.mouse_x = mouse.x as i32;
    input.mouse_y = mouse.y as i32;
    input.mouse_wheel = rl.get_mouse_wheel_move() as i32;

    let buttons = [
        (0usize, MouseButton::MOUSE_BUTTON_LEFT),
        (1, MouseButton::MOUSE_BUTTON_RIGHT),
        (2, MouseButton::MOUSE_BUTTON_MIDDLE),
        (3, MouseButton::MOUSE_BUTTON_SIDE),
        (4, MouseButton::MOUSE_BUTTON_EXTRA),
        (5, MouseButton::MOUSE_BUTTON_FORWARD),
        (6, MouseButton::MOUSE_BUTTON_BACK),
    ];

    for (index, button) in buttons {
        if index < input.mouse_buttons_down.len() {
            input.mouse_buttons_down[index] = rl.is_mouse_button_down(button);
            input.mouse_buttons_pressed[index] = rl.is_mouse_button_pressed(button);
        }
    }

    input
}

fn modified_time(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}
