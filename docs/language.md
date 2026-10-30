# Language Guide

This page describes the GameForth language as implemented in this repository. It is a small Forth-like language: programs are sequences of whitespace-separated tokens, numbers are pushed onto a data stack, and words consume and produce stack values.

## Contents

- [Execution model](#execution-model)
- [Lexical structure](#lexical-structure)
- [Data types and the stack](#data-types-and-the-stack)
- [Arithmetic](#arithmetic)
- [Stack manipulation](#stack-manipulation)
- [Comparison and truth](#comparison-and-truth)
- [Output](#output)
- [Control flow](#control-flow)
- [Word definitions](#word-definitions)
- [Variables](#variables)
- [State and frame lifecycle](#state-and-frame-lifecycle)
- [Errors](#errors)
- [Limitations](#limitations)
- [Idioms and workarounds](#idioms-and-workarounds)

## Execution model

A GameForth program is a text file. The application reads the file once at startup, then evaluates the same token stream once per frame.

Each evaluation:

1. Clears the graphics command list.
2. Executes the program's tokens from beginning to end.
3. Renders the commands that were queued during execution.

The following state persists between evaluations:

- the data stack
- user-defined words in the dictionary
- variables and their values
- the current drawing color

The following state does not persist:

- queued draw commands, which are cleared before every evaluation

This means a GameForth source file is best thought of as a frame script, not as a program that runs once. If you want a value to change over time, store it in a variable and update it once per evaluation.

A program should normally leave the stack balanced at the end of each frame. Values left on the stack persist and will be added to on the next frame, which is rarely what you want.

## Lexical structure

The lexer splits the source on whitespace. Each token is classified as either:

- a number, if it parses as a signed 64-bit integer (`i64`)
- a word, otherwise

Examples:

```forth
350 250 100 100 rl-red draw-rectangle
```

Tokens:

- `350`, `250`, `100`, `100` — numbers
- `rl-red`, `draw-rectangle` — words

### Case sensitivity

Words are case-sensitive. `dup` works; `DUP` is an unknown word unless it has been defined separately.

### No comments

There is no comment syntax. The characters `\` and parentheses are not treated specially. A token such as `\` or `(comment)` is just an unknown word.

This is invalid:

```forth
\ this is not a comment
```

To document a Forth program, keep notes outside the program or in a separate Markdown file.

### No strings or character literals

There are no string or character literals. A quoted token such as `"hello"` is just a word named `"hello"`, which is normally unknown.

### Whitespace

Spaces, tabs, and newlines are all equivalent separators. Indentation is cosmetic.

### Word names

Any token that is not an integer can be a word name. Existing names include symbols such as `+`, `-`, `*`, `/`, `=`, `<`, `>`, `0=`, `@`, `!`, `+!`, `:`, and `;`.

## Data types and the stack

GameForth has one data type: signed 64-bit integers.

All values live on a single data stack. Words operate on the top of the stack. The stack effect notation used in this documentation is:

```text
( before -- after )
```

The rightmost item is the top of the stack.

Examples:

```text
+      ( a b -- a+b )
dup    ( a -- a a )
swap   ( a b -- b a )
```

### Truth values

GameForth follows the traditional Forth convention:

- `0` is false.
- Any nonzero value is true in `if`.
- Comparison words produce `-1` for true and `0` for false.

### Empty-stack errors

Most words require operands. If there are not enough values on the stack, evaluation fails with:

```text
Stack underflow
```

There is no stack-depth query and no stack-empty check. A program cannot ask how many values are on the stack.

## Arithmetic

All arithmetic is integer arithmetic on `i64`.

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `+` | `( a b -- a+b )` | Addition |
| `-` | `( a b -- a-b )` | Subtraction |
| `*` | `( a b -- a*b )` | Multiplication |
| `/` | `( a b -- a/b )` | Integer division |

Examples:

```forth
7 3 + .
```

Prints `10`.

```forth
17 5 / .
```

Prints `3`. Integer division truncates toward zero.

```forth
7 2 / .
```

Prints `3`.

```forth
-7 2 / .
```

Prints `-3`.

Division by zero fails with:

```text
Division by zero
```

There is no `mod`, `negate`, `abs`, `min`, `max`, `1+`, `1-`, `2*`, or `2/` word. See [Idioms and workarounds](#idioms-and-workarounds).

### Overflow

Arithmetic uses Rust's normal `i64` operations. Avoid values outside the `i64` range. In debug builds, arithmetic overflow panics; in release builds, Rust's default wrapping behavior applies. GameForth does not add its own overflow checks.

## Stack manipulation

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `dup` | `( a -- a a )` | Duplicate the top value |
| `drop` | `( a -- )` | Discard the top value |
| `swap` | `( a b -- b a )` | Exchange the top two values |
| `over` | `( a b -- a b a )` | Copy the second value to the top |

Examples:

```forth
5 dup + .
```

Stack: `5` → `5 5` → `10`. Prints `10`.

```forth
10 20 swap . .
```

Stack: `10 20` → `20 10`. The first `.` prints `10`, then the second prints `20`.

```forth
3 4 over . . .
```

Stack: `3 4` → `3 4 3`. The output is `3`, `4`, `3` in that order.

## Comparison and truth

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `=` | `( a b -- flag )` | `-1` if equal, otherwise `0` |
| `<` | `( a b -- flag )` | `-1` if `a < b`, otherwise `0` |
| `>` | `( a b -- flag )` | `-1` if `a > b`, otherwise `0` |
| `0=` | `( n -- flag )` | `-1` if `n == 0`, otherwise `0` |

Examples:

```forth
10 10 = .
```

Prints `-1`.

```forth
10 20 < .
```

Prints `-1`.

```forth
10 20 > .
```

Prints `0`.

```forth
0 0= .
```

Prints `-1`.

Comparisons are normally used with `if` or `until`.

## Output

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `.` | `( n -- )` | Pop an integer and print it to stdout followed by a newline |

Example:

```forth
42 .
```

Prints:

```text
42
```

`.` prints to the terminal, not to the game window. There is no word for drawing text in the game window.

## Control flow

### `if`, `else`, `then`

Syntax:

```forth
<condition>
if
  <true branch>
else
  <false branch>
then
```

The `else` branch is optional:

```forth
<condition>
if
  <true branch>
then
```

Semantics:

- `if` pops one value.
- If the value is `0`, execution jumps to the `else` branch if present, otherwise to just after `then`.
- If the value is nonzero, execution continues with the true branch.
- `else` transfers control to just after the matching `then`.
- `then` marks the end of the conditional and does nothing at runtime.

Example:

```forth
10 5 > if
  1 .
else
  0 .
then
```

`10 5 >` produces `-1`, so the program prints `1`.

Conditionals can be nested. The interpreter tracks nested `if`/`then` pairs while finding the matching `else` and `then`.

There is no `exit`, `while`, `repeat`, `do`, or `loop`.

### `begin` and `until`

Syntax:

```forth
begin
  <body>
  <condition>
until
```

Semantics:

- `begin` marks the start of a loop and does nothing at runtime.
- `until` pops one value.
- If the value is `0`, execution jumps back to the matching `begin`.
- If the value is nonzero, execution continues after `until`.

In other words, the loop continues while the condition is false (`0`) and stops when it becomes true (nonzero).

Example: count down from 5 to 0.

```forth
variable n
5 n !

begin
  n @ .
  -1 n +!
  n @ 0 <
until
```

This prints `5`, `4`, `3`, `2`, `1`, `0`, then stops.

Loops can be nested. The interpreter counts nested `begin`/`until` pairs when searching backwards for the matching `begin`.

### Mixing conditionals and loops

Conditionals and loops can be nested inside each other. For example:

```forth
variable i
5 i !

begin
  i @ 0 > if
    i @ .
  then
  -1 i +!
  i @ 0=
until
```

This prints `5`, `4`, `3`, `2`, `1` and stops when `i` reaches `0`.

## Word definitions

Syntax:

```forth
: name
  <body>
;
```

A definition:

- consumes the next token as the word name
- captures all tokens up to the first `;`
- stores the captured token list in the dictionary
- does not execute the body at definition time

When the word is later executed, its body is executed with the current stack, variables, and graphics state.

Example:

```forth
: square
  dup *
;

7 square .
```

`square` duplicates the top value and multiplies it by itself. The program prints `49`.

Example with two operations:

```forth
: double
  2 *
;

21 double .
```

Prints `42`.

### Redefinition

Defining a word with an existing name replaces the old definition. Because the entire source is re-evaluated every frame, definitions are effectively re-created every frame. This is normally harmless because the body is static text.

### Forward references

A definition body is stored as tokens and is only resolved when the word is executed. Therefore a word can refer to a word that is defined later in the file, as long as the later definition has already executed by the time the first word is called.

### Recursion

A word can call itself. The interpreter implements user-defined word calls with Rust function recursion, so deeply recursive Forth words can exhaust the host stack. There is no tail-call optimization and no exposed return stack.

Example:

```forth
: countdown
  dup 0 > if
    dup .
    -1 +
    countdown
  else
    drop
  then
;

5 countdown
```

This prints `5`, `4`, `3`, `2`, `1`. The final `drop` removes the zero.

### No locals

There are no local variables or local scopes. Use the data stack and global variables.

## Variables

Variables are the primary way to keep state across frames.

### Declaring a variable

Syntax:

```forth
variable name
```

This:

- allocates one `i64` cell, initialized to `0`
- defines `name` so that executing `name` pushes the cell's address
- has no stack effect itself

Example:

```forth
variable score
```

### Reading and writing

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `@` | `( addr -- value )` | Fetch the value at an address |
| `!` | `( value addr -- )` | Store a value at an address |
| `+!` | `( n addr -- )` | Add `n` to the value at an address |

Example:

```forth
variable score
0 score !
10 score +!
score @ .
```

Prints `10`.

Step by step:

- `variable score` creates a cell containing `0`.
- `0 score !` stores `0` (redundant, but shows `!`).
- `10 score +!` adds `10` to the cell.
- `score @` pushes the cell's value.
- `.` prints it.

### Addresses are opaque indices

An address is a small integer index into the interpreter's variable storage. Do not rely on the numeric value of an address. It is not a raw pointer and cannot be used to access arbitrary memory.

Using an address that does not refer to an existing variable fails with:

```text
Invalid variable address: <number>
```

### Declaration is idempotent

Because the source is re-evaluated every frame, `variable name` only allocates the first time it is seen. If the name already refers to a variable, the declaration does nothing. It does not reset the cell.

This is essential for animation programs: you can put `variable x` at the top of the file and update `x` every frame without resetting it.

To reset a variable explicitly, store a value into it:

```forth
0 x !
```

### Initialization pattern

A variable starts at `0`, but that may not be a good starting position. Use a separate flag variable to initialize once:

```forth
variable x
variable initialized

initialized @ 0= if
  350 x !
  -1 initialized !
then
```

On the first frame, `initialized` is `0`, so the initialization block runs and sets it to `-1`. On later frames, `initialized` is nonzero, so the block is skipped.

This pattern is used by the bouncing-rectangle example.

### Scope

All variables are global for the lifetime of the interpreter. There are no local variables, modules, or namespaces.

### Interaction with word definitions

A `variable` declaration and a `:` definition both write to the same dictionary. Avoid using the same name for both: the dictionary has only one entry per name, and because the source is re-evaluated every frame, the interaction between a variable declaration and a later word definition can be confusing.

## State and frame lifecycle

This section summarizes the most important consequence of the implementation: the source runs every frame.

| State | Survives frames? | Notes |
| --- | --- | --- |
| Data stack | Yes | Values left on the stack accumulate |
| Dictionary | Yes | Definitions and variables persist |
| Variables | Yes | `variable` does not reset them |
| Current color | Yes | Set by `rl-*` words |
| Draw commands | No | Cleared before each evaluation |
| Window contents | No | Redrawn from the command list each frame |

### Recommended frame structure

A typical animation program looks like this:

```forth
variable x
variable initialized

initialized @ 0= if
  0 x !
  -1 initialized !
then

x @ 1 + x !

x @ 400 100 100 rl-red draw-rectangle
```

Each frame:

1. Initialize state if this is the first frame.
2. Update state.
3. Draw using the updated state.

### Keep the stack balanced

The stack is not cleared between frames. If a program pushes values and never consumes them, the stack grows indefinitely. A frame should normally end with the same stack depth it started with.

### Drawing is deferred

`draw-rectangle` and `draw-circle` do not draw immediately. They append commands to a queue. The queue is cleared before the next evaluation and rendered after the current evaluation. This means:

- multiple draw words in one frame produce multiple shapes
- later commands are drawn on top of earlier commands
- a draw command keeps the color that was current when it was queued

See [Graphics and animation](graphics.md) for details.

## Errors

Evaluation returns an error string. The main loop prints it as:

```text
Forth error: <message>
```

and then stops.

| Error message | Cause |
| --- | --- |
| `Stack underflow` | A word needed more values than were on the stack |
| `Division by zero` | The right operand of `/` was zero |
| `Unknown word: <word>` | A token was not a number and was not found in the dictionary |
| `Expected word name after :` | `:` was not followed by a word token |
| `Missing ; in definition` | A `:` definition reached the end of the token stream without `;` |
| `Unexpected ;` | A `;` appeared outside a definition |
| `Expected variable name after variable` | `variable` was not followed by a word token |
| `Invalid variable address: <n>` | `@`, `!`, or `+!` was given an address that does not refer to a variable |
| `Missing then for if` | An `if` had no matching `then` |
| `Missing begin for until` | An `until` had no matching `begin` |

The application also panics with `Could not read Forth file` if `examples/test.fth` cannot be read at startup.

There is no error recovery, exception handling, or `try`/`catch` inside Forth.

## Limitations

Current limitations include:

- No comments.
- No strings or character literals.
- No floating-point numbers.
- No text rendering from Forth.
- No keyboard, mouse, or other input.
- No time, frame counter, or randomness words.
- No sound or music.
- No lines, triangles, polygons, images, or textures.
- No `mod`, `negate`, `abs`, `min`, `max`, `and`, `or`, `xor`, or `invert`.
- No `1+`, `1-`, `2*`, or `2/`.
- No `constant`, `value`, `to`, `create`, `does>`, `allot`, `here`, or `,`.
- No local variables or scopes.
- No return stack words such as `>r` and `r>`.
- No `exit`, `leave`, `while`, `repeat`, `do`, or `loop`.
- No `recurse`; a word can still call itself by name.
- No standard library or module system.
- No user-defined error handling.
- No overflow checking beyond Rust's normal behavior.
- Only one data type: signed 64-bit integer.
- Only one global stack.
- Only filled rectangles and filled circles as graphics primitives.

## Idioms and workarounds

### Negate a number

There is no `negate`. Use either:

```forth
-1 *
```

or:

```forth
0 swap -
```

Example:

```forth
5 -1 * .
```

Prints `-5`.

### Compute a remainder

There is no `mod`. Use integer division and subtraction:

```forth
dup <divisor> / <divisor> * -
```

Example: `17 mod 5`

```forth
17 dup 5 / 5 * - .
```

Prints `2`.

### Clamp a value

Use `if` blocks and store the clamped value back into the variable:

```forth
x @ 0 < if
  0 x !
then

x @ 700 > if
  700 x !
then
```

### Reflect a velocity at a boundary

Negate the velocity when a position crosses a limit:

```forth
x @ 700 > if
  700 x !
  -1 dx @ * dx !
then
```

### Initialize once

Use a flag variable:

```forth
variable initialized

initialized @ 0= if
  <initial values>
  -1 initialized !
then
```

### Toggle a boolean

A true flag is `-1` and a false flag is `0`. To toggle a variable between those values, use `0=` and store the result back:

```forth
flag @ 0= flag !
```

If `flag` is `0`, `0=` produces `-1`. If `flag` is `-1`, `0=` produces `0`.

### Print a value

```forth
value .
```

`.` prints to stdout, not to the game window.
