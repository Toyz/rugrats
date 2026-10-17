---
number: 186
title: The put-down lets go only at its event
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/tests/hle.rs, crates/rugrats/examples/throw_check.rs
started: 2026-10-17T04:13:31-06:00
took: 7m
---

# 186. The put-down lets go only at its event

The put-down lets go only at its event. The put-down state's handler (0x80060910) runs every frame of state 8 and calls the put-down proper (0x80062ac4) only when +0xe0 has bit 2048, the event of the frame that ended. If the state runs out without it, the state goes back to standing with the thing still in hand.

At level 6's shelf, Chuckie's put-down plays out without the event in both the original and the port, whose animator now matches. Hooks on 0x80060910, 0x80062ac4, 0x80061f88, 0x8008864c, 0x80060810 and 0x8005fdb4 show the handler running 25 times and none of the others. The port used to let go when the animation ended and slid the box away. It now keeps it, and the box stays on the original's place to the end of `level_6_chuckie_box_as_the_original`, which now asserts it is still in hand. The throw keeps its fallback (let go at the animation's end); its own handler (0x80060b44) lets go at event 16 or with D_800c56a8 bit 64.

`examples/throw_check.rs` can hook any list of functions (HOOKFNS=0x...,0x...) and print their first two arguments.

**Still unknown:**
- Why the event does not come at that shelf. It is the animation's own frame (both sides agree), but where the put-down would go was not looked at.
- The throw's end without event 16: whether the original's throw then stays pending.
