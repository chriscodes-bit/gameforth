# Word Reference

This page is the reference for every word currently available in GameForth.

## Contents

- [Notation](#notation)
- [Word kinds](#word-kinds)
- [Arithmetic words](#arithmetic-words)
- [Fixed-point words](#fixed-point-words)
- [Float and math function words](#float-and-math-function-words)
- [Random words](#random-words)
- [Stack words](#stack-words)
- [Return stack words](#return-stack-words)
- [Comparison words](#comparison-words)
- [Output words](#output-words)
- [Graphics words](#graphics-words)
- [Color words](#color-words)
- [Input words](#input-words)
- [Time words](#time-words)
- [Memory words](#memory-words)
- [Constants](#constants)
- [Control-flow forms](#control-flow-forms)
- [Definition forms](#definition-forms)
- [Sugar forms](#sugar-forms)
- [Error summary](#error-summary)

## Notation

Stack effects are written as:

```text
( before -- after )
```

The rightmost item is the top of the stack. `n` means a signed 64-bit integer, `x` means a number of either kind, `f` means a float result. `flag` means a truth value: `0` is false, any nonzero is true. Comparison words produce `-1` for true and `0` for false. `handle` means a string-pool handle, `addr` means a memory address, `key` means a key code, `button` means a mouse button code.

## Word kinds

| Kind | Examples | How it is resolved |
| --- | --- | --- |
| Builtin word | `+`, `dup`, `draw-rectangle`, `key-down?` | Function stored in the dictionary, executed with the runtime context |
| User-defined word | Anything made with `: name ... ;` | Compiled token list with its own control-flow table, called recursively with a depth limit |
| Color word | `rl-red`, `rl-skyblue` | Dictionary entry that sets the current drawing color |
| Address word | `variable x`, `create arr` | Dictionary entry that pushes an arena index |
| Constant word | `pi`, `e`, `key-space`, `max-hp` from `constant` | Dictionary entry that pushes a fixed value (integer or float) |
| Value word | Anything made with `value` | Dictionary entry that pushes a memory cell's contents |
| Interpreter form | `if`, `do`, `:`, `to`, `include` | Handled directly by the interpreter, never dictionary entries |

## Arithmetic words

Most arithmetic words are polymorphic: two integers stay integers (wrapping on overflow); if any operand is a float, the operands are promoted to `f64` and the result is a float. `mod`, `/mod`, `2*`, `2/`, and the bitwise words require integers and reject floats with `expected an integer, got a float (convert with int-of)`.

| Word | Stack effect | Summary |
| --- | --- | --- |
| `+` | `( x y -- x+y )` | Add; promotes to float if either operand is a float |
| `-` | `( x y -- x-y )` | Subtract; same promotion |
| `*` | `( x y -- x*y )` | Multiply; same promotion |
| `/` | `( x y -- x/y )` | Integers: truncating division; floats: IEEE division, no zero-divisor error |
| `mod` | `( n m -- n%m )` | Remainder, integers only |
| `/mod` | `( n m -- rem quot )` | Remainder and quotient, integers only |
| `negate` | `( x -- -x )` | Negate |
| `abs` | `( a -- |a| )` | Absolute value |
| `min` | `( x y -- min )` | Minimum; promotes like the arithmetic words |
| `max` | `( x y -- max )` | Maximum; promotes like the arithmetic words |
| `1+` | `( x -- x+1 )` | Increment |
| `1-` | `( x -- x-1 )` | Decrement |
| `2*` | `( n -- n*2 )` | Double, integers only |
| `2/` | `( n -- n>>1 )` | Arithmetic halve, integers only |
| `and` | `( n m -- n&m )` | Bitwise and |
| `or` | `( n m -- n|m )` | Bitwise or |
| `xor` | `( n m -- n^m )` | Bitwise exclusive or |
| `invert` | `( n -- ~n )` | Bitwise complement |
| `lshift` | `( n c -- n<<c )` | Shift left |
| `rshift` | `( n c -- n>>c )` | Logical shift right |
| `sqrt` | `( x -- s )` | Integers: truncating integer root; floats: float square root; negative input is an error |

Integer arithmetic wraps on overflow. `/`, `mod`, `/mod` fail on a zero integer divisor and on `i64::MIN / -1`. `lshift`/`rshift` require `0 <= c < 64`; `rshift` treats `n` as unsigned.

## Fixed-point words

Fractional math can use a scale of `1000` on integers: `1.5` is `1500`. These words are integer-only; see the float words below for plain float math.

| Word | Stack effect | Summary |
| --- | --- | --- |
| `fixed` | `( n -- n*1000 )` | Integer to fixed |
| `int` | `( x -- x/1000 )` | Fixed to integer, truncating |
| `f*` | `( a b -- product )` | Fixed multiply |
| `f/` | `( a b -- quotient )` | Fixed divide |
| `f+` | `( a b -- sum )` | Alias of `+` |
| `f-` | `( a b -- diff )` | Alias of `-` |
| `sin` | `( r -- s )` | Sine of `r` radians (`r` in fixed), result in fixed |
| `cos` | `( r -- c )` | Cosine, same conventions |
| `tan` | `( r -- t )` | Tangent, same conventions |
| `f-sqrt` | `( x -- s )` | Square root of a fixed value, result in fixed |

`sin`, `cos`, and `tan` take radians in fixed-point for integer arguments: `0` is `0`, `pi` (the constant `3142`) is roughly one pi. A float argument switches to plain float math and returns a float. `f-sqrt` requires `x >= 0`.

## Float and math function words

| Word | Stack effect | Summary |
| --- | --- | --- |
| `int-of` | `( x -- n )` | Float to integer, truncating toward zero; an integer passes through unchanged |
| `exp` | `( x -- f )` | e^x, as a float |
| `ln` | `( x -- f )` | Natural logarithm, as a float |
| `log10` | `( x -- f )` | Base-10 logarithm, as a float |
| `log2` | `( x -- f )` | Base-2 logarithm, as a float |
| `pow` | `( x y -- f )` | Two integers stay integer (wrapping; negative exponent is an error); otherwise `powf`, as a float |
| `floor` | `( x -- y )` | Largest integer value not above `x`, as a float; an integer input is untouched |
| `ceil` | `( x -- y )` | Smallest integer value not below `x`, as a float; an integer input is untouched |
| `round` | `( x -- y )` | Nearest integer value (half away from zero), as a float; an integer input is untouched |
| `trunc` | `( x -- y )` | Integer part of `x`, as a float; an integer input is untouched |
| `asin` | `( x -- f )` | Arc sine, radians, as a float |
| `acos` | `( x -- f )` | Arc cosine, radians, as a float |
| `atan` | `( x -- f )` | Arc tangent, radians, as a float |
| `atan2` | `( x y -- f )` | Angle of the point `(x, y)`; `y` is on top of the stack |
| `sinh` | `( x -- f )` | Hyperbolic sine, as a float |
| `cosh` | `( x -- f )` | Hyperbolic cosine, as a float |
| `tanh` | `( x -- f )` | Hyperbolic tangent, as a float |
| `hypot` | `( x y -- f )` | `sqrt(x^2 + y^2)` without intermediate overflow, as a float |

Everything in this table except `int-of` (float to integer) produces float results. Integer inputs are promoted to `f64` first, so `100 log10` gives `2.0`. The rounding words `floor`/`ceil`/`round`/`trunc` are the exception: they only do work on floats and hand integers straight back, so they are safe to sprinkle into integer code.

```forth
e ln .              \ ln of e, prints 1
2 10 pow .          \ integer power, prints 1024
2.0 0.5 pow .       \ float power, prints 1.4142135623730951
3.7 round .         \ prints 4 (a float)
3.7 int-of .        \ prints 3 (an integer)
```

## Random words

| Word | Stack effect | Summary |
| --- | --- | --- |
| `random` | `( n -- r )` | Uniform in `0..n-1`; `0` when `n <= 0` |
| `random-seed` | `( n -- )` | Seed the xorshift generator |

There is no built-in two-argument range word; the usual idiom defines one from `random`:

```forth
: random-between ( lo hi -- n ) over - random + ;

50 400 random-between .     \ somewhere in 50..399
```

## Stack words

| Word | Stack effect | Summary |
| --- | --- | --- |
| `dup` | `( a -- a a )` | Duplicate top |
| `drop` | `( a -- )` | Remove top |
| `swap` | `( a b -- b a )` | Exchange top two |
| `over` | `( a b -- a b a )` | Copy second to top |
| `rot` | `( a b c -- b c a )` | Rotate third to top |
| `-rot` | `( a b c -- c a b )` | Rotate top to third |
| `2dup` | `( a b -- a b a b )` | Duplicate top two |
| `2drop` | `( a b -- )` | Drop two |
| `nip` | `( a b -- b )` | Drop second |
| `tuck` | `( a b -- b a b )` | Copy top under second |
| `?dup` | `( a -- a a? )` | Duplicate only if nonzero (floats count by `!= 0.0`) |
| `depth` | `( -- n )` | Data stack depth |
| `pick` | `( x_n ... x_0 u -- ... x_u )` | Copy the item `u` levels down |

Stack words move values without caring about their kind, so they work on integers and floats alike. `pick` counts from the top with 0, so `0 pick` is a slow `dup`. Negative or out-of-range indexes fail.

## Return stack words

| Word | Stack effect | Summary |
| --- | --- | --- |
| `>r` | `( x -- )` | Push onto return stack |
| `r>` | `( -- x )` | Pop from return stack |
| `r@` | `( -- x )` | Copy return-stack top |
| `rdepth` | `( -- n )` | Return stack depth |

The return stack is a plain secondary stack. Loop state is kept in a separate loop stack, so `>r`/`r>` inside `do` loops cannot corrupt loop indexes, but pairing mistakes still fail with `return stack underflow`.

## Comparison words

All produce `-1` or `0`. Comparison words are polymorphic: two integers compare exactly, and if either operand is a float both are promoted to `f64` (so integers above 2^53 lose exactness in a mixed comparison). The zero-comparisons (`0=`, `0<>`, `0<`, `0>`, and `not`, which is an alias of `0=`) treat a float as truthy when it is not exactly `0.0`.

| Word | Stack effect | Summary |
| --- | --- | --- |
| `=` | `( a b -- flag )` | Equal |
| `<>` | `( a b -- flag )` | Not equal |
| `<` | `( a b -- flag )` | Less than |
| `>` | `( a b -- flag )` | Greater than |
| `<=` | `( a b -- flag )` | Less or equal |
| `>=` | `( a b -- flag )` | Greater or equal |
| `0=` | `( n -- flag )` | Is zero |
| `0<>` | `( n -- flag )` | Not zero |
| `0<` | `( n -- flag )` | Negative |
| `0>` | `( n -- flag )` | Positive |
| `not` | `( n -- flag )` | Alias of `0=` |

## Output words

| Word | Stack effect | Summary |
| --- | --- | --- |
| `.` | `( x -- )` | Print a number (integer or float) and a newline |
| `.s` | | Print the whole stack: `<depth> v1 v2 ...`, integers and floats alike |
| `cr` | | Newline |
| `space` | | One space |
| `type` | `( handle -- )` | Print a pool string |
| `emit` | `( code -- )` | Print one character from a code point |

These write to stdout. In windowed mode they share the console with reload/error messages.

## Graphics words

Coordinates are integer pixels. All commands queue; they render after `draw` finishes. Colors come from the current color set by color words (see the note on ordering below).

| Word | Stack effect | Summary |
| --- | --- | --- |
| `draw-rectangle` | `( x y w h -- )` | Filled rectangle, top-left corner |
| `draw-rectangle-lines` | `( x y w h -- )` | Rectangle outline |
| `draw-circle` | `( x y r -- )` | Filled circle, center point |
| `draw-circle-lines` | `( x y r -- )` | Circle outline |
| `draw-line` | `( x1 y1 x2 y2 -- )` | Line segment |
| `draw-text` | `( handle x y size -- )` | Text, top-left corner |
| `draw-number` | `( n x y size -- )` | A number rendered as text; coordinates and size stay integer-only |
| `string-length` | `( handle -- n )` | Characters in a pool string |
| `text-width` | `( handle size -- pixels )` | Approximate width: `chars * size / 2` |

Color word note: a color word changes the interpreter's current color; every command queued afterwards uses it. Put the color after the number arguments and before the draw word:

```text
<number arguments> <rl-color> <drawing word>
```

For example `100 100 40 40 rl-red draw-rectangle`, not `rl-red 100 100 40 40 draw-rectangle`.

`text-width` does not measure the real font. It returns `chars * size / 2`, which is close for the default font but worth recalibrating in your own code if you need precision.

## Color words

All are named `rl-<color>`, matching raylib's palette:

`rl-lightgray`, `rl-gray`, `rl-darkgray`, `rl-yellow`, `rl-gold`, `rl-orange`, `rl-pink`, `rl-red`, `rl-maroon`, `rl-green`, `rl-lime`, `rl-darkgreen`, `rl-skyblue`, `rl-blue`, `rl-darkblue`, `rl-purple`, `rl-violet`, `rl-darkpurple`, `rl-beige`, `rl-brown`, `rl-darkbrown`, `rl-white`, `rl-black`, `rl-magenta`, `rl-raywhite`.

There are no words for blending or arbitrary RGB colors. Custom colors can be approximated by layering primitives, or by adding a word in `src/dictionary.rs`.

## Input words

| Word | Stack effect | Summary |
| --- | --- | --- |
| `key-down?` | `( key -- flag )` | Key currently held |
| `key-pressed?` | `( key -- flag )` | Key went down this frame |
| `mouse-x` | `( -- x )` | Mouse x |
| `mouse-y` | `( -- y )` | Mouse y |
| `mouse-wheel` | `( -- delta )` | Wheel movement this frame |
| `mouse-down?` | `( button -- flag )` | Button currently held |
| `mouse-pressed?` | `( button -- flag )` | Button went down this frame |

In headless mode every query returns `0`.

## Time words

| Word | Stack effect | Summary |
| --- | --- | --- |
| `dt` | `( -- ms )` | Milliseconds since the previous frame |
| `time` | `( -- ms )` | Milliseconds since startup |
| `frame` | `( -- n )` | Frames since startup |

In headless mode, `dt` is 16 and `time` advances 16 ms per frame.

## Memory words

| Word | Stack effect | Summary |
| --- | --- | --- |
| `@` | `( addr -- value )` | Fetch |
| `!` | `( value addr -- )` | Store |
| `+!` | `( value addr -- )` | Add-in-place |
| `here` | `( -- addr )` | Next free cell |
| `allot` | `( n -- )` | Reserve (or shrink by) `n` cells |
| `,` | `( x -- )` | Append a cell |
| `cells` | `( n -- n )` | Cell count (currently the identity) |
| `cell+` | `( addr -- addr+1 )` | Next cell |

Addresses are indices into one flat `i64` arena, always bounds-checked: `invalid address: n` rather than a crash.

## Constants

### Builtin constants

| Name | Value |
| --- | --- |
| `true` | `-1` |
| `false` | `0` |
| `pi` | `3142` (fixed-point pi) |
| `e` | `2.718281828459045` (a float) |

Floating-point constants can also be named with `constant`: `1.5 constant half` stores a float half and pushes it back as a float.

### Key constants

`key-a` through `key-z` (65–90), `key-0` through `key-9` (48–57), and:

`key-space`, `key-escape`, `key-enter`, `key-tab`, `key-backspace`, `key-insert`, `key-delete`, `key-right`, `key-left`, `key-down`, `key-up`, `key-page-up`, `key-page-down`, `key-home`, `key-end`, `key-caps-lock`, `key-scroll-lock`, `key-num-lock`, `key-print-screen`, `key-pause`, `key-f1` through `key-f12`, `key-left-shift`, `key-left-control`, `key-left-alt`, `key-right-shift`, `key-right-control`, `key-right-alt`, plus the shorthand `key-shift`, `key-control`, `key-alt` (identical to the left variants), and punctuation: `key-comma`, `key-period`, `key-minus`, `key-equal`, `key-slash`, `key-semicolon`, `key-apostrophe`, `key-grave`, `key-left-bracket`, `key-right-bracket`, `key-backslash`.

The codes match raylib's `KeyboardKey` values, so they also work via `rl.is_key_down` if you extend the host.

### Mouse button constants

`mouse-left` (0), `mouse-right` (1), `mouse-middle` (2), `mouse-side` (3), `mouse-extra` (4), `mouse-forward` (5), `mouse-back` (6).

## Control-flow forms

These are interpreter forms, not dictionary words. They are structurally paired at compile time.

| Form | Pairs with | Notes |
| --- | --- | --- |
| `if` | `else` (optional), `then` | Pops a flag |
| `begin` | `until`, or `while` ... `repeat` | Post-test and pre-test loops |
| `do` (`?do`) | `loop`, `+loop` | `( limit start -- )`; `?do` skips empty ranges |
| `leave` | — | Exits the innermost loop immediately |
| `exit` | — | Returns from the current word |
| `i` | — | Current loop index |
| `j` | — | Enclosing loop index |
| `recurse` | — | Calls the current word again |

Notes:

- `loop` adds 1 per iteration and continues while `index < limit`; `+loop` pops a signed increment and compares accordingly, rejecting an increment of `0`.
- Plain `do` runs its body at least once even when `start >= limit`; `?do` does not.
- `leave`/`exit` used outside their structures in `init`, `update`, or `draw` are reported as errors.
- Nested loops: `i` is the innermost index, `j` the enclosing one.

## Definition forms

| Form | Meaning |
| --- | --- |
| `: name ... ;` | Define a word |
| `variable name` | One cell, zero-initialized; `name` pushes its address |
| `create name` | `name` pushes `here`; allocate separately |
| `constant name` | Integer or float constant from the stack; a float value makes a float constant |
| `value name` | Like `variable` but `name` pushes the contents |
| `to name` | `( x -- )` store into a `value` or into a variable at its address |
| `include "path"` | Load and run another file inline |

`variable` is harmless to repeat: if the name is already an address word, the existing cell is kept. This makes double-`include` safe for declarations, though definitions would just be re-created.

## Sugar forms

| Form | Meaning |
| --- | --- |
| `"text"` | String literal; pushes a pool handle |
| `s" text"` | Same, traditional spelling |
| `\` | Line comment |
| `( ... )` | Block comment |
| `0x`, `$`, `0b`, `%` | Hex and binary number prefixes |

## Error summary

| Error | Typical cause |
| --- | --- |
| `stack underflow` | Popped an empty stack |
| `division by zero` | `/`, `mod`, `/mod` with a zero integer divisor (float `/` follows IEEE semantics instead) |
| `expected an integer, got a float (convert with int-of)` | A float reached an integer-only word (memory, `mod`, loops, `random`, `emit`, coordinates, `>r`) |
| `negative exponent in pow (use floats)` | `pow` with two integer operands and a negative exponent |
| `division overflow` | `i64::MIN / -1` |
| `unknown word: w` | Misspelling or missing `include` |
| `invalid address: n` | `@`/`!`/`+!` outside the arena |
| `invalid string handle: n` | Stale or garbage handle |
| `return stack underflow` | Unbalanced `>r`/`r>` |
| `recursion limit exceeded (4096)` | Runaway recursion |
| `pick index out of range` | Bad `pick` index |
| `shift count out of range` | Shift not in `0..64` |
| `sqrt of a negative number` / `f-sqrt of a negative number` | Bad argument |
| `zero increment in +loop` | `+loop` with `0` |
| `i outside loop`, `j outside a nested loop` | Loop index misuse |
| `leave outside loop` | `leave` outside any loop; also reported per lifecycle word |
| `missing then for if`, `missing until or repeat for begin`, `missing loop for do`, `else without if`, `while without begin`, `duplicate else`, `then without if`, `repeat without while`, `repeat without begin`, `until without begin`, `loop without do`, `do without loop` | Compile-time structural mismatch |
| `unexpected ;` | `;` outside a definition |
| `expected word name after :` and friends | Definitions missing their name token |
| `missing ; in definition` | Definition that never closes |
| `to: w is not a value or variable` | `to` on the wrong kind of word |
| `expected path string after include` | `include` without a string path |
| `cannot include 'path': ...` | File could not be opened or read |
| `include cycle detected: path` | Recursive includes |
