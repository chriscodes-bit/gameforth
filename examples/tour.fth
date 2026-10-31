\ tour.fth — a language tour with no graphics words at all.
\
\ GameForth is framed as a game language, but only `update` and `draw`
\ are game-specific. Everything else is a plain console Forth. This
\ program is all top-level code with zero drawing: it runs once at
\ load, prints a section at a time, and finishes.
\
\ Run it headless (no window, no frame loop needed):
\
\   cargo run -- --headless examples/tour.fth
\
\ It also runs windowed, but the output is the same and the window
\ shows only the background color.

\ ---------------------------------------------------------------------------
\ A number printer that does not add a newline. The builtin `.` prints
\ value + newline, which is awkward mid-list, so this one emits raw
\ digits via `/`, `mod`, and `emit`. It is also a small recursion demo.
\ ---------------------------------------------------------------------------

: emit-digits ( n -- )
  dup 0< if 45 emit negate then        \ leading '-' (char 45)
  dup 9 > if
    dup 10 / recurse                   \ all digits except the last
    10 mod 48 + emit                   \ then the last digit
  else
    48 + emit                          \ one digit, fin
  then
;

\ ---------------------------------------------------------------------------
\ Two classic algorithms, to show off begin/while, tuck/mod, and recurse.
\ ---------------------------------------------------------------------------

: gcd ( a b -- g )
  begin
    dup 0= not
  while
    tuck mod                          \ ( a b -- b a-mod-b )
  repeat
  drop
;

: fib ( n -- f )
  dup 2 < if exit then
  dup 1 - recurse swap 2 - recurse +
;

: fact ( n -- n! )
  1 swap 1+ 1 ?do i * loop
;

\ ---------------------------------------------------------------------------
\ Sieve of Eratosthenes, marking composites in a real memory array.
\ ---------------------------------------------------------------------------

40 constant limit

create composite limit cells allot

: comp-cell ( i -- addr ) cells composite + ;

: mark-multiples ( -- called from inside the sieve's do loop )
  i i * limit < if
    limit i i * do
      -1 i comp-cell !                \ mark this multiple, not the prime
      j +loop                         \ step by the prime (the outer index)
  then
;

: sieve ( -- )
  limit 2 do
    i comp-cell @ 0= if mark-multiples then
  loop
;

: print-primes ( -- )
  "primes under " type limit emit-digits cr
  limit 2 do
    i comp-cell @ 0= if i emit-digits space then
  loop
  cr
;

\ ---------------------------------------------------------------------------
\ Bubble sort over ten random numbers. begin/until loops until a pass
\ makes no swaps; the inner do-loop uses a `swapped` counter with +!.
\ ---------------------------------------------------------------------------

10 constant deck-size

create deck deck-size cells allot
variable swapped

: deck@ ( i -- value ) cells deck + @ ;
: deck! ( value i -- ) cells deck + ! ;

: random-deck ( -- )
  deck-size 0 do 100 random i deck! loop
;

: print-deck ( -- )
  deck-size 0 do i deck@ emit-digits space loop cr
;

: bubble ( -- )
  begin
    0 swapped !
    deck-size 1 - 0 do
      i deck@ i 1 + deck@ 2dup > if
        swap i 1 + deck! i deck!
        1 swapped +!
      else
        2drop
      then
    loop
    swapped @ 0=
  until
;

\ ---------------------------------------------------------------------------
\ Statistics in fixed point. The dataset is written with `,` (compile
\ here, like db); mean and variance run in scale-1000 fixed point and
\ use f/ and f-sqrt.
\ ---------------------------------------------------------------------------

create data 100 , 220 , 315 , 90 , 500 , 1000 ,
6 constant data-len

: mean ( -- mean, fixed )
  0
  data-len 0 do i cells data + @ + loop
  data-len f/
;

: variance ( -- variance, fixed )
  0
  data-len 0 do
    i cells data + @ fixed mean - dup f* +
  loop
  data-len /                          \ plain /: sum of diffs^2 is already fixed-scaled
;

: stddev ( -- sigma, fixed )
  variance f-sqrt
;

: print-data ( -- )
  "dataset: " type
  data-len 0 do i cells data + @ emit-digits space loop
  cr
;

\ ---------------------------------------------------------------------------
\ Miscellaneous one-shot words used by the tour sections.
\ ---------------------------------------------------------------------------

variable found

: first-multiple-of-7 ( -- prints result )
  -1 found !
  50 31 do
    i 7 mod 0= if i found ! leave then
  loop
  found @ emit-digits
;

: only-small ( n -- )
  dup 90 > if
    drop "too big" type cr exit
  then
  emit-digits cr
;

: countdown ( n -- )
  begin
    dup emit-digits space 1- dup 0=
  until
  drop cr
;

\ ---------------------------------------------------------------------------
\ The tour itself: definitions above, output below. Top-level code runs
\ once per load, which is exactly once per program launch here.
\ ---------------------------------------------------------------------------

cr
"==================================================" type cr
" GameForth language tour (console only)" type cr
"==================================================" type cr
cr

"== strings and characters ==" type cr
"hello from GameForth" type cr
"its length: " type "hello from GameForth" string-length .
"s p e l l e d  o u t: " type 104 emit 101 emit 108 emit 108 emit 111 emit cr
cr

"== arithmetic ==" type cr
"17 rem 5  = " type 17 5 mod .
"17 / 5    = " type 17 5 / .
"17 5 /mod -> quotient, then remainder: " type 17 5 /mod . .
"7 * 6     = " type 7 6 * .
"-7 abs    = " type -7 abs .
"3 min 9   = " type 3 9 min .
"3 max 9   = " type 3 9 max .
"99 sqrt   = " type 99 sqrt .
"1 << 10   = " type 1 10 lshift .
"1024 >> 3 = " type 1024 3 rshift .
cr

"== bits ==" type cr
"0xFF and 0x0F = " type 0xFF 0x0F and .
"0xF0  or 0x0F = " type 0xF0 0x0F or .
"5 xor 3       = " type 5 3 xor .
"7 invert      = " type 7 invert .
cr

"== stack words, shown with .s ==" type cr
"rot:  input 10 20 30 -> " type 10 20 30 rot .s drop drop drop cr
"-rot: input 10 20 30 -> " type 10 20 30 -rot .s drop drop drop cr
"tuck: input 10 20 -> " type 10 20 tuck .s drop drop drop cr
"nip:  input 10 20 -> " type 10 20 nip .s drop cr
"pick 1 on 10 20 30 -> " type 10 20 30 1 pick .s drop drop drop drop cr
cr

"== values and variables ==" type cr
7 value hp
"hp: " type hp .
"after hp 2 + to hp = " type hp 2 + to hp hp .
variable clicks
0 clicks !
5 clicks +!
"clicks after 5 clicks +!: " type clicks @ .
cr

"== control flow ==" type cr
"do/loop count to five: " type
5 0 do i emit-digits space loop cr

"48 18 gcd = " type 48 18 gcd .
"fib 10 = " type 10 fib .
"5! = " type 5 fact .
"0! = " type 0 fact .

"countdown from 3: " type 3 countdown

"only-small checks 73 and 105: " type
73 only-small
105 only-small

"evens 20..2 via +loop: " type
0 20 do i emit-digits space -2 +loop cr

"?do skips an empty range, do runs once: " type
7 7 ?do i emit-digits space loop "<" type
7 7 do i emit-digits space loop "<" type cr

"first multiple of 7 after 30 (leave): " type first-multiple-of-7 cr

"small times table (nested do, i and j): " type cr
4 1 do
  4 1 do
    i j * emit-digits space
  loop
  cr
loop

"== memory arrays ==" type cr
"squares of 0..7 in an allot'd array: " type
create squares 8 cells allot
8 0 do i i * squares i cells + ! loop
8 0 do squares i cells + @ emit-digits space loop cr
"next free cell (here) after all that: " type here .
cr

"== fixed point ==" type cr
"1.5 * 2 via f*       = " type 1500 2000 f* .
"sin of ~pi/2 (1571)  = " type 1571 sin .
print-data
"mean (fixed)         = " type mean .
"mean, truncated int  = " type mean int .
"variance (fixed)     = " type variance .
"stddev via f-sqrt    = " type stddev .
cr

"== random ==" type cr
"three d100 rolls: " type
100 random emit-digits space 100 random emit-digits space 100 random emit-digits space cr
"same seed, same sequence, twice: " type cr
1234 random-seed
20 0 do 10 random emit-digits space loop cr
1234 random-seed
20 0 do 10 random emit-digits space loop cr
cr

"== sieve and sort ==" type cr
sieve print-primes

"ten random values: " type
random-deck print-deck
"after bubble sort: " type
bubble print-deck
cr

"== done ==" type cr
"final stack depth (should be 0): " type depth . cr
"and .s agrees: " type .s cr
