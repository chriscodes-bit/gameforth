# GameForth Documentation

GameForth is a small, Forth-like, stack-based language embedded in a Rust + raylib application. A Forth source file is evaluated once per frame, so a program can update persistent state and queue 2D drawing commands for that frame.

This folder contains the complete documentation for the language as it currently exists.

## Quick start

Requirements:

- A working Rust toolchain
- The native build prerequisites for raylib (a C compiler, CMake, etc.)

Run the application from the repository root:

```powershell
cargo run
```

The application:

- loads `examples/test.fth` at startup
- opens an 800x600 window titled `GameForth`
- targets 60 frames per second
- clears the background to RGB `(24, 26, 36)`
- evaluates the Forth source once per frame
- renders the draw commands produced by that evaluation
- prints `Forth error: ...` to stderr and stops if evaluation fails

To try another program, edit `examples/test.fth` and restart the application.

## Smallest useful program

```forth
350 250 100 100 rl-red draw-rectangle
```

This draws a filled red rectangle whose top-left corner is at `(350, 250)` and whose size is `100x100`.

Because the source is evaluated every frame, the rectangle is redrawn every frame. Without variables or another source of changing state, it stays in the same place.

## At a glance

| Property | Current behavior |
| --- | --- |
| Language style | Postfix, stack-based, Forth-like |
| Number type | Signed 64-bit integer (`i64`) |
| Tokenization | Whitespace-separated tokens |
| Case sensitivity | Case-sensitive |
| Comments | Not supported |
| Strings | Not supported |
| Floating point | Not supported |
| Persistent data | Data stack, dictionary, variables |
| Per-frame data | Draw command queue (cleared each frame) |
| Persistent drawing state | Current color |
| Text output | `.` prints an integer to stdout |
| Graphics | Filled rectangles and filled circles |
| Window | 800x600 at 60 FPS |
| Error handling | Evaluation stops on the first error |

## Documentation map

- [Language guide](language.md) — syntax, stack model, arithmetic, control flow, definitions, variables, execution lifecycle, errors, and limitations.
- [Word reference](words.md) — every builtin, special form, color word, and variable operation, with stack effects.
- [Graphics and animation](graphics.md) — coordinate system, draw commands, color state, layering, and animation patterns.
- [Examples](examples.md) — complete programs ranging from a single rectangle to a bouncing rectangle.
- [Internals](internals.md) — source layout, interpreter architecture, data structures, and extension points.

## Execution model in one paragraph

The application reads the source file once, then repeatedly executes the entire token stream. Between frames, the interpreter's stack, dictionary, and variables survive, but the graphics command list is cleared before each evaluation. Drawing words do not paint immediately; they append commands to a queue. After evaluation finishes, the renderer walks that queue and draws each command in order using the color that was current when the command was queued. This makes the source behave like a frame script: state changes persist, while drawing commands are rebuilt from scratch every frame.

## Example: a bouncing rectangle

The repository's current `examples/test.fth` is a complete animation example:

```forth
variable x
variable y
variable dx
variable dy
variable initialized

initialized @ 0= if
  350 x !
  250 y !
  4 dx !
  3 dy !
  -1 initialized !
then

dx @ x +!
dy @ y +!

x @ 0 < if
  0 x !
  -1 dx @ * dx !
then

x @ 700 > if
  700 x !
  -1 dx @ * dx !
then

y @ 0 < if
  0 y !
  -1 dy @ * dy !
then

y @ 500 > if
  500 y !
  -1 dy @ * dy !
then

x @ y @ 100 100 rl-red draw-rectangle
```

See [Examples](examples.md#bouncing-rectangle) for a line-by-line explanation.

## What GameForth is not

GameForth is not a complete ANS Forth implementation. It does not currently provide comments, strings, floating point, text rendering from Forth, input, timing, randomness, sound, a return stack, locals, constants, or general memory allocation. See [Limitations](language.md#limitations) for the full list.
