# Internals

This page describes how the GameForth implementation is organized and how to extend it. It is intended for contributors and for anyone who wants to understand why the language behaves the way it does.

## Contents

- [Project layout](#project-layout)
- [Build and test](#build-and-test)
- [Runtime architecture](#runtime-architecture)
- [Data structures](#data-structures)
- [Execution details](#execution-details)
- [Control-flow compilation](#control-flow-compilation)
- [Lifecycle and the host loop](#lifecycle-and-the-host-loop)
- [Includes](#includes)
- [Diagnostics](#diagnostics)
- [Memory model](#memory-model)
- [Strings](#strings)
- [Input and time](#input-and-time)
- [Graphics pipeline](#graphics-pipeline)
- [Tests](#tests)
- [Extending the language](#extending-the-language)
- [Possible future work](#possible-future-work)

## Project layout

| Path | Purpose |
| --- | --- |
| `src/main.rs` | CLI parsing, windowed/headless loops, hot reload, input snapshot, rendering |
| `src/lib.rs` | Exposes the runtime as a library so `main.rs` (and external users) link against it |
| `src/lexer.rs` | Tokenizer; located tokens, comments, strings, number bases |
| `src/stack.rs` | The `i64` data stack |
| `src/memory.rs` | Flat cell arena |
| `src/strings.rs` | String pool |
| `src/input.rs` | Per-frame input snapshot and tracked key list |
| `src/time.rs` | `dt`/`time`/`frame` state |
| `src/rng.rs` | Xorshift generator |
| `src/context.rs` | Everything a builtin can touch, behind one `&mut Context` |
| `src/dictionary.rs` | Dictionary, all builtin words, colors, key/mouse constants |
| `src/control.rs` | Control-flow compiler and `CompiledWord` |
| `src/error.rs` | `SourceLocation`, `CallFrame`, `ForthError` |
| `src/interpreter.rs` | Execution, lifecycle, includes, loops, definitions, diagnostics |
| `src/graphics.rs` | Draw command queue and current color |
| `src/bin/raylib_test.rs` | Separate raylib smoke-test binary; unrelated to Forth |
| `examples/test.fth` | Simple bounce demo |
| `examples/demo/main.fth` | Modular demo entry point |
| `examples/demo/lib/*.fth` | Demo modules loaded via `include` |
| `docs/` | This documentation |

`Cargo.toml` sets `default-run = "gameforth"` so `cargo run` builds the interpreter, not the raylib test bin.

## Build and test

```sh
cargo build
cargo run
cargo test
cargo fmt -- --check
```

## Runtime architecture

```text
main.rs
  +-- load_interpreter(path)
  |     +-- Interpreter::load(source, path)
  |           +-- tokenize(source, file) -> Vec<Token>
  |           +-- CompiledWord::new("<top-level>", tokens)   (control-flow compile)
  |           +-- execute top level
  |           +-- call init
  |
  +-- per frame (windowed)
        +-- snapshot_input(rl)          -> InputState
        +-- set_time(dt, elapsed, frame)
        +-- Interpreter::run_frame()
        |     +-- graphics.clear()
        |     +-- call update
        |     +-- call draw
        +-- render interpreter.context.graphics.commands
```

Everything the interpreter needs lives in one `Context` value. Builtins are plain `fn(&mut Context) -> Result<(), ForthError>` function pointers, so adding a word does not require threading new state through call sites.

## Data structures

### `Token` (`src/lexer.rs`)

```rust
enum TokenKind {
    Number(i64),
    Word(String),
    String(String),
}
struct Token { kind: TokenKind, location: SourceLocation }
```

Tokens keep their file/line/column so every error can point at an exact position.

### `Context` (`src/context.rs`)

One struct holding the data stack, `Graphics`, `Memory`, `StringPool`, `InputState`, `TimeState`, `Rng`, and the return stack. Passed mutably to every builtin.

### `Dictionary` and `Word` (`src/dictionary.rs`)

```rust
enum Word {
    Builtin(fn(&mut Context) -> Result<(), ForthError>),
    UserDefined(Rc<CompiledWord>),
    Color(Color),
    Address(usize),   // variable / create
    Constant(i64),
    Value(usize),     // value: index of one cell in the arena
}
```

`UserDefined` words are `Rc` so the dictionary can hand out clones without copying token lists.

### `CompiledWord` and `ControlMap` (`src/control.rs`)

Code that runs is always a `CompiledWord`: a token slice plus a `ControlMap` of precomputed jump targets for `if`/`else`/`then`, `begin`/`until`/`while`/`repeat`, and `do`/`loop`/`+loop`. Compilation happens when a file is loaded or a definition is made, which is how `begin ... nowhere` errors surface before any code executes.

### `Interpreter` (`src/interpreter.rs`)

Owner of the `Context` and `Dictionary`, plus bookkeeping: include stack, call stack, loop stack, recursion depth, and loaded-file tracking.

### `Memory` (`src/memory.rs`)

A flat `Vec<i64>`. Addresses are indices. `allocate`, `push_cell`, `allot` grow it; every accessor bounds-checks into a `ForthError`. `CELL_SIZE` is 1 (one arena slot per cell), so `cells` is currently the identity.

### `ForthError` (`src/error.rs`)

Message plus optional `SourceLocation` plus optional call stack. `Display` renders `file:line:col: message` and one `  in word at file:line:col` line per call frame.

## Execution details

Execution is a program counter walk over a token slice:

- Numbers push `i64`; strings intern into the pool and push the handle.
- Control words consume the `ControlMap`: `if` jumps to `else`+1 or `then`+1 on a false flag, `until`/`while`/`repeat`/`loop` jump to their precomputed targets.
- Everything else is dictionary lookup, dispatching on the `Word` kind.

Flow propagation uses an enum:

```rust
enum Flow { Normal, Exit, Leave }
```

`exit` propagates as `Flow::Exit` and unwinds only the current word: `call_word` and the `recurse` arm map a callee's `Exit` to `Flow::Normal`, so the caller sees an ordinary return and tokens after the call keep running. Without that containment, an `exit` deep inside a recursive word would silently end the whole program. `leave` propagates until the frame's own loop stack entry is found, then jumps one past that loop's `loop` word. A `leave` with no loop in the same execution aborts as an error (`leave outside loop`), and `load`/`run_frame` turn a leaked `Leave` from a lifecycle word into a distinct error message.

### Loop state

Each active `do` body gets a `LoopFrame { exec_id, do_index, loop_index, index, limit }` pushed onto a loop stack. `exec_id` distinguishes nested executions of the same word (recursion), so `loop` always pairs with the `do` currently on top, and `i`/`j` read the top two frames. The loop stack is truncated on word exit, which makes `leave`/`exit` from a loop self-cleaning.

### Definitions

`:` captures the body tokens, compile-checks them with `compile_control`, and stores an `Rc<CompiledWord>` in the dictionary. Markers like the definition's name token and closing `;` are consumed at definition time. Because bodies are `Rc`s resolved at call time, forward references work naturally.

`variable`, `create`, `constant`, `value`, and `to` are handled inline by the interpreter for the same reason — they need the parser cursor and stack at the same time.

### `to`

`to` works on both `value` words and plain variable addresses stored by `Address` entries. Anything else errors with `to: <name> is not a value or variable`.

## Control-flow compilation

`compile_control` walks a token stream with a stack of open constructs:

- `if` pushes an open `If`, resolved by `then` (recording optional `else` edges).
- `begin` pushes an open `Begin`; `while` replaces tracking with a `While` carrying its `begin` index; `until`/`repeat` close.
- `do`/`?do` push an open `Do`, closed by `loop`/`+loop`.

A `:` definition body is skipped in one pass so a definition's control words are never paired with constructs outside it — the body is compiled separately through `CompiledWord::new` when the interpreter reaches the `:`. Unclosed constructs at the end of a stream are errors that name the construct and point at its opening token.

## Lifecycle and the host loop

`Interpreter::load(&str, Option<PathBuf>)`:

1. Tokenizes and compiles the top level.
2. Pushes the file onto the include stack (cycle detection and relative-include resolution treat it as the root).
3. Executes the top level, tolerating `exit`.
4. Calls `init` once if a user-defined word by that name exists.

`run_frame()` clears the draw commands, then calls `update` and `draw`. Both are optional; neither runs unless defined. The host (`main.rs`) supplies per-frame time and input before each call:

- Windowed: real `dt` from raylib, real input snapshot, hot reload by mtime comparison.
- Headless: fixed 16 ms `dt`, empty input, `--frames` bound.

Hot reload rebuilds the entire `Interpreter`, so each save resets all state. A failed reload keeps the previous interpreter running and reports the error.

## Includes

`include "path"`:

1. Resolves the path: absolute as-is, else relative to the currently including file (`include_stack` top), else the main source file's directory, else `.`.
2. Canonicalizes the result; failure is an error at the `include` token's location.
3. Rejects the file if it is already on the include stack (cycle detection).
4. Tokenizes, compiles, remembers it in `loaded_files`, pushes it onto the include stack, executes, pops.

No caching, no deduplication: including the same file twice runs it twice. Cycle detection is what keeps the graph from looping forever.

## Diagnostics

Error plumbing works like this:

1. Builtins return bare `ForthError`s with a message.

2. The token executor attaches the offending token's location if none exists.

3. User-defined word calls push a `CallFrame` and, on error, attach a snapshot of the call stack to it via `with_call_stack`.

4. `Display` prints the location and then the stack in reverse-depth order.

This is why an error deep inside an included file reports `main.fth:...`, `lib/...`, and every intermediate word.

## Memory model

One arena for everything. Creation points:

| Source | Allocation |
| --- | --- |
| `variable x` | One cell, once per name (idempotent check via dictionary probe) |
| `create arr` | None; records `here` |
| `allot` | Extends or shrinks by `n` cells |
| `,` | One cell appended |
| `value v` | One cell |

Re-entry during hot reload starts a fresh interpreter, so a stale arena is never a concern within a session; within a single session there is no `free`.

## Strings

`StringPool` interns each string literal evaluation and returns an index. There is no deduplication — the same literal evaluated each frame becomes a new entry — so programs that reintern heavily grow the pool. This matches the design constraint that strings are handles, never pointers.

## Input and time

`InputState` is a plain snapshot: a `HashSet` of held and just-pressed key codes, mouse position/wheel, and two `bool` arrays for button state. `TRACKED_KEYS` enumerates the key codes main.rs polls; untracked keys resolve consistently to false.

`TimeState` is three integers (`dt_ms`, `elapsed_ms`, `frame`) copied in by the host per frame. Real time information comes from raylib in windowed mode; headless uses the fixed values from the CLI contract.

## Graphics pipeline

`Graphics` keeps a `Vec<DrawCommand>` plus a `current_color: Color`. Every draw word appends the current color into the command. `main.rs` matches on the enum and calls the matching raylib function — `draw_rectangle`, `draw_rectangle_lines`, `draw_circle`, `draw_circle_lines`, `draw_line`, `draw_text`.

Adding a new command is three steps: add the variant, add a `Graphics` method, add a match arm in `main.rs`.

## Tests

`cargo test` covers:

- Arithmetic, stack manipulation, and comparison words
- Control flow: `if`/`else`/`then`, `begin`/`until`, `while`/`repeat`, `do`/`loop`, `leave`, `exit`, `recurse`
- Variables, values, constants, `to`, arrays via `create`/`allot`/`,`
- Fixed-point math
- Strings: literals, `string-length`, `draw-text`
- Comments (`\` and `( ... )`)
- Lifecycle: `init` once, `update`/`draw` per frame, correct draw ordering
- Diagnostics: error messages, locations, call stacks
- Includes: nested relative resolution and cycle detection
- The bounce example, 500 frames, staying in bounds

Structural pairing (mismatched `if`/`then` and friends) surfaces through compile errors in the same suite.

## Extending the language

To add a builtin:

1. Write `fn my_word(ctx: &mut Context) -> Result<(), ForthError>`.
2. Register it in `Dictionary::new()` (`dict.add("my-word", my_word)`).
3. Document it in `docs/words.md` with a stack effect.

To add a constant, use `dict.add_constant`; for a color, `dict.add_color`. To add a compiler form, add a match arm in `Interpreter::execute_word_token` and — if it pairs with anything — a field in `ControlMap` and handling in `compile_control`.

Inputs for new host capabilities (audio, textures) would follow the input/time pattern: a struct on `Context`, filled in by `main.rs` per frame, read by builtins.

## Possible future work

- Local variables or a locals stack.
- Textures/sprites as first-class draw commands.
- Sound.
- A second wordlist or module namespaces.
- A REPL / watch mode for single-line evaluation.
- Constant folding or bytecode caching of `CompiledWord`s.
- Better text metrics by measuring the real font.
