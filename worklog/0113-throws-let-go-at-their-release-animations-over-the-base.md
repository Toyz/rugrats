---
number: 113
title: Throws let go at their release; animations over the base; speed
date: 2026-10-15
area: play
files: crates/rugrats/src/actors/animator.rs crates/rugrats/src/actors/player.rs crates/rugrats/src/world/game/things.rs crates/rugrats/src/world/game/mod.rs crates/rugrats/src/world/game/story.rs crates/rugrats/src/levels/box_toss.rs crates/rugrats/src/levels/aisle_seven.rs crates/rugrats/src/levels/training.rs crates/rugrats/tests/play.rs
started: 2026-10-15T23:24:19-06:00
took: 15m
---

# 113. Throws let go at their release; animations over the base; speed

30.2 left the animation events that time the grab and the release. The
port threw on the button press. Reading them out showed three more
things the animator was missing.

**Events.** Each FRM frame's first halfword (the port's
`Frame::unknown_0`) is its event bits. 0x80055d40 ORs them into +0xe0
when the frame's time runs out. In DB03:

- the throw's track 1 has 0x10 on its third frame: the release, which
  0x80060b44 waits for;
- the pick-up's track 1 has 0x820 on its fourth: the grab, 2048;
- 1 marks an animation's last frame.

`Animator::events` collects them each step.

**States over the base.** The throw (logical 18, table 26) has no base
column. 0x800571d8 plays such a state on track 1, and 0x800561b0 lets
tracks 2 and 3 take its columns where it has them, the base's
otherwise. Track 0 goes on with the base state, so the legs keep
standing or walking while the arms throw. When track 1 completes
(0x8005678c), tracks 1-3 go back to the base. The port had reset all
four tracks, leaving the legs unanimated and the action over at once.
`Animator` now keeps a base and an `upper` state.

**Speed.** 0x800572a8's third argument is the body's animation speed
(+0xe4, clamped 2048..16384 by 0x80057214), and a frame's time runs
down by dt times it >> 12. The oracle shows D_800c3f1c = 10 and +0xe4 =
4096 while running. The stand handler asks for the throw at 0x2000; the
cry is 0x800572a8(player, 16, 2560). `start_at` / `Player::act_at` take
it. The player update (0x8005a068) enters every state at 4096, or 2048
for a body of template 8, which is Reptar (shape 9). Reptar's
animations, his fire included, run at half speed:
`Animator::written_speed`. The fire takes about 70 VSyncs, as in the
oracle; the port had it at 38.

**The release.** A throw is now `Game::pending_throw`: the press starts
the animation, and the thing stays in hand until event 16, or the
animation's end. The aim and speed are taken then, as 0x80060b44 does.
`throw_carried` (levels 20-21) goes through it too. A press during a
throw is ignored (+0x188 bit 4), which the box toss lacked.

**The grab.** The pick-up handler (0x800601e4) takes the object
(+0x1f0 = +0x1ec) only at event 2048. The port took it on the press. It
is now `Game::pending_grab`, taken at the event (or the animation's
end), with the grip measured from the hands at that frame.

Measured in the port, Tommy's throw at 0x2000 lets go on its 12th frame
and ends on its 19th (test `a_throw_runs_at_its_speed`). With the arm
forward at the release, level 9's can again comes back into Tommy as in
the oracle.

The tests changed with the slower, correct throw rate:

- level 16's toss now replays the oracle's own run, heading 913 and a
  press every 24 VSyncs. Pressing faster no longer helps: the original
  ignores presses while throwing;
- levels 20-21's policy no longer restarts its throw by pressing again;
- the camera-after-a-cry test waits the longer cry out.

**Still unknown:**
- the root motion of bit-32 states (0x80055904: interpolated root offsets, scaled by body size for others than the player); the port moves bodies itself
- the speed argument of the other 0x800572a8 calls the port makes (all 4096 here but the throw and the cry)
- level 16's pies still roll by the fitted stand-in (34.1), which hits less often than the original's flights
