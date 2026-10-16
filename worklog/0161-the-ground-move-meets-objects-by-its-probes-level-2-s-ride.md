---
number: 161
title: The ground move meets objects by its probes; level 2's ride is Spike's
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/doors.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T11:57:25-06:00
took: 1h 20m
---

# 161. The ground move meets objects by its probes; level 2's ride is Spike's

Worklog 159's open item, the port stepping Tommy onto level 3's spinning
toy, led to how the original moves the player against objects at all.

**The ground move's probes (0x80063d3c).** A hook trace on the oracle
(Tommy running east along level 3's z 4290): 0x80065210 is not called in
play (only at load); the move calls the segment test 0x80067284 four
times a frame from 0x80063d3c, called by 0x80064a34 three times: probes
from the body's middle, (0, 0, d), (-w, 0, d) and (w, 0, d) turned by the
facing, d = the body box's depth (header +8/+0x10, the larger) + speed
+0x184 x D_800c3f1c (10, peeked), w its half-width. Each object of the
nearby list (template flag 8 aside), each model box: the probe at the
box's own middle height (its unturned (lo y + hi y) / 2 + the object's
y), kept within the body's (pos + body lo y .. hi y), tested against the
turned box (0x8006a714 -> 0x80069c40); a COL wall nearer cancels the hit
(0x80063c3c). The level 3 toy's middle is y 504 (its box lies along x,
turned), above the band its box spans (692..968): the probes pass over
it, and the oracle's Tommy runs past at every phase.

The port's player step now tests these probes against objects on the
ground (`Player::probes`, `probe_height`, `Box3::segment_hit`); a hit
sends the body back across, as before. The oracle, the shut hub back
door from z 22000: run, stopped at 23020 (250 + 170 short of the panel);
walking, 23120; both exact in the port (`hub_back_door_opens_to_the_yard`).
An object is floor on the ground only with its top not above the body's
origin (the floor test runs down from it), so the toy is not stood on
(`tommy_runs_past_level_3s_tipped_toy_as_the_original`). In the air the
old test stands. A door is touched for a use by these probes along the
facing too (a run's reach: the back door opens from 23020).

**Level 2's ride is Spike's.** The probes met level 2's hurdle (template
1187, a bar 950..1283 over the yard floor) for Tommy on Spike, his body
box -1230..700 high: on the oracle he is thrown back from it too, and the
player character is 6 after cine 3 (0x800c5680, 1 before). 0x80060f1c
gives character 6 a running jump of 1400 up, 50 a unit of speed along
(level 11's 1700, 20): the oracle's ride, run then Cross, peaks with the
feet 2020 over the floor, clearing the bar; its run is 200 a frame
(Spike's gait). `chase` now sets character 6, gait (20, 10) and that
jump; the court gives Tommy his back.

The walk: the plan's steps keep the middle probe clear of things
(`Ground::probes_clear`), and a jump is tried where it is not within 1024;
a leap ends on landing (Cross held on, the body jumped again: level 18's
bunny hops to the map's edge), takes off within 85 of its point (a
shelf's edge), and goes on over a thing that holds the run up; a walk
gets 80 plans and clears its bans once when every way is banned; a last
stride counts within 512 (a thing holds the body its probes' reach off);
level 10's babies are gone up to within 850 (the tag's 900), level 11's
bones from about the run-up (`near`) with the turn within 64.

**Still unknown:**
- The riding body box: the oracle's player hierarchy then is 0x17 with
  depth 1400; the port's Spike hierarchy has 1000, and the oracle's ride
  is thrown back about 1200 short of the hurdle's bar, as 1000 + 170
  gives.
- What throws the ride back from the hurdle (state 9's hurt?): the port
  stops it.
- The put-down's hand path: the port's animation 14 does not bring the
  box down to the middle in front, nor give event 2048 (it lets go at the
  end); the oracle's comes down to 630 ahead, y -1211.
