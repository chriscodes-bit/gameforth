# Examples

This page documents the programs that ship with the repository plus a few short fragments worth stealing.

## Contents

- [Running the examples](#running-the-examples)
- [Bouncing rectangle](#bouncing-rectangle)
- [Modular starfield demo](#modular-starfield-demo)
- [Console language tour](#console-language-tour)
- [How `include` composes the demo](#how-include-composes-the-demo)
- [Fragments](#fragments)

## Running the examples

```sh
cargo run                                        # examples/test.fth, windowed
cargo run -- examples/demo/main.fth              # the demo, windowed
cargo run -- examples/tour.fth --headless        # the language tour, console only
cargo run -- --headless --frames 60 examples/test.fth
```

## Console language tour

`examples/tour.fth` is a complete language tour that uses **no graphics or game words at all** — it is all top-level code, so it runs once at load with no `init`/`update`/`draw`. Run it headless:

```sh
cargo run -- --headless examples/tour.fth
```

It exercises, in one self-checking run:

- string literals and `emit`-based character output (`emit-digits`, a small recursion example)
- arithmetic, bit operations, and integer `sqrt`
- every stack word, demonstrated with `.s`
- `value`/`to`, `variable`/`!`/`+!`
- `do`/`loop`, `?do` vs `do`, `+loop` with a negative step, `leave`, nested `i`/`j`
- recursion (`gcd`, `fib`, factorial) and `exit`
- memory arrays (`create`/`allot`/`,`), a Sieve of Eratosthenes, and an in-place bubble sort
- fixed-point math: `f*`, `f/`, `sin`, `f-sqrt`, mean/variance/stddev of a dataset
- deterministic `random`/`random-seed` sequences

The last section prints the final stack depth, which must be `0` — every fragment in the program is stack-balanced, so any imbalance fails the self-check.

Anything can be pointed at any file:

```sh
gameforth path/to/program.fth
```

The program is loaded once: top-level code and `init` run, then the host loops `update`/`draw` every frame. In windowed mode, saving the file reloads the program (see the [CLI reference](README.md#cli-reference)).

## Bouncing rectangle

`examples/test.fth` — the whole program:

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

Walking through it:

- `variable x` ... declares four cells. Top-level runs once, so they are allocated exactly once.
- `: init` seeds positions and velocities. At load time the host calls `init` once, so the square starts at `(350, 250)`.
- `: update` moves first — `dx @ x +!` is `( value addr -- )` — then checks each wall. When a wall is crossed the position is clamped and the velocity flips using `negate` instead of `-1 *`.
- The wall limits (700 and 500) are tuned for a 100-wide square in an 800x600 window: `800 - 100 = 700`, `600 - 100 = 500`.
- `: draw` first renders a label with `draw-text` (the signature is `handle x y size`), then the rectangle — arguments come off the stack in `( x y w h )` order, with the color word after the numbers and before the draw word.

## Modular starfield demo

`examples/demo/main.fth` splits a program across three files:

```forth
\ examples/demo/main.fth
\ Starfield demo.
\ Run with: cargo run -- examples/demo/main.fth

include "lib/starfield.fth"

variable player-x
variable player-y

: init
  init-stars
  400 player-x !
  300 player-y !
;

: update
  update-stars

  key-left  key-down? if player-x @ 5 - player-x ! then
  key-right key-down? if player-x @ 5 + player-x ! then
  key-up    key-down? if player-y @ 5 - player-y ! then
  key-down  key-down? if player-y @ 5 + player-y ! then

  key-space key-pressed? if init-stars then
;

: draw
  draw-stars
  player-x @ player-y @ 18 rl-skyblue draw-circle
  player-x @ player-y @ 26 rl-darkblue draw-circle-lines

  "Arrow keys move. Space reseeds the stars." 20 20 20 rl-white draw-text
  "frame" 20 50 16 rl-gray draw-text
  frame 80 50 16 rl-gray draw-number
  "stars" 20 70 16 rl-gray draw-text
  star-count @ 80 70 16 rl-gray draw-number
;
```

The starfield lives in `lib/starfield.fth`, which includes one helper of its own:

```forth
\ examples/demo/lib/starfield.fth
\ A tiny starfield built from three parallel arrays.

include "random.fth"

create stars-x 80 cells allot
create stars-y 80 cells allot
create stars-speed 80 cells allot
variable star-count

: star-x ( i -- addr ) cells stars-x + ;
: star-y ( i -- addr ) cells stars-y + ;
: star-speed ( i -- addr ) cells stars-speed + ;

: init-stars
  80 0 do
    800 random i star-x !
    600 random i star-y !
    1 4 random-between i star-speed !
  loop
  80 star-count !
;

: update-stars
  star-count @ 0 do
    i star-y @ i star-speed @ + i star-y !
    i star-y @ 600 > if
      0 i star-y !
      800 random i star-x !
    then
  loop
;

: draw-stars
  star-count @ 0 do
    i star-x @ i star-y @ 2 rl-lightgray draw-circle
  loop
;
```

```forth
\ examples/demo/lib/random.fth
\ Random helpers shared by the demo.

: random-between ( lo hi -- n )
  over - random + ;
```

Each star is one index in three parallel arrays. `star-x` etc. are index-to-address helpers: `i star-x !` stores the loop counter's row. Things to notice:

- `create stars-x 80 cells allot` — `create` reserves nothing by itself; `allot` grows the arena.
- `include "random.fth"` inside `lib/starfield.fth` resolves relative to that file's directory, not the main file's, so the tree can be moved as a unit.
- `star-count @ 0 do ... loop` — the count is a variable, so the loop reads its size once at loop start.
- `key-space key-pressed?` reshuffles stars once per press, versus `key-down?` which would refill every frame it is held.

The `( i -- addr )` comments are Forth's traditional stack-effect comment convention. GameForth treats them as block comments: the text between `(` and `)` is skipped. Watch the spaces — `( i -- addr )` needs the paren to start its own token, which it does here because of the space in front.

## How `include` composes the demo

Loading `main.fth` does this, in order:

1. `main.fth` sees `include "lib/starfield.fth"`.
2. `starfield.fth` sees `include "random.fth"` → `lib/random.fth` defines `random-between`.
3. Back in `starfield.fth`: three arrays and `star-count` are carved out of the arena, and the `star-*`, `init-stars`, `update-stars`, `draw-stars` words are defined.
4. Back in `main.fth`: the `player-*` words and `init`/`update`/`draw` are defined.
5. Top level ends; `init` runs once; the frame loop starts.

A file included twice would run twice — declarations (`variable`, `constant`, `value`) tolerate this, but plain definitions would simply be replaced, and side-effecting top-level statements would run twice. Includes guard against cycles instead of deduplication, so structure your includes as a tree.

## Fragments

Small, self-contained chunks to adapt.

### Counting loop that prints

```forth
10 0 do i . loop
```

### Sine-wave row of dots

```forth
: wave
  30 0 do
    i 10 / fixed sin 30 * 100 +
    i 20 * swap 4 4 rl-yellow draw-circle
  loop
;

: draw wave ;
```

`i 10 / fixed` converts the loop index into fixed radians; `sin` gives `-1000..1000`; `30 *` is ±30 px; `100 +` centers the wave around `y = 100`. Then `i 20 * swap` lines up `( y x )` into `( x y )` before the circle draw.

### Two-key paddle control

```forth
variable paddle-x

: init 380 paddle-x ! ;

: update
  key-left  key-down? if paddle-x @ 8 - paddle-x ! then
  key-right key-down? if paddle-x @ 8 + paddle-x ! then
  paddle-x @ 0 < if 0 paddle-x ! then
  paddle-x @ 720 > if 720 paddle-x ! then
;

: draw
  paddle-x @ 550 60 16 rl-white draw-rectangle
;
```

### Score with `value` and `to`

```forth
0 value score

: bump 1 score + 10 mod to score ;

: update
  key-space key-pressed? if bump then
;

: draw
  "score" 20 20 20 rl-white draw-text
  score 90 20 20 rl-white draw-number
;
```

`value` pushes the contents directly, so reading is terse; `to` is the write side.

### Headless smoke test

```sh
cargo run -- --headless --frames 120 examples/test.fth
```

Runs 120 simulated frames at 16 ms each and exits nonzero on the first error. No window appears. Good for checking a program's logic in CI or before opening the real window.
