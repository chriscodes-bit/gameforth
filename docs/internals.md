# Internals

This page describes how the GameForth implementation is organized and how to extend it. It is intended for contributors and for anyone who wants to understand why the language behaves the way it does.

## Contents

- [Project layout](#project-layout)
- [Build and test](#build-and-test)
- [Runtime architecture](#runtime-architecture)
- [Data structures](#data-structures)
- [Execution details](#execution-details)
- [Control-flow scanning](#control-flow-scanning)
- [Definitions and variables](#definitions-and-variables)
- [Graphics pipeline](#graphics-pipeline)
- [Error handling](#error-handling)
- [Tests](#tests)
- [Extending the language](#extending-the-language)
- [Implementation quirks](#implementation-quirks)
- [Possible future work](#possible-future-work)

## Project layout

| Path | Purpose |
| --- | --- |
| `src/main.rs` | Window setup and the frame loop |
| `src/lexer.rs` | Tokenizer and `Token` enum |
| `src/stack.rs` | Data stack |
| `src/dictionary.rs` | Dictionary, word kinds, builtins, color words |
| `src/interpreter.rs` | Token execution, control flow, definitions, variables |
| `src/graphics.rs` | Draw command queue and color state |
| `src/bin/raylib_test.rs` | Separate raylib smoke-test binary; not part of Forth |
| `examples/test.fth` | Source file loaded by the main application |
| `docs/` | This documentation |

## Build and test

```powershell
cargo build
cargo run
cargo test
cargo fmt -- --check
```

The main binary is `gameforth`. Cargo also builds `raylib_test` from `src/bin/raylib_test.rs`; that binary is an unrelated raylib drawing test and does not use the Forth interpreter.

## Runtime architecture

At a high level:

```text
main loop
  |
  +-- graphics.clear()
  |
  +-- Interpreter::eval(source)
  |     |
  |     +-- tokenize(source) -> Vec<Token>
  |     |
  |     +-- execute_tokens(tokens)
  |           |
  |           +-- Number       -> stack.push(number)
  |           +-- Special form -> if/else/then, begin/until, :/;,
  |           |                  variable, @, !, +!
  |           +-- Dictionary lookup
  |                 +-- Builtin     -> fn(&mut Stack, &mut Graphics)
  |                 +-- UserDefined -> execute_tokens(body)
  |                 +-- Color       -> graphics.set_color(color)
  |                 +-- Variable    -> stack.push(address)
  |
  +-- render graphics.commands in order
  |
  +-- repeat
```

The source string is read once in `main`, then passed to `Interpreter::eval` every frame.

## Data structures

### `Token`

```rust
pub enum Token {
    Number(i64),
    Word(String),
}
```

`tokenize` splits on whitespace. A token that parses as `i64` becomes `Number`; everything else becomes `Word`.

### `Stack`

```rust
pub struct Stack {
    data: Vec<i64>,
}
```

The stack exposes:

- `push(value)`
- `pop() -> Option<i64>`
- `peek() -> Option<&i64>`
- `second() -> Option<&i64>`

`second` returns the value below the top, or `None` if there are fewer than two values.

### `Word`

```rust
pub enum Word {
    Builtin(Builtin),
    UserDefined(Vec<Token>),
    Color(Color),
    Variable(usize),
}
```

- `Builtin` stores a function pointer.
- `UserDefined` stores a captured token list.
- `Color` stores a raylib color.
- `Variable` stores an index into the interpreter's variable storage.

`Dictionary::find` clones the matching `Word`, so execution does not hold a borrow on the dictionary.

### `Builtin`

```rust
pub type Builtin = fn(&mut Stack, &mut Graphics) -> Result<(), String>;
```

Builtins can access the stack and graphics state. They cannot directly access the dictionary or variables. Features that need interpreter-level state are implemented as special forms instead.

### `Dictionary`

```rust
pub struct Dictionary {
    words: HashMap<String, Word>,
}
```

The dictionary maps exact, case-sensitive names to words. Builtins and color words are registered in `Dictionary::new`.

### `Interpreter`

```rust
pub struct Interpreter {
    pub stack: Stack,
    pub graphics: Graphics,
    variables: Vec<i64>,
    dictionary: Dictionary,
}
```

The interpreter owns the stack, graphics state, variable storage, and dictionary. These all persist for the lifetime of the interpreter.

### `Graphics`

```rust
pub struct Graphics {
    pub commands: Vec<DrawCommand>,
    pub current_color: Color,
}
```

`DrawCommand` currently has two variants:

```rust
pub enum DrawCommand {
    Rectangle { x: i32, y: i32, width: i32, height: i32, color: Color },
    Circle { x: i32, y: i32, radius: i32, color: Color },
}
```

`Graphics::clear` removes all commands but does not reset `current_color`.

## Execution details

`Interpreter::execute_tokens` is a loop over a token slice with a program counter:

```rust
let mut pc = 0;

while pc < tokens.len() {
    match &tokens[pc] {
        // ...
    }
    pc += 1;
}
```

Numbers are pushed directly.

Special forms are matched by token text before dictionary lookup. This means they cannot be redefined by user code.

Other words are looked up in the dictionary:

- `Builtin` — call the function pointer
- `UserDefined` — recursively call `execute_tokens` on the stored body
- `Color` — update `graphics.current_color`
- `Variable` — push the stored address

User-defined word execution uses the Rust call stack. There is no separate Forth return stack.

## Control-flow scanning

GameForth does not compile control flow into jump instructions. Instead, it scans the token list at runtime.

### `if`, `else`, `then`

When the interpreter encounters `if`, it calls `find_conditional_boundaries(tokens, pc + 1)`.

That function scans forward and tracks nested `if` depth:

- nested `if` increments depth
- `then` at depth `0` is the matching `then`
- `then` at a deeper level decrements depth
- `else` at depth `0` is recorded as the matching `else`

It returns `(else_index, then_index)`.

If the popped condition is `0`, the program counter is set to the `else` index if one exists, otherwise to the `then` index. The loop's final `pc += 1` moves execution into the selected branch.

When the interpreter encounters `else` from the true branch, it scans forward for the matching `then` and jumps to it.

`then` itself does nothing at runtime.

### `begin`, `until`

When the interpreter encounters `until`, it pops a condition. If the condition is `0`, it calls `find_matching_begin(tokens, pc)` and jumps back to the matching `begin`.

`find_matching_begin` scans backward and tracks nested `until`/`begin` pairs:

- `until` increments depth
- `begin` at depth `0` is the match
- `begin` at a deeper level decrements depth

If the condition is nonzero, execution continues after `until`.

## Definitions and variables

### Word definitions

When the interpreter encounters `:`:

1. It reads the next token as the word name.
2. It scans forward for the first `;`.
3. It captures the tokens between the name and `;` as the body.
4. It inserts a `UserDefined(body)` entry into the dictionary.
5. It sets `pc` to the `;` token; the loop then advances past it.

The body is not executed at definition time.

### Variable declarations

When the interpreter encounters `variable`:

1. It reads the next token as the variable name.
2. If the dictionary already maps that name to `Word::Variable`, it does nothing.
3. Otherwise it allocates a new cell at `variables.len()`, initializes it to `0`, and defines the name as `Word::Variable(address)`.
4. It skips the name token.

The idempotent behavior is required because the source is evaluated every frame. Without it, every frame would allocate a new cell and reset the variable to `0`.

### Variable access

`@`, `!`, and `+!` are special forms:

- `@` pops an address, validates it, reads the cell, and pushes the value.
- `!` pops an address and a value, validates the address, and writes the cell.
- `+!` pops an address and a value, validates the address, and adds to the cell.

Address validation is performed by `Interpreter::variable_index`, which rejects negative addresses and addresses beyond the current variable storage.

## Graphics pipeline

Drawing words are builtins in `dictionary.rs`:

- `draw-rectangle` pops `height`, `width`, `y`, then `x`
- `draw-circle` pops `radius`, `y`, then `x`

Both call methods on `Graphics`:

- `Graphics::draw_rectangle`
- `Graphics::draw_circle`

Those methods push a `DrawCommand` containing the current `current_color`.

The main loop:

1. Calls `interpreter.graphics.clear()`.
2. Evaluates the source.
3. Iterates over `interpreter.graphics.commands`.
4. Calls the corresponding raylib drawing function for each command.

Because commands store their own color, changing `current_color` after a command is queued does not affect that command.

Coordinates are cast from `i64` to `i32` with Rust's `as` cast.

## Error handling

`Interpreter::eval` returns `Result<(), String>`. Builtins and special forms return `Err(String)` for:

- stack underflow
- division by zero
- unknown words
- malformed definitions
- invalid variable addresses
- missing control-flow terminators

Errors propagate with `?` through `execute_tokens`. The main loop handles the final error:

```rust
if let Err(err) = interpreter.eval(&source) {
    eprintln!("Forth error: {}", err);
    break;
}
```

There is no in-language error recovery.

## Tests

Tests live in the `tests` module at the bottom of `src/interpreter.rs`.

Current tests cover:

- variable store and fetch
- idempotent variable declarations
- `+!`
- invalid variable addresses
- the bouncing example staying within screen bounds

Run them with:

```powershell
cargo test
```

## Extending the language

### Adding a builtin word

1. Write a function with the `Builtin` signature:

```rust
fn my_word(stack: &mut Stack, graphics: &mut Graphics) -> Result<(), String> {
    let value = stack.pop().ok_or("Stack underflow")?;
    stack.push(value);
    Ok(())
}
```

2. Register it in `Dictionary::new`:

```rust
dict.add("my-word", my_word);
```

3. Document its stack effect in `docs/words.md`.

### Adding a color word

Register it in `Dictionary::new`:

```rust
dict.add_color("rl-cyan", Color::SKYBLUE);
```

Then document it in `docs/words.md`.

### Adding a special form

Add a match arm in `Interpreter::execute_tokens` before the generic dictionary lookup:

```rust
Token::Word(word) if word == "my-form" => {
    // manipulate self.stack, self.variables, etc.
}
```

Special forms are appropriate when a feature needs access to the interpreter, dictionary, or variable storage.

### Adding a draw primitive

1. Add a variant to `DrawCommand`.
2. Add a method to `Graphics` that pushes the command with `self.current_color`.
3. Add a builtin in `dictionary.rs` that pops the required values.
4. Add a render arm in `main.rs`.

### Adding a variable-like feature

1. Add a variant to `Word`.
2. Add a method to `Dictionary` that inserts it.
3. Handle the variant in `Interpreter::execute_tokens`.

## Implementation quirks

- The source is read once and evaluated every frame.
- The stack is not cleared between frames.
- The dictionary and variables persist between frames.
- Draw commands are cleared every frame.
- `current_color` is not cleared every frame.
- `variable` declarations are idempotent.
- Special forms are matched before dictionary lookup and cannot be redefined.
- `:` captures the first `;`; nested definitions are not supported.
- Control-flow boundaries are found by runtime scans, not compiled jumps.
- User-defined words recurse on the Rust call stack.
- Coordinates are truncated to `i32` with an `as` cast.
- There are no comments, strings, floats, or standard library words.

## Possible future work

Natural extensions include:

- comments
- strings and text rendering
- floating-point numbers
- frame/time words
- random numbers
- more color control, including alpha
- lines, triangles, and polygons
- input handling
- constants and locals
- a return stack
- `mod`, `negate`, and other arithmetic conveniences
- error recovery
- a configurable source path or command-line argument
