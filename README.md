# GameForth

GameForth is a small Forth-like language embedded in a Rust program that uses raylib for rendering. You write a `.fth` file, the program evaluates it once per frame, and the words in the file push numbers onto a stack and queue drawing commands.

It is not ANS Forth and it is not trying to be. It is a small interpreter built around a tokenizer, a dictionary, a data stack, and a list of draw commands. The goal is a small, stack-based language for making simple 2D things without a lot of setup.

## Status

Early and still changing. The language currently has:

- signed 64-bit integer arithmetic
- stack words such as `dup`, `drop`, `swap`, and `over`
- comparisons and `if` / `else` / `then`
- `begin` / `until` loops
- user-defined words with `: name ... ;`
- global variables with `variable`, `@`, `!`, and `+!`
- filled rectangles and circles
- a handful of named raylib colors
- `.` for printing integers to the terminal

There are no comments, strings, floats, text rendering words, input words, or timing words yet.

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
cargo run
```

It opens an 800x600 window at 60 FPS and loads [`examples/test.fth`](examples/test.fth). Edit that file and restart the program to try something else.

If evaluation fails, the error is printed to stderr as `Forth error: ...` and the loop stops.

## A small program

```forth
350 250 100 100 rl-red draw-rectangle
```

That draws a red `100x100` rectangle with its top-left corner at `(350, 250)`. Since the source runs every frame, the rectangle is redrawn every frame. It stays put until something changes its coordinates.

For animation, keep state in variables:

```forth
variable x
variable initialized

initialized @ 0= if
  0 x !
  -1 initialized !
then

x @ 2 + x !
x @ 700 > if
  0 x !
then

x @ 250 100 100 rl-blue draw-rectangle
```

This moves a blue rectangle to the right and wraps it back to the left edge when it reaches the end.

The current [`examples/test.fth`](examples/test.fth) is a slightly bigger version that bounces diagonally and reverses direction at the screen edges.

## How it fits together

| File | What it does |
| --- | --- |
| `src/lexer.rs` | Splits source into numbers and words |
| `src/stack.rs` | The `i64` data stack |
| `src/dictionary.rs` | Builtin words, color words, user definitions, and variable entries |
| `src/interpreter.rs` | Executes tokens, control flow, definitions, and variables |
| `src/graphics.rs` | Stores the draw command queue and current color |
| `src/main.rs` | Runs the raylib window and frame loop |

The important part to understand is the frame loop:

1. Clear the previous frame's draw commands.
2. Evaluate the whole Forth source file.
3. Render the commands that were queued during evaluation.

The stack, dictionary, variables, and current color survive between frames. The draw command list does not. That is why variables are used for animation and drawing words are called again every frame.

## Language at a glance

| Category | Words |
| --- | --- |
| Arithmetic | `+` `-` `*` `/` |
| Stack | `dup` `drop` `swap` `over` |
| Comparison | `=` `<` `>` `0=` |
| Control flow | `if` `else` `then` `begin` `until` |
| Definitions | `:` `;` |
| Variables | `variable` `@` `!` `+!` |
| Graphics | `draw-rectangle` `draw-circle` |
| Colors | `rl-red` `rl-green` `rl-blue` `rl-white` `rl-black` `rl-yellow` `rl-orange` `rl-purple` `rl-pink` `rl-gray` |
| Output | `.` |

Words are whitespace-separated, case-sensitive, and postfix. Numbers are pushed onto the stack; words consume and produce stack values.

## Documentation

More detailed documentation lives in [`docs/`](docs/README.md):

- [Language guide](docs/language.md)
- [Word reference](docs/words.md)
- [Graphics and animation](docs/graphics.md)
- [Examples](docs/examples.md)
- [Internals](docs/internals.md)

## Tests

```sh
cargo test
cargo fmt -- --check
```

The tests cover variable storage, idempotent variable declarations, `+!`, invalid addresses, and the bouncing example staying inside the window.

## Known limitations

- No comments or string literals.
- No floating-point numbers.
- No text drawing from Forth.
- No keyboard, mouse, or other input.
- No frame counter, time, or random words.
- No sound.
- Only filled rectangles and circles.
- Variables are global.
- The source path is fixed to `examples/test.fth`.
- The language and its syntax are still changing.

## License

No license has been chosen yet.
