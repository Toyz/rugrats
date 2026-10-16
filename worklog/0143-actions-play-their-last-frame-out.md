---
number: 143
title: Actions play their last frame out
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/player.rs
started: 2026-10-16T02:27:17-06:00
took: 4m
---

# 143. Actions play their last frame out

`Animator::playing` said an animation was over once its last frame began.
An action (a climb, a throw, a pick-up) therefore ended a frame or two
early, its last frame cut short. On the oracle Chuckie's climb in level
10 goes on through its last frame (-152, -172, -193) before state 0
comes. The port stopped at -153.

`playing` now holds until the last frame's time has run out. All actions
gain their last frame.

The climb also ends as its animation does: in the frame the next state
begins under it (`Player::finish_climb`, called after the animation's
step). It no longer waits for the next frame's action test. In between,
the body was drawn at its physics height, which a higher box under the
step (the seat's back) had moved; that put it 300 off for a frame. The
climb keeps its top height (`Climb::top`), and the body is put there when
it ends.

Chuckie's climb against the oracle, the end:

- port: -134, -153, -173, then state 0 at -214, the run at -249, -286;
- oracle: -133, -152, -172, -193, then state 0 at -192, the run at -217,
  -250.

The climb's own frames match. The port's state 0 frame is 20 off, from the
run's root y coming in a frame sooner.

All play tests pass, and so does the whole game's run from a new game to
the credits (app::play::tests::the_whole_game_from_new_to_the_credits).

**Still unknown:**

- The oracle holds state 13 for one more frame (-193) than the port.
