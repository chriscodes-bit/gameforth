# Word Reference

This page is the reference for every word currently available in GameForth.

## Contents

- [Notation](#notation)
- [Word kinds](#word-kinds)
- [Quick reference](#quick-reference)
- [Arithmetic words](#arithmetic-words)
- [Stack words](#stack-words)
- [Comparison words](#comparison-words)
- [Output word](#output-word)
- [Graphics words](#graphics-words)
- [Color words](#color-words)
- [Control-flow forms](#control-flow-forms)
- [Definition forms](#definition-forms)
- [Variable forms](#variable-forms)
- [Redefinition and shadowing](#redefinition-and-shadowing)
- [Error summary](#error-summary)

## Notation

Stack effects are written as:

```text
( before -- after )
```

The rightmost item is the top of the stack.

For example:

```text
+   ( a b -- a+b )
dup ( a -- a a )
```

`flag` means a truth value: `0` is false, and any nonzero value is true. Comparison words produce `-1` for true and `0` for false.

## Word kinds

GameForth has four kinds of named behavior:

| Kind | Examples | How it is resolved |
| --- | --- | --- |
| Builtin | `+`, `dup`, `draw-rectangle` | Function pointer stored in the dictionary |
| Special form | `if`, `then`, `:`, `variable`, `@`, `!` | Handled directly by the interpreter before dictionary lookup |
| Color word | `rl-red`, `rl-blue` | Dictionary entry that sets the current color |
| Variable word | Any name declared with `variable` | Dictionary entry that pushes the variable's address |

User-defined words created with `:` are stored as token lists and executed when called.

## Quick reference

### Arithmetic

| Word | Stack effect | Summary |
| --- | --- | --- |
| `+` | `( a b -- a+b )` | Add |
| `-` | `( a b -- a-b )` | Subtract |
| `*` | `( a b -- a*b )` | Multiply |
| `/` | `( a b -- a/b )` | Integer divide |

### Stack

| Word | Stack effect | Summary |
| --- | --- | --- |
| `dup` | `( a -- a a )` | Duplicate top |
| `drop` | `( a -- )` | Discard top |
| `swap` | `( a b -- b a )` | Swap top two |
| `over` | `( a b -- a b a )` | Copy second to top |

### Comparison

| Word | Stack effect | Summary |
| --- | --- | --- |
| `=` | `( a b -- flag )` | Equal |
| `<` | `( a b -- flag )` | Less than |
| `>` | `( a b -- flag )` | Greater than |
| `0=` | `( n -- flag )` | Equal to zero |

### Output

| Word | Stack effect | Summary |
| --- | --- | --- |
| `.` | `( n -- )` | Print integer to stdout |

### Graphics

| Word | Stack effect | Summary |
| --- | --- | --- |
| `draw-rectangle` | `( x y width height -- )` | Queue a filled rectangle |
| `draw-circle` | `( x y radius -- )` | Queue a filled circle |

### Colors

| Word | Stack effect | Summary |
| --- | --- | --- |
| `rl-red` | `( -- )` | Set current color to red |
| `rl-green` | `( -- )` | Set current color to green |
| `rl-blue` | `( -- )` | Set current color to blue |
| `rl-white` | `( -- )` | Set current color to white |
| `rl-black` | `( -- )` | Set current color to black |
| `rl-yellow` | `( -- )` | Set current color to yellow |
| `rl-orange` | `( -- )` | Set current color to orange |
| `rl-purple` | `( -- )` | Set current color to purple |
| `rl-pink` | `( -- )` | Set current color to pink |
| `rl-gray` | `( -- )` | Set current color to gray |

### Control flow and definitions

| Word | Stack effect | Summary |
| --- | --- | --- |
| `if` | `( flag -- )` | Begin a conditional |
| `else` | `( -- )` | Begin the false branch |
| `then` | `( -- )` | End a conditional |
| `begin` | `( -- )` | Begin a loop |
| `until` | `( flag -- )` | Loop back while `flag` is `0` |
| `:` | `( -- )` | Begin a word definition; consumes the next token as the name |
| `;` | `( -- )` | End a word definition |

### Variables

| Word | Stack effect | Summary |
| --- | --- | --- |
| `variable` | `( -- )` | Declare a variable; consumes the next token as the name |
| `@` | `( addr -- value )` | Fetch |
| `!` | `( value addr -- )` | Store |
| `+!` | `( n addr -- )` | Add to stored value |

## Arithmetic words

### `+`

```text
( a b -- a+b )
```

Pops `b` and `a`, pushes their sum.

```forth
7 3 + .
```

Prints `10`.

### `-`

```text
( a b -- a-b )
```

Pops `b` and `a`, pushes `a - b`.

```forth
10 4 - .
```

Prints `6`.

```forth
4 10 - .
```

Prints `-6`.

### `*`

```text
( a b -- a*b )
```

Pops `b` and `a`, pushes their product.

```forth
6 7 * .
```

Prints `42`.

### `/`

```text
( a b -- a/b )
```

Pops `b` and `a`, pushes integer division `a / b`.

Integer division truncates toward zero.

```forth
17 5 / .
```

Prints `3`.

```forth
-7 2 / .
```

Prints `-3`.

If `b` is `0`, evaluation fails with `Division by zero`.

## Stack words

### `dup`

```text
( a -- a a )
```

Duplicates the top value.

```forth
5 dup + .
```

Prints `10`.

### `drop`

```text
( a -- )
```

Removes the top value.

```forth
1 2 drop .
```

Prints `1`.

### `swap`

```text
( a b -- b a )
```

Exchanges the top two values.

```forth
10 20 swap . .
```

Prints `10`, then `20`.

### `over`

```text
( a b -- a b a )
```

Copies the second value to the top.

```forth
3 4 over . . .
```

Prints `3`, `4`, `3`.

## Comparison words

Comparison words return `-1` for true and `0` for false.

### `=`

```text
( a b -- flag )
```

```forth
10 10 = .
```

Prints `-1`.

```forth
10 20 = .
```

Prints `0`.

### `<`

```text
( a b -- flag )
```

True when `a < b`.

```forth
10 20 < .
```

Prints `-1`.

### `>`

```text
( a b -- flag )
```

True when `a > b`.

```forth
20 10 > .
```

Prints `-1`.

### `0=`

```text
( n -- flag )
```

True when `n` is zero.

```forth
0 0= .
```

Prints `-1`.

```forth
5 0= .
```

Prints `0`.

## Output word

### `.`

```text
( n -- )
```

Pops an integer and prints it to standard output, followed by a newline.

```forth
42 .
```

Terminal output:

```text
42
```

`.` does not draw text in the game window. It is useful for debugging, but printing every frame will flood the terminal.

## Graphics words

### `draw-rectangle`

```text
( x y width height -- )
```

Queues a filled rectangle.

- `x`, `y` — top-left corner in pixels
- `width`, `height` — size in pixels
- color — the current color at the time the command is queued

```forth
350 250 100 100 rl-red draw-rectangle
```

This queues a red `100x100` rectangle at `(350, 250)`.

The word consumes all four stack values and produces nothing.

### `draw-circle`

```text
( x y radius -- )
```

Queues a filled circle.

- `x`, `y` — center in pixels
- `radius` — radius in pixels
- color — the current color at the time the command is queued

```forth
400 300 50 rl-blue draw-circle
```

This queues a blue circle centered at `(400, 300)` with radius `50`.

The word consumes all three stack values and produces nothing.

## Color words

Color words set the interpreter's current drawing color. They do not draw anything and they do not change the stack.

These are raylib's named `Color` constants, not CSS colors. Some values differ from the bright variants you might expect; for example, `rl-green` is `(0, 128, 0)`.

| Word | RGBA |
| --- | --- |
| `rl-red` | `(255, 0, 0, 255)` |
| `rl-green` | `(0, 128, 0, 255)` |
| `rl-blue` | `(0, 0, 255, 255)` |
| `rl-white` | `(255, 255, 255, 255)` |
| `rl-black` | `(0, 0, 0, 255)` |
| `rl-yellow` | `(255, 255, 0, 255)` |
| `rl-orange` | `(255, 165, 0, 255)` |
| `rl-purple` | `(128, 0, 128, 255)` |
| `rl-pink` | `(255, 192, 203, 255)` |
| `rl-gray` | `(128, 128, 128, 255)` |

The initial color is white. The current color persists across frames until another color word changes it.

Example:

```forth
50 50 100 100 rl-red draw-rectangle
200 50 100 100 rl-blue draw-rectangle
50 200 50 rl-yellow draw-circle
```

The first rectangle is red, the second is blue, and the circle is yellow.

Because each draw command snapshots the current color, changing the color after a draw word does not affect the command that was already queued.

## Control-flow forms

These forms are handled directly by the interpreter. They are not ordinary dictionary words.

### `if`

```text
( flag -- )
```

Pops a flag and begins a conditional. If the flag is `0`, execution jumps to the matching `else` (if present) or to just after the matching `then`.

### `else`

```text
( -- )
```

Marks the start of the false branch. When execution reaches `else` from the true branch, it jumps to just after the matching `then`.

### `then`

```text
( -- )
```

Marks the end of a conditional. It does nothing at runtime.

Example:

```forth
10 5 > if
  1 .
else
  0 .
then
```

Prints `1`.

### `begin`

```text
( -- )
```

Marks the start of a loop. It does nothing at runtime.

### `until`

```text
( flag -- )
```

Pops a flag. If the flag is `0`, execution jumps back to the matching `begin`. If the flag is nonzero, execution continues after `until`.

The loop therefore continues while the condition is false.

Example:

```forth
variable n
5 n !

begin
  n @ .
  -1 n +!
  n @ 0 <
until
```

Prints `5`, `4`, `3`, `2`, `1`, `0`.

## Definition forms

### `:`

```text
( -- )
```

Starts a word definition. The next token is the word name. All tokens up to the first `;` become the body.

```forth
: double
  2 *
;
```

### `;`

```text
( -- )
```

Ends a word definition. A `;` outside a definition is an error.

Example:

```forth
: square
  dup *
;

7 square .
```

Prints `49`.

## Variable forms

### `variable`

```text
( -- )
```

Declares a variable. The next token is the variable name.

The variable:

- is initialized to `0`
- has one `i64` cell
- is global
- is only allocated the first time the declaration is seen

After declaration, the variable name pushes the cell's address.

```forth
variable score
```

### `@`

```text
( addr -- value )
```

Fetches the value stored at an address.

```forth
variable score
42 score !
score @ .
```

Prints `42`.

### `!`

```text
( value addr -- )
```

Stores a value at an address.

```forth
variable score
42 score !
```

### `+!`

```text
( n addr -- )
```

Adds `n` to the value stored at an address.

```forth
variable score
10 score !
5 score +!
score @ .
```

Prints `15`.

### Variable addresses

An address is an index into the interpreter's variable storage. It is not a raw memory pointer. Passing an address that does not refer to an existing variable to `@`, `!`, or `+!` fails with:

```text
Invalid variable address: <number>
```

Do not rely on the numeric value of an address.

## Redefinition and shadowing

Most named words can be replaced with `:`:

```forth
: draw-rectangle
  drop drop drop drop
;
```

After this definition, `draw-rectangle` no longer queues a rectangle; it just drops four values.

Special forms such as `if`, `then`, `:`, `;`, `variable`, `@`, `!`, and `+!` are matched before dictionary lookup, so defining a word with one of those names does not change how the token is interpreted.

Because the source is evaluated every frame, word definitions are re-created each frame. A definition's body is captured as tokens when the definition executes.

## Error summary

| Error | Typical cause |
| --- | --- |
| `Stack underflow` | Not enough operands |
| `Division by zero` | `/` with a zero divisor |
| `Unknown word: <word>` | Undefined word or typo |
| `Expected word name after :` | `:` not followed by a name |
| `Missing ; in definition` | Definition not terminated |
| `Unexpected ;` | Stray `;` |
| `Expected variable name after variable` | `variable` not followed by a name |
| `Invalid variable address: <n>` | Bad address passed to `@`, `!`, or `+!` |
| `Missing then for if` | Unclosed conditional |
| `Missing begin for until` | `until` without a matching `begin` |

When an error occurs, the main loop prints `Forth error: <message>` and stops.
