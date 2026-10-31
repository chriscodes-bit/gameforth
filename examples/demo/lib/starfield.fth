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
