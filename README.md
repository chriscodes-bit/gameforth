# GameForth

GameForth is a small Forth-like language for making simple 2D games and animations with Rust and raylib. You write a `.fth` file, the interpreter runs `init` once, then calls `update` and `draw` every frame. Words push numbers onto a stack and queue drawing commands.

It is not ANS Forth and it is not trying to be. It is one interpreter, a stack of integers and floats, and a list of draw commands. The goal is a small, stack-based language for making simple 2D things without a lot of setup.

## Status

Early and still changing. The language currently has:

- signed 64-bit integers plus first-class floats (`3.14`), with mixed-type promotion
- a math-function set: `exp`, `ln`, `log10`, `log2`, `pow`, `sqrt`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `hypot`, `floor`, `ceil`, `round`, `trunc`
- fixed-point helpers at a scale of `1000` for integer-only integer-ish math
- a full stack vocabulary: `dup`, `drop`, `swap`, `over`, `rot`, `pick`, and return stack words
- comparisons, `if`/`else`/`then`, `begin`/`until`, `begin`/`while`/`repeat`, `do`/`loop`, `?do`/`+loop`
- `leave`, `exit`, `recurse`, and loop indexes `i` and `j`
- line comments (`\`), block comments (`( ... )`), and string literals (`"text"`)
- memory: `variable`, `create`, `allot`, `,`, `constant`, `value`, `to`
- keyboard and mouse input with named key constants
- delta time, elapsed time, and a frame counter
- text rendering, filled and outlined rectangles and circles, and lines
- modular programs with `include "path"`
- errors that report file, line, column, and the call stack

There are no textures, sprites, or sound yet.

## Requirements

- A recent stable Rust toolchain (the crate uses edition 2024)
- The native build tools raylib needs:
  - **Windows:** MSVC build tools (or MinGW), CMake, a C compiler, and LLVM/libclang
  - **macOS:** Xcode command line tools and CMake
  - **Linux:** GCC or Clang, CMake, libclang, and the usual X11/OpenGL development packages

Raylib is built through the `raylib` crate, and its build uses bindgen, so libclang needs to be available. The first build can take a while.

## Running it

From the repository root:

```sh
cargo run                                       # windowed, loads examples/test.fth
cargo run -- examples/demo/main.fth             # the modular starfield demo
cargo run -- --headless --frames 60 examples/test.fth
```

Options:

| Option | Meaning |
| --- | --- |
| `[source.fth]` | Which file to run. Defaults to `examples/test.fth` |
| `--headless` | Run without opening a window |
| `--frames N` | Stop after N frames (headless defaults to 1) |
| `--no-watch` | Disable hot reload |

Windowed mode watches the source file: save it and the program reloads on the next frame. If the new file fails to load, the error is printed and the old program keeps running.

In windowed mode, a Forth error is printed to stderr and the loop stops. In headless mode it is printed and the process exits nonzero.

## A small program

```forth
\ bounce.fth

variable x
variable dx

: init
  0 x !
  4 dx !
;

: update
  dx @ x +!
  x @ 700 > if
    700 x !
    dx @ negate dx !
  then
;

: draw
  x @ 0 100 100 rl-red draw-rectangle
;
```

The file is loaded once: top-level code runs, then `init` runs once. After that, each frame runs `update`, then `draw`, then the queue of draw commands is rendered. Drawing words do not paint immediately; they queue commands, so the last command drawn is the one on top.

## How it fits together

| File | What it does |
| --- | --- |
| `src/lexer.rs` | Splits source into located tokens: numbers, floats, words, strings, comments |
| `src/stack.rs` | The data stack: integers and floats (`Value`) |
| `src/memory.rs` | Cell arena addressed by integers |
| `src/strings.rs` | String pool; Forth code sees integer handles |
| `src/input.rs` | Keyboard and mouse snapshot for the current frame |
| `src/time.rs` | Delta time, elapsed time, frame counter |
| `src/rng.rs` | Xorshift random number generator |
| `src/dictionary.rs` | Builtin words, colors, constants |
| `src/control.rs` | Precomputed jump targets for control flow |
| `src/error.rs` | Source locations and call stacks |
| `src/context.rs` | Shared runtime state passed to every builtin |
| `src/interpreter.rs` | Execution, lifecycle, includes, loops, diagnostics |
| `src/graphics.rs` | Draw command queue and current color |
| `src/main.rs` | CLI, windowed/headless loop, hot reload, input snapshot, rendering |

## Lifecycle in one paragraph

Loading a file tokenizes it, executes the top-level code once (definitions, variables, includes), then calls `init` if it is defined. Every frame after that clears the draw list, calls `update`, calls `draw`, and renders the queue. The stack, memory, dictionary, string pool, and current color persist between frames. The draw command list does not.

## Documentation

More detailed documentation lives in [`docs/`](docs/README.md):

- [Language guide](docs/language.md) — syntax, lifecycle, control flow, memory, strings, input, time, includes, errors
- [Word reference](docs/words.md) — every word with stack effects
- [Graphics and animation](docs/graphics.md) — coordinates, draw commands, colors, text
- [Examples](docs/examples.md) — the bounce example and the modular demo, walkthroughs
- [Internals](docs/internals.md) — module layout, architecture, extension points

## Tests

```sh
cargo test
cargo fmt -- --check
```

The tests cover the stack and arithmetic words, floats and mixed-arithmetic promotion, the math functions, comparisons, control flow (`do`/`loop`, `leave`, `while`/`repeat`, `exit`, `recurse`), variables, values and constants, memory arrays, fixed-point math, strings and text commands, comments, the `init`/`update`/`draw` lifecycle, diagnostics with locations and call stacks, and `include` with relative paths and cycle detection. The bounce example is run for 500 frames and checked to stay in bounds.

## Known limitations

- Floats print with Rust's `{}` formatting (`0.1 0.1 + .` shows `0.2` but also traces like `0.30000000000000004`).
- Mixed int/float comparisons promote to `f64`, so integers beyond 2^53 lose exactness against floats.
- `text-width` is approximate (`chars * size / 2`), not measured with the real font.
- No textures, sprites, or images.
- No sound.
- Only one global dictionary and one global stack; no locals.
- Hot reload restarts the whole interpreter, so state resets on every save.
- The language and its syntax are still changing.

## License

No license has been chosen yet.
