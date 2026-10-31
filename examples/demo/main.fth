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

  key-left key-down? if player-x @ 5 - player-x ! then
  key-right key-down? if player-x @ 5 + player-x ! then
  key-up key-down? if player-y @ 5 - player-y ! then
  key-down key-down? if player-y @ 5 + player-y ! then

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
