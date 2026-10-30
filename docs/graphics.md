# Graphics and Animation

GameForth's graphics model is deliberately small: a program queues filled rectangles and circles, and the host application renders those commands after evaluating the program for the current frame.

## Contents

- [Window and frame loop](#window-and-frame-loop)
- [Coordinate system](#coordinate-system)
- [The draw command queue](#the-draw-command-queue)
- [Rectangles](#rectangles)
- [Circles](#circles)
- [Color state](#color-state)
- [Layering and draw order](#layering-and-draw-order)
- [Animation](#animation)
- [Bouncing rectangle](#bouncing-rectangle)
- [Multiple objects](#multiple-objects)
- [Off-screen coordinates](#off-screen-coordinates)
- [Debugging graphics programs](#debugging-graphics-programs)
- [Current graphics limitations](#current-graphics-limitations)

## Window and frame loop

The host application creates:

- an 800x600 window
- title `GameForth`
- a target of 60 frames per second
- a background color of RGB `(24, 26, 36)`

Each frame, the host loop does the following:

1. Clears the previous frame's draw command list.
2. Evaluates the entire Forth source file.
3. Begins drawing and clears the background.
4. Renders each queued draw command in order.
5. Presents the frame and repeats.

If evaluation returns an error, the host prints `Forth error: ...` to stderr and leaves the loop.

## Coordinate system

Coordinates are in pixels and follow raylib's screen coordinate system:

- origin `(0, 0)` is the top-left corner
- positive x goes right
- positive y goes down
- the visible area is `0 <= x < 800` and `0 <= y < 600`

For an 800x600 window:

- `x = 0` is the left edge
- `x = 800` is one pixel beyond the right edge
- `y = 0` is the top edge
- `y = 600` is one pixel beyond the bottom edge

Coordinates are converted from Forth's `i64` values to signed 32-bit integers before being passed to raylib. Keep coordinates within the `i32` range.

## The draw command queue

`draw-rectangle` and `draw-circle` do not draw immediately. They append a command to `Graphics::commands`.

This has several consequences:

- A program can queue many shapes in one frame.
- Nothing appears until the Forth evaluation finishes.
- Commands are rendered in the order they were queued.
- The queue is cleared before the next frame's evaluation.
- A command stores the color that was current when it was queued.

This design is what makes the source act like a frame script: state can persist, but the picture is rebuilt every frame.

## Rectangles

Syntax:

```forth
x y width height draw-rectangle
```

Stack effect:

```text
( x y width height -- )
```

- `x`, `y` — top-left corner
- `width`, `height` — size in pixels
- color — the current color

Example:

```forth
350 250 100 100 rl-red draw-rectangle
```

This queues a red rectangle with its top-left at `(350, 250)` and size `100x100`.

Rectangles are filled. There is no outline-only mode and no border width.

## Circles

Syntax:

```forth
x y radius draw-circle
```

Stack effect:

```forth
( x y radius -- )
```

- `x`, `y` — center
- `radius` — radius in pixels
- color — the current color

Example:

```forth
400 300 50 rl-blue draw-circle
```

This queues a blue circle centered at `(400, 300)` with radius `50`.

Circles are filled. There is no outline-only mode.

## Color state

The interpreter keeps one current drawing color. It starts as white.

Color words set the current color:

```forth
rl-red
rl-green
rl-blue
rl-white
rl-black
rl-yellow
rl-orange
rl-purple
rl-pink
rl-gray
```

Color words have no stack effect and do not draw anything.

When a draw word runs, it copies the current color into the queued command. Later color changes do not modify commands that are already queued.

Example:

```forth
50 50 100 100 rl-red draw-rectangle
200 50 100 100 rl-blue draw-rectangle
50 200 50 rl-yellow draw-circle
```

Result:

- a red rectangle at `(50, 50)`
- a blue rectangle at `(200, 50)`
- a yellow circle centered at `(50, 200)`

### Color persists across frames

The command queue is cleared each frame, but the current color is not. If a program ends a frame with `rl-red`, the next frame starts with red until another color word changes it.

For clarity, set the color immediately before each draw command:

```forth
x @ y @ 100 100 rl-red draw-rectangle
```

## Layering and draw order

Commands are rendered in queue order. Later commands appear on top of earlier ones.

Example:

```forth
100 100 200 200 rl-blue draw-rectangle
150 150 100 100 rl-red draw-rectangle
```

The red rectangle is drawn after the blue rectangle, so it covers the overlapping area.

## Animation

Animation requires state that changes between frames. Use variables for that state.

A typical animation frame:

1. Initialize variables once, using a flag.
2. Update position or other state.
3. Apply bounds checks.
4. Draw using the updated state.

Because the source is re-evaluated every frame, variables persist while draw commands are rebuilt.

### Position update

```forth
variable x
variable dx

dx @ x +!
```

`dx @ x +!` adds the current velocity to the position.

### Bounds check

```forth
x @ 0 < if
  0 x !
  -1 dx @ * dx !
then

x @ 700 > if
  700 x !
  -1 dx @ * dx !
then
```

The first block handles the left edge; the second handles the right edge. `-1 dx @ * dx !` negates the velocity.

For a 100-pixel-wide rectangle on an 800-pixel-wide screen, the valid x range is `0..700`. For a 100-pixel-tall rectangle on a 600-pixel-tall screen, the valid y range is `0..500`.

### Initialization flag

A variable starts at `0`. To initialize position and velocity once, use a separate flag:

```forth
variable x
variable initialized

initialized @ 0= if
  350 x !
  -1 initialized !
then
```

Do not use the position itself as the initialization flag if `0` is a valid position, because the block would run again whenever the position reaches `0`.

## Bouncing rectangle

The repository's `examples/test.fth` is a complete diagonal bounce:

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

How it works:

- `x`, `y` — position of the rectangle's top-left corner
- `dx`, `dy` — velocity in pixels per frame
- `initialized` — a one-time initialization flag

Each frame:

1. On the first frame, set the starting position and velocity, then mark `initialized` as true.
2. Add `dx` to `x` and `dy` to `y`.
3. If the rectangle crosses a boundary, clamp it back to the boundary and reverse the corresponding velocity.
4. Queue a red rectangle at the new position.

The rectangle moves `4` pixels horizontally and `3` pixels vertically per frame. At 60 FPS, that is roughly `240` pixels per second horizontally and `180` pixels per second vertically.

## Multiple objects

Each object can have its own variables:

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

This moves one rectangle right and the other left. Add bounds checks if they should bounce.

## Off-screen coordinates

Raylib accepts coordinates outside the window. Shapes with negative coordinates or coordinates beyond the right/bottom edges are simply clipped or partially visible.

Example:

```forth
-50 250 100 100 rl-red draw-rectangle
```

Only the right half of the rectangle is visible.

GameForth itself does not clip coordinates; clipping is handled by the renderer.

## Debugging graphics programs

### Print state

`.` prints to stdout:

```forth
x @ .
```

Because the program runs every frame, this prints once per frame and can flood the terminal. Guard debug output with a condition, or temporarily remove it.

### Check for stack imbalance

If a program pushes values without consuming them, the stack grows every frame. A common symptom is that shapes start using old values from previous frames.

Keep the stack balanced at the end of each frame.

### Check color state

The current color persists across frames. If a shape has an unexpected color, make sure the intended color word appears before the draw word in the current frame.

### Check draw order

Later commands are drawn on top. If a shape seems to be missing, it may be underneath a later shape.

### Check initialization

If an animation resets every frame, the initialization block is probably running every frame. Use a separate flag variable and set it to a nonzero value after initialization.

## Current graphics limitations

- Only filled rectangles and filled circles are available.
- No lines, triangles, polygons, images, or textures.
- No text rendering from Forth.
- No outline/stroke options.
- No per-shape alpha control; the provided colors are fully opaque.
- No camera, transform, or scaling words.
- No input or timing words.
- No sound.
- Drawing is limited to the 800x600 window.
