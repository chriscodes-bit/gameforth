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
