# GameForth Documentation

GameForth is a small, Forth-like, stack-based language embedded in a Rust + raylib application. A program is loaded once: the top-level runs, then `init` runs once. After that, the host calls `update` and `draw` every frame, and the words executed there push numbers onto a stack and queue 2D drawing commands.

This folder contains the complete documentation for the language as it currently exists.

## Quick start

Requirements:

- A working Rust toolchain
- The native build prerequisites for raylib (a C compiler, CMake, libclang, etc.)

Run from the repository root:

```sh
cargo run                                  # windowed, loads examples/test.fth
cargo run -- examples/demo/main.fth        # the modular demo
cargo run -- --headless --frames 10 examples/test.fth
```

The application:

- loads the source file you pass on the command line (default: `examples/test.fth`)
- opens an 800x600 window titled `GameForth` at 60 FPS, unless `--headless`
- runs the top-level code and `init` once at load
- every frame: clears the draw queue, runs `update`, runs `draw`, renders the queue
- clears the background to RGB `(24, 26, 36)` in windowed mode
- watches the source file and reloads it when it changes (unless `--no-watch`)

On a Forth error, windowed mode prints the error to stderr and stops; headless mode prints it and exits nonzero.

## Smallest useful program

```forth
350 250 100 100 rl-red draw-rectangle
```

Top-level code runs once, but draw commands live in a queue that is rebuilt every frame, so a shape drawn only at the top level disappears after the first frame unless `draw` draws it. Per-frame drawing belongs in `draw`:

```forth
: draw 350 250 100 100 rl-red draw-rectangle ;
```

## CLI reference

```text
gameforth [source.fth] [--headless] [--frames N] [--no-watch]
```

| Option | Meaning |
| --- | --- |
| `[source.fth]` | Source file to run (default `examples/test.fth`) |
| `--headless` | Run without a window |
| `--frames N` | Stop after N frames; headless defaults to 1 |
| `--no-watch` | Disable hot reload |
| `-h`, `--help` | Print usage |

Headless frames use a fixed delta time of 16 ms and a default (empty) input state.

## Documentation map

- [Language guide](language.md) — syntax, lifecycle, control flow, memory, strings, input, time, includes, errors.
- [Word reference](words.md) — every builtin, special form, color, and constant, with stack effects.
- [Graphics and animation](graphics.md) — coordinate system, draw commands, color state, text, and animation patterns.
- [Examples](examples.md) — the bouncing rectangle and a modular starfield demo, walkthroughs of both.
- [Internals](internals.md) — module layout, interpreter architecture, data structures, and extension points.

## At a glance

| Property | Current behavior |
| --- | --- |
| Language style | Postfix, stack-based, Forth-like |
| Number type | Signed 64-bit integer (`i64`) and 64-bit float (`f64`) literals like `3.14` |
| Fixed point | Scale `1000`, with `fixed`, `int`, `f*`, `f/`, `f-sqrt`; integer-only view |
| Tokenization | Whitespace-separated, each token knows its file/line/column |
| Case sensitivity | Case-sensitive |
| Comments | `\ line` and `( block )` |
| Strings | `"text"` literals, passed around as string-pool handles |
| Floating point | First class: literals, promoted arithmetic, and `exp`/`ln`/`pow`/trig/rounding functions |
| Persistent data | Data stack, return stack, memory arena, dictionary, string pool |
| Per-frame data | Draw command queue (cleared each frame), input snapshot |
| Graphics | Filled/outlined rectangles and circles, lines, text |
| Input | Keyboard, mouse position, buttons, wheel |
| Time | `dt`, `time`, `frame` |
| Modularity | `include "path"` with cycle detection |
| Diagnostics | Errors carry file/line/column and the call stack |
| Window | 800x600 at 60 FPS |

## Execution model in one paragraph

Loading a file tokenizes it into located tokens, precomputes jump targets for every control-flow construct, and executes the top-level once. Definitions, variables, and includes registered there become part of the dictionary. After the top level, `init` is called once if it exists. From then on, every frame clears the draw command queue, calls `update`, calls `draw`, and the host renders the queued commands in order. State changes persist through memory and the dictionary; drawing is rebuilt from scratch every frame, with each command keeping the color that was current when it was queued.

## Example: a bouncing rectangle

The repository's `examples/test.fth` is a complete animation example:

```forth
\ GameForth demo: a bouncing rectangle.
\ The host runs init once, then update and draw every frame.

variable x
variable y
variable dx
variable dy

: init
  350 x !
  250 y !
  4 dx !
  3 dy !
;

: update
  dx @ x +!
  dy @ y +!

  x @ 0 < if 0 x ! dx @ negate dx ! then
  x @ 700 > if 700 x ! dx @ negate dx ! then
  y @ 0 < if 0 y ! dy @ negate dy ! then
  y @ 500 > if 500 y ! dy @ negate dy ! then
;

: draw
  "Bouncing rectangle" 20 20 20 rl-white draw-text
  x @ y @ 100 100 rl-red draw-rectangle
;
```

See [Examples](examples.md#bouncing-rectangle) for a line-by-line explanation.

## What GameForth is not

GameForth is not a complete ANS Forth implementation. It does not currently provide floating point, textures, sprites, sound, locals, scopes, standard output redirection, or file I/O from Forth. See [Limitations](language.md#limitations) for the full list.
