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
