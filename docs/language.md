# Language Guide

This page describes the GameForth language as implemented in this repository. It is a small Forth-like language: programs are sequences of whitespace-separated tokens, numbers are pushed onto a data stack, and words consume and produce stack values.

## Contents

- [Execution model](#execution-model)
- [Lexical structure](#lexical-structure)
- [Data types and the stack](#data-types-and-the-stack)
- [Arithmetic](#arithmetic)
- [Fixed-point math](#fixed-point-math)
- [Random numbers](#random-numbers)
- [Stack manipulation](#stack-manipulation)
- [Return stack](#return-stack)
- [Comparison and truth](#comparison-and-truth)
- [Output](#output)
- [Control flow](#control-flow)
- [Word definitions](#word-definitions)
- [Memory and variables](#memory-and-variables)
- [Strings](#strings)
- [Input](#input)
- [Time](#time)
- [Includes](#includes)
- [Errors](#errors)
- [Limitations](#limitations)

## Execution model

A GameForth program is a text file. The host loads it once:

1. The file is tokenized, and jump targets for all control-flow words are precomputed. Errors in `if`/`do`/`begin` pairing are found here, before anything runs.
2. The top-level code runs once. This is where `:` definitions, `variable` declarations, and `include` calls normally go.
3. If a word named `init` is defined, it is called once.

Then, every frame:

1. The draw command queue is cleared.
2. `update` is called (if defined).
3. `draw` is called (if defined).
4. The host renders the queued draw commands.

The following state persists across frames:

- the data stack
- the return stack (between words; cleared within a word's call)
- user-defined words in the dictionary
- the memory arena (variables, arrays, constants, values)
- the string pool
- the current drawing color

The following does not persist:

- queued draw commands, which are cleared before every frame
- the input snapshot, which is refreshed every frame

`update` and `draw` are both optional. If `draw` is missing, the screen just shows the background.

A word should normally leave the stack as deep as it found it. The data stack is not cleared between frames, so leftover values accumulate.

## Lexical structure

The lexer splits the source on whitespace and classifies each token as a number, a word, or a string. Every token remembers the file, line, and column it came from, and errors point back at it.

### Numbers

Integers parse as signed 64-bit values. Known prefixes:

```forth
42        \ decimal
-42       \ negative
0xFF      \ hexadecimal
$FF       \ hexadecimal
0b1010    \ binary
%1010     \ binary
```

A token that does not parse as a number is treated as a word.

### Comments

```forth
\ a line comment, to end of line
( a block comment, to the closing paren )
```

A `\` starting a token comments out the rest of the line. A `(` starting a token opens a block comment closed by the next `)`. There is no nesting for block comments.

### Strings

A string literal is `"text"`. The traditional `s" text"` spelling is also accepted.

```forth
"hello" drop
s" escape: \" quote, newline \n" drop
```

Escape sequences inside strings: `\n`, `\t`, `\r`, `\\`, `\"`, `\0`. An unknown escape keeps its backslash and character. Evaluating a string token pushes a handle (an integer) into the string pool; see [Strings](#strings).

### Whitespace and case

Spaces, tabs, and newlines are all equivalent separators. Words are case-sensitive: `dup` works, `DUP` is unknown unless you define it.

## Data types and the stack

GameForth has one data type: signed 64-bit integers. Strings exist only as handles into a pool.

Stack effects in this documentation are written as:

```text
( before -- after )
```

The rightmost item is the top of the stack.

### Truth values

- `0` is false.
- Any nonzero value is true in `if`, `until`, and `while`.
- Comparison words produce `-1` for true and `0` for false.

### Stack underflow

```text
stack underflow
```

Popping from an empty stack is an error, not undefined behavior.

## Arithmetic

All arithmetic is integer arithmetic on `i64`, wrapping on overflow.

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `+` | `( a b -- a+b )` | Addition |
| `-` | `( a b -- a-b )` | Subtraction |
| `*` | `( a b -- a*b )` | Multiplication |
| `/` | `( a b -- a/b )` | Division, truncating toward zero |
| `mod` | `( a b -- a%b )` | Remainder, sign follows `a` |
| `/mod` | `( a b -- rem quot )` | Remainder and quotient |
| `negate` | `( a -- -a )` | Negate |
| `abs` | `( a -- |a| )` | Absolute value |
| `min` | `( a b -- min )` | Smaller of the two |
| `max` | `( a b -- max )` | Larger of the two |
| `1+` | `( a -- a+1 )` | Increment |
| `1-` | `( a -- a-1 )` | Decrement |
| `2*` | `( a -- a*2 )` | Double |
| `2/` | `( a -- a>>1 )` | Halve (arithmetic shift) |
| `and` | `( a b -- a&b )` | Bitwise and |
| `or` | `( a b -- a|b )` | Bitwise or |
| `xor` | `( a b -- a^b )` | Bitwise exclusive or |
| `invert` | `( a -- ~a )` | Bitwise complement |
| `lshift` | `( n c -- n<<c )` | Shift left by `c` |
| `rshift` | `( n c -- n>>c )` | Logical shift right by `c` |
| `sqrt` | `( n -- isqrt )` | Integer square root, truncating |

`/`, `mod`, and `/mod` fail with `division by zero` when `b` is zero, and `division overflow` on the one case Rust's `checked_div` cannot represent (`i64::MIN / -1`).

```forth
17 5 /mod . .     \ prints 2 then 3
0x10 .            \ prints 16
10 3 mod .        \ prints 1
-7 2 / .          \ prints -3
```

`lshift` and `rshift` fail when the shift count is not in `0..64`. `rshift` is a logical (unsigned) shift; `2/` is arithmetic.

## Fixed-point math

There are no floats. Fractional values use a fixed-point representation with a scale of `1000`: the value `1.5` is stored as `1500`. Addition and subtraction work the same as integer math at this scale; multiplication and division need the dedicated words.

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `fixed` | `( n -- n*1000 )` | Integer to fixed |
| `int` | `( x -- x/1000 )` | Fixed to integer, truncating |
| `f*` | `( a b -- a*b )` | Fixed multiply, result scaled back down |
| `f/` | `( a b -- a/b )` | Fixed divide, result scaled back up |
| `f+` / `f-` | | Aliases of `+` and `-`, for readability with fixed values |
| `sin` | `( x -- s )` | Sine of `x` radians (in fixed), result in fixed |
| `cos` | `( x -- c )` | Cosine of `x` radians (in fixed), result in fixed |
| `tan` | `( x -- t )` | Tangent of `x` radians (in fixed), result in fixed |
| `f-sqrt` | `( x -- s )` | Square root of a fixed value, result in fixed |
| `pi` | | Constant, `3142` (roughly pi in fixed) |

```forth
1500 2000 f* .       \ 3.0 * 1.5 = prints 3000
1000 fixed int .     \ prints 1
pi sin .             \ pi is 3142, about 3.14 radians, so sin is almost 0
```

For a smooth bobbing offset, a common pattern — `sin` returns roughly `-1000..1000` in fixed-point, so multiply by the amplitude you want in pixels:

```forth
: bob-y
  time 200 / fixed sin 30 * 100 +
;
```

Then use `100 bob-y` as a y coordinate for a draw word.

## Random numbers

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `random` | `( n -- r )` | A number in `0..n-1`; `0` when `n <= 0` |
| `random-seed` | `( n -- )` | Seed the xorshift generator |

```forth
100 random .
1234 random-seed 100 random .
```

The generator is deterministic per seed and shared by the whole program.

## Stack manipulation

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `dup` | `( a -- a a )` | Duplicate the top value |
| `drop` | `( a -- )` | Discard the top value |
| `swap` | `( a b -- b a )` | Exchange the top two values |
| `over` | `( a b -- a b a )` | Copy the second value to the top |
| `rot` | `( a b c -- b c a )` | Rotate the third value to the top |
| `-rot` | `( a b c -- c a b )` | Rotate the top value to the third position |
| `2dup` | `( a b -- a b a b )` | Duplicate the top two values |
| `2drop` | `( a b -- )` | Discard the top two values |
| `nip` | `( a b -- b )` | Drop the second value |
| `tuck` | `( a b -- b a b )` | Copy the top below the second |
| `?dup` | `( a -- a a? )` | Duplicate only if nonzero |
| `depth` | `( -- n )` | Number of values on the data stack |
| `pick` | `( x_n ... x_1 x_0 u -- ... x_u )` | Copy the item `u` levels down (0 is the top) |

```forth
1 2 3 rot .s      \ <3> 2 3 1
1 2 3 -rot .s     \ <3> 3 1 2
5 3 pick .s       \ puts a copy of the third item on top
```

`pick` fails with `pick index out of range` for negative or too-large indexes.

## Return stack

A second stack exists for temporary values. It is also used implicitly for loop state, so be careful using it inside `do`/`loop` bodies — see [Control flow](#control-flow).

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `>r` | `( x -- )` | Move the top value to the return stack |
| `r>` | `( -- x )` | Move the top return-stack value back |
| `r@` | `( -- x )` | Copy the top return-stack value |
| `rdepth` | `( -- n )` | Number of values on the return stack |

```forth
10 >r 20 >r r@ r> r> + + .   \ prints 40
```

## Comparison and truth

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `=` | `( a b -- flag )` | `-1` if equal |
| `<>` | `( a b -- flag )` | `-1` if not equal |
| `<` | `( a b -- flag )` | `-1` if `a < b` |
| `>` | `( a b -- flag )` | `-1` if `a > b` |
| `<=` | `( a b -- flag )` | `-1` if `a <= b` |
| `>=` | `( a b -- flag )` | `-1` if `a >= b` |
| `0=` | `( n -- flag )` | `-1` if `n == 0` |
| `0<>` | `( n -- flag )` | `-1` if `n != 0` |
| `0<` | `( n -- flag )` | `-1` if `n < 0` |
| `0>` | `( n -- flag )` | `-1` if `n > 0` |
| `not` | `( n -- flag )` | Alias of `0=` |

## Output

Terminal output words are available everywhere, including headless mode, which makes them useful for debugging.

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `.` | `( n -- )` | Print an integer and a newline |
| `.s` | `( -- )` | Print the whole stack: `<depth> v1 v2 ...` |
| `cr` | | Print a newline |
| `space` | | Print one space |
| `type` | `( handle -- )` | Print a string from the pool |
| `emit` | `( code -- )` | Print one character from a Unicode code point |

```forth
.s cr
"count: " type 42 . cr
```

## Control flow

All control-flow words are structurally checked when a piece of code is compiled — at load time for top-level code, and at definition time for `:` bodies. Mismatched pairs fail before any of the code runs.

### `if`, `else`, `then`

```forth
<condition>
if <true branch> else <false branch> then
```

`else` is optional. `if` pops one value; `0` jumps to the `else` branch or past `then`, nonzero continues into the `true` branch.

```forth
x @ 100 > if 100 x ! then
```

Nesting is allowed to any depth.

### `begin` and `until`

```forth
begin <body> <condition> until
```

The body always runs at least once. `until` pops a flag; `0` jumps back to `begin`, nonzero falls through.

### `begin`, `while`, `repeat`

```forth
begin <condition> while <body> repeat
```

`while` pops a flag; `0` jumps past `repeat`. `repeat` jumps back to `begin`.

```forth
: up-count
  0
  begin dup 5 < while 1+ repeat
;

up-count .   \ prints 5
```

### `do`, `?do`, `loop`, `+loop`

```forth
<limit> <start> do <body> loop
```

`do` pops `start`, then `limit`, and runs the body. `loop` adds `1` and loops while the index is less than the limit. `+loop` pops an increment instead: with a positive value it loops while `index < limit`, with a negative value while `index > limit`. A `0` increment is an error (`zero increment in +loop`).

`?do` is the same except that when `start` equals `limit` the body is skipped entirely. Plain `do` always runs the body at least once, even when `start >= limit`.

Inside the body, `i` pushes the index and `j` pushes the enclosing loop's index.

```forth
: table
  10 0 do
    i i 100 + 5 5 rl-red draw-circle
  loop
;
```

Simple examples that work as-is:

```forth
10 0 do i . loop               \ prints 0 through 9
5 0 do 100 i 10 * + 3 3 rl-red draw-circle loop
```

```forth
2 0 do 10 0 do i j * . loop cr loop
```

`leave` exits the innermost loop immediately, jumping to just after its `loop`/`+loop`:

```forth
: find-five
  10 0 do i 5 = if leave then loop
;
```

If `leave` or `exit` appears in a lifecycle word (`init`, `update`, `draw`) outside of any loop, load or the frame fails with an error.

`i` and `j` outside any loop fail with `i outside loop` / `j outside a nested loop`.

## Word definitions

```forth
: name <body> ;
```

At definition time, the body tokens up to `;` are captured and compile-checked, but not executed. Calling the word runs the body.

```forth
: square dup * ;
7 square .        \ prints 49

: greet "hello" type cr ;
greet
```

### Redefinition

A later definition with the same name replaces the earlier one. Since top-level code runs once, redefinition behavior is the same as standard Forth: words defined later in the file win from that point on.

### Forward references

Because the body is resolved at call time, a word may call a word defined later in the file, as long as the later definition exists before the first call happens.

### Self-calls and `recurse`

A word can call itself by name, or with `recurse`. The interpreter enforces a recursion limit (4096 nested calls) and fails with `recursion limit exceeded (4096)` rather than crashing the host.

```forth
: countdown
  dup 0 > if
    dup . 1- recurse
  else
    drop
  then
;

5 countdown
```

### `exit`

`exit` stops executing the current word immediately and returns to the caller.

```forth
: early 1 2 exit 3 + ;
early .s      \ <2> 1 2
```

## Memory and variables

All named storage lives in one flat arena of `i64` cells. An address is a plain index into that arena.

### `variable`

```forth
variable name
```

Allocates one cell (initialized to `0`) and makes `name` push its address. Executing `name` pushes the address; `@` reads it. If `name` already refers to a variable (for instance in a file that gets included twice), the declaration does nothing and the existing cell is kept.

### `@`, `!`, `+!`, and friends

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `@` | `( addr -- value )` | Fetch a cell |
| `!` | `( value addr -- )` | Store a cell |
| `+!` | `( value addr -- )` | Add to a cell |
| `here` | `( -- addr )` | Address of the next free cell |
| `allot` | `( n -- )` | Reserve `n` more cells (negative shrinks) |
| `,` | `( x -- )` | Store `x` at `here` and advance by one cell |
| `cells` | `( n -- n )` | Cell size multiplier (currently `1`) |
| `cell+` | `( addr -- addr+1 )` | Address of the next cell |

### `create` and arrays

```forth
create name
```

Defines `name` to push the current `here`, without allocating anything. Follow it with `allot`, `,`, or both:

```forth
create scores 10 cells allot
42    scores 0 cells + !
7     scores 3 cells + !

scores @ .              \ 42, the first cell
scores 3 cells + @ .    \ 7, the fourth cell
```

A common pattern is a helper word that turns an index into an address:

```forth
: score ( i -- addr ) cells scores + ;

5 score !
5 score @ .
```

### `constant` and `value`

```forth
100 constant max-hp
5 value level

level 1 + to level
level .          \ 6
max-hp .         \ 100
```

`to` stores into either a `value` or a `variable` address. `constant` and `value` pop their value from the stack at declaration time.

### Address safety

`@`, `!`, and `+!` bounds-check every access. A bad address fails with `invalid address: n`; it is never memory corruption.

## Strings

String literals push integer handles into a string pool. The pool persists for the lifetime of the interpreter and only ever grows.

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `type` | `( handle -- )` | Print the string to stdout |
| `string-length` | `( handle -- n )` | Number of characters (chars, not bytes) |
| `draw-text` | `( handle x y size -- )` | Queue a text draw command |
| `text-width` | `( handle size -- pixels )` | Approximate width: `chars * size / 2` |

```forth
"score" string-length .
"hello" 10 10 20 rl-white draw-text
```

There are no string operations (concatenation, slicing, comparison); strings are meant for display and light debugging. Comparing handles with `=` compares pool identity.

## Input

The host takes a snapshot of keyboard and mouse state every frame and hands it to the interpreter before `update` runs.

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `key-down?` | `( key -- flag )` | Key is currently held |
| `key-pressed?` | `( key -- flag )` | Key went down this frame |
| `mouse-x` | `( -- x )` | Mouse position |
| `mouse-y` | `( -- y )` | Mouse position |
| `mouse-wheel` | `( -- delta )` | Wheel movement this frame |
| `mouse-down?` | `( button -- flag )` | Button is currently held |
| `mouse-pressed?` | `( button -- flag )` | Button went down this frame |

Keys are compared against named constants (see the [word reference](words.md#constants); `key-left`, `key-space`, `key-a`, ...):

```forth
: update
  key-left  key-down? if player-x @ 5 - player-x ! then
  key-right key-down? if player-x @ 5 + player-x ! then
;
```

In headless mode the snapshot is empty, so all queries return false or zero.

## Time

| Word | Stack effect | Meaning |
| --- | --- | --- |
| `dt` | `( -- ms )` | Milliseconds since the previous frame |
| `time` | `( -- ms )` | Milliseconds since startup |
| `frame` | `( -- n )` | Number of frames since startup |

Use `dt` to scale movement by real elapsed time:

```forth
: update
  player-speed @ dt 16 / * player-x +! ...
;
```

Headless mode uses a fixed `dt` of 16 ms.

## Includes

```forth
include "path/to/file.fth"
```

Loads and runs another file immediately, as if its tokens were spliced in at that point:

- The path is resolved relative to the file containing the `include`, then to the main source file's directory. Absolute paths are used as-is.
- Includes can nest; an included file includes its own files.
- A file directly including itself (directly or transitively) fails with `include cycle detected`.
- Definitions and declarations inside an included file find the same global dictionary.

```forth
\ main.fth
include "lib/starfield.fth"
init-stars
```

Included errors are reported with the file and line of the offending token.

## Errors

Every error carries an optional location and a call stack. It prints like this:

```text
<file>:<line>:<col>: unknown word: draw-rectangel
  in bad at <file>:<line>:<col>
```

Common errors:

| Error | Cause |
| --- | --- |
| `stack underflow` | A word needed more values than were on the stack |
| `division by zero` | `/`, `mod`, `/mod`, or `f/` with a zero divisor |
| `division overflow` | `i64::MIN / -1` |
| `unknown word: w` | Token that is not a number, string, or dictionary entry |
| `invalid address: n` | `@`, `!`, or `+!` outside the arena |
| `invalid string handle: n` | A handle that does not exist in the pool |
| `recursion limit exceeded (4096)` | Deep unbounded recursion |
| `i outside loop` / `j outside a nested loop` | Loop index words used outside loops |
| `zero increment in +loop` | `+loop` with increment `0` |
| `leave outside loop` | `leave` not inside any `do`/`begin` loop |
| `missing then for if`, `missing loop for do`, ... | Structural mismatch found at compile time |
| `unexpected ;` | A `;` with no definition open |
| `include cycle detected: path` | Include graph contains a loop |
| `cannot include 'path': ...` | The file could not be opened |

There is no error recovery inside Forth; the first error stops execution (and the frame, in windowed mode).

## Limitations

- One number type: `i64`. No floats.
- No string operations beyond length, text drawing, and printing.
- No locals, no wordlist scopes; one global dictionary.
- No textures, sprites, shaders, or cameras.
- No sound.
- No file I/O from Forth (includes aside).
- `text-width` is an estimate, not a font measurement.
- `do` always runs its body at least once; use `?do` for the skip-if-empty case.
- The return stack shares bookkeeping with loop state; using it brazenly inside loops can break `loop` bookkeeping.
