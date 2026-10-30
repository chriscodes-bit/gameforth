# Examples

This page contains complete, runnable GameForth programs.

The application always loads `examples/test.fth` at startup. To run one of these examples, replace the contents of `examples/test.fth` with the example and run:

```powershell
cargo run
```

None of the programs below use comments, because GameForth does not support them.

## Contents

- [Static rectangle](#static-rectangle)
- [Multiple shapes and colors](#multiple-shapes-and-colors)
- [Defining and calling a word](#defining-and-calling-a-word)
- [Conditionals](#conditionals)
- [Nested conditionals](#nested-conditionals)
- [Counting loop](#counting-loop)
- [Variables and score](#variables-and-score)
- [Moving rectangle with wrap](#moving-rectangle-with-wrap)
- [Bouncing rectangle](#bouncing-rectangle)
- [Multiple moving objects](#multiple-moving-objects)
- [Useful arithmetic idioms](#useful-arithmetic-idioms)

## Static rectangle

```forth
350 250 100 100 rl-red draw-rectangle
```

Draws a filled red `100x100` rectangle with its top-left at `(350, 250)`.

The rectangle is redrawn in the same place every frame.

## Multiple shapes and colors

```forth
50 50 100 100 rl-red draw-rectangle
200 50 100 100 rl-blue draw-rectangle
50 200 50 rl-yellow draw-circle
400 300 80 rl-purple draw-circle
```

This queues four shapes:

- a red rectangle at `(50, 50)`
- a blue rectangle at `(200, 50)`
- a yellow circle centered at `(50, 200)`
- a purple circle centered at `(400, 300)`

The shapes are drawn in order, so later shapes appear on top of earlier ones.

## Defining and calling a word

```forth
: square
  dup *
;

: double
  2 *
;

7 square .
21 double .
```

Output:

```text
49
42
```

`square` duplicates the top value and multiplies it by itself. `double` multiplies the top value by `2`.

## Conditionals

```forth
10 5 > if
  1 .
else
  0 .
then
```

`10 5 >` produces `-1`, so the program prints `1`.

```forth
10 5 < if
  1 .
else
  0 .
then
```

`10 5 <` produces `0`, so the program prints `0`.

## Nested conditionals

```forth
variable n
7 n !

n @ 5 > if
  n @ 10 > if
    2 .
  else
    1 .
  then
else
  0 .
then
```

`n` is `7`:

- `7 > 5` is true, so the outer true branch runs.
- `7 > 10` is false, so the inner false branch runs.
- The program prints `1`.

## Counting loop

```forth
variable n
5 n !

begin
  n @ .
  -1 n +!
  n @ 0 <
until
```

Output:

```text
5
4
3
2
1
0
```

`until` loops while its condition is `0`. After `n` reaches `-1`, `n @ 0 <` produces `-1`, so the loop exits.

## Variables and score

```forth
variable score
0 score !
10 score +!
5 score +!
score @ .
```

Output:

```text
15
```

`variable score` creates a cell initialized to `0`. `!` stores, `+!` adds, and `@` fetches.

## Moving rectangle with wrap

This program moves a rectangle to the right and wraps it back to the left edge when it reaches the right side.

```forth
variable x
variable initialized

initialized @ 0= if
  0 x !
  -1 initialized !
then

x @ 1 + x !

x @ 700 > if
  0 x !
then

x @ 250 100 100 rl-red draw-rectangle
```

How it works:

1. On the first frame, `initialized` is `0`, so the initialization block sets `x` to `0` and marks `initialized` as true.
2. `x @ 1 + x !` increments `x` by one.
3. If `x` is greater than `700`, it is reset to `0`.
4. The rectangle is drawn at the new `x`.

The valid x range for a 100-pixel-wide rectangle is `0..700`.

## Bouncing rectangle

This is the program currently in `examples/test.fth`.

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

Variables:

- `x`, `y` — top-left position
- `dx`, `dy` — velocity per frame
- `initialized` — one-time setup flag

Each frame:

1. On the first frame, set the starting position and velocity, then set `initialized` to `-1`.
2. Add the velocity to the position.
3. If the rectangle crosses a boundary, clamp it to the boundary and negate the corresponding velocity.
4. Draw the rectangle at the updated position.

The rectangle moves diagonally at `4` pixels per frame horizontally and `3` pixels per frame vertically. At 60 FPS, that is about `240` pixels per second horizontally and `180` pixels per second vertically.

## Multiple moving objects

```forth
variable x1
variable y1
variable x2
variable y2
variable initialized

initialized @ 0= if
  100 x1 !
  200 y1 !
  400 x2 !
  300 y2 !
  -1 initialized !
then

x1 @ 1 + x1 !
x2 @ -1 + x2 !

x1 @ y1 @ 60 60 rl-green draw-rectangle
x2 @ y2 @ 80 80 rl-orange draw-rectangle
```

This moves one rectangle to the right and the other to the left. Add bounds checks if they should bounce instead of leaving the screen.

## Useful arithmetic idioms

### Negate a value

```forth
5 -1 * .
```

Output:

```text
-5
```

### Compute a remainder

There is no `mod` word. Compute `17 mod 5` with:

```forth
17 dup 5 / 5 * - .
```

Output:

```text
2
```

### Clamp a variable

```forth
x @ 0 < if
  0 x !
then

x @ 700 > if
  700 x !
then
```

### Reflect a velocity

```forth
x @ 700 > if
  700 x !
  -1 dx @ * dx !
then
```

### Print debug state

```forth
x @ .
```

This prints the value of `x` to stdout. It runs every frame, so use it only temporarily.
