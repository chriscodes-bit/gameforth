# Graphics and Animation

GameForth's graphics model is deliberately small: a program queues rectangles, circles, lines, and text, and the host application renders those commands after `draw` finishes for the current frame.

## Contents

- [Window and frame loop](#window-and-frame-loop)
- [Coordinate system](#coordinate-system)
- [The draw command queue](#the-draw-command-queue)
- [Rectangles](#rectangles)
- [Circles and lines](#circles-and-lines)
- [Text](#text)
- [Color state](#color-state)
- [Layering and draw order](#layering-and-draw-order)
- [Animation](#animation)
- [Bouncing rectangle](#bouncing-rectangle)
- [Text-driven effects](#text-driven-effects)
- [Off-screen coordinates](#off-screen-coordinates)
- [Headless mode](#headless-mode)
- [Current graphics limitations](#current-graphics-limitations)

## Window and frame loop

Windowed mode creates:

- an 800x600 window
- title `GameForth`
- a target of 60 frames per second
- a background color of RGB `(24, 26, 36)`

Each frame, the host loop does the following:

1. Takes a snapshot of keyboard and mouse state.
2. Sets `dt`, `time`, and `frame`.
3. Clears the previous frame's draw command list.
4. Runs `update`.
5. Runs `draw`.
6. Begins drawing, clears the background, and renders each queued command in order.
7. Rewinds: hot reload check, then repeat.

If a frame raises a Forth error, the host prints it to stderr and leaves the loop.

Loading works slightly differently: the top-level code runs once, then `init` runs once, before the first frame. All of your `:` definitions, `variable` declarations, and `include` calls belong there.

## Coordinate system

Coordinates are in pixels and follow raylib's screen coordinate system:

- origin `(0, 0)` is the top-left corner
- positive x goes right
- positive y goes down
- the visible area is `0 <= x < 800` and `0 <= y < 600`

For an 800x600 window:

- `x = 0` is the left edge
- `x = 799` is the last visible pixel column
- `y = 599` is the last visible row

Rectangles take their top-left corner; circles and text do not (circles take a center, text takes a top-left). Coordinates are converted from `i64` to `i32` on the way to raylib; stay inside the `i32` range.

## The draw command queue

Drawing words do not paint immediately. Each one appends a command — including the color that was current when it was queued — to a list that is cleared at the start of the next frame and rendered at the end of this one. This means:

- multiple draw words in one frame produce multiple shapes
- later commands are drawn on top of earlier commands
- everything drawn in `draw` appears exactly one frame's worth of state later

Shapes drawn only at the top level disappear after the first frame, because the queue is rebuilt from scratch every frame by `update`/`draw`, which means everything that needs to stay on screen must be queued again in `draw`.

## Rectangles

```forth
\ x y w h color
100 50 40 40 rl-red draw-rectangle
100 120 40 20 rl-maroon draw-rectangle-lines
```

`draw-rectangle` fills; `draw-rectangle-lines` outlines. Width and height are exact pixel counts, unlike the circle radius.

## Circles and lines

```forth
\ x y r
200 300 24 rl-skyblue draw-circle
200 300 31 rl-darkblue draw-circle-lines

\ x1 y1 x2 y2
400 100 700 240 rl-green draw-line
```

Circles take a center point and a radius. Adding a filled circle and a slightly larger outlined circle is a cheap way to get an outlined disc.

## Text

```forth
\ handle x y size   (color comes from the current color word)
"Arrows to move" 20 20 20 rl-white draw-text

\ n x y size
score @ 20 50 20 rl-white draw-number
```

`"text"` pushes a string-pool handle. `draw-number` takes the number itself, not a handle. The default raylib font is used at the requested size; there is no font selection.

`text-width` estimates a string's pixel width without drawing it — `chars * size / 2` — which is close enough for the default font to, say, center a label roughly or align a HUD column. Treat it as an estimate.

## Color state

A color word changes the interpreter's current color. The order matters:

```forth
40 40 60 60 rl-red draw-rectangle
60 60 60 60 rl-blue draw-rectangle   \ blue now, until another color word
```

Every queued command keeps the color that was active when it was queued, so later color changes do not affect earlier commands.

## Layering and draw order

Commands render in queue order. To draw a background then a HUD, queue the background first:

```forth
: draw
  draw-background
  draw-stars
  draw-player
  draw-hud
;
```

## Animation

State changes belong in `update`; drawing belongs in `draw`. This separation is the whole lifecycle:

```forth
variable pos

: init 10 pos ! ;

: update
  pos @ 2 + 780 mod pos !
;

: draw
  pos @ 280 30 30 rl-yellow draw-circle
;
```

Scale movement with `dt` when frames might jitter:

```forth
variable pos
variable speed

: init
  0 pos !
  120 speed !      \ pixels per second
;

: update
  speed @ dt * 1000 / pos +!
  pos @ 800 mod pos !
;
```

Here `speed` is in pixels per second, `dt` in milliseconds, so `speed * dt / 1000` is the distance to move this frame.

## Bouncing rectangle

The bundled example (`examples/test.fth`) bounces a square on all four walls, using `negate` to flip the velocity at the boundary:

```forth
: update
  dx @ x +!
  dy @ y +!

  x @ 0 < if 0 x ! dx @ negate dx ! then
  x @ 700 > if 700 x ! dx @ negate dx ! then
  y @ 0 < if 0 y ! dy @ negate dy ! then
  y @ 500 > if 500 y ! dy @ negate dy ! then
;
```

See the [word reference](words.md#graphics-words) for the exact stack effects of every drawing word.

## Text-driven effects

A few things text is good at besides HUD labels:

- Frame counters: `frame 20 20 16 rl-gray draw-number`
- Random dice: `100 random 30 30 24 rl-white draw-number`
- Fixed-point oscillation as motion: `time 200 / fixed sin 30 * 100 +` gives a smooth up-and-down y offset between 70 and 130 (the residue-free way to remember it: `sin` returns `-1000..1000`, so multiply by the pixel amplitude you want)

## Off-screen coordinates

Nothing clips coordinates: a rectangle at `x = -50` draws partially off the left edge, and a huge radius draws over everything earlier in the queue. Clipping is your job, usually with boundary checks in `update` like the bounce example.

## Headless mode

`--headless` skips the window entirely but still runs the lifecycle per frame. Draw commands are queued and never rendered. This is useful for validating program logic from the command line:

```sh
cargo run -- --headless --frames 60 examples/test.fth
```

Draw words are safe to call in headless mode; they just do nothing visible.

## Current graphics limitations

- No images, textures, sprites, or render targets exposed to Forth.
- No camera, scissor, rotation, or blending.
- No arbitrary RGB words; colors come from the fixed raylib palette.
- `text-width` is an estimate, not a font measurement.
- Only the default raylib font, no font loading.
- The palette colors are all fully opaque (alpha `255`); there is no per-command transparency.
