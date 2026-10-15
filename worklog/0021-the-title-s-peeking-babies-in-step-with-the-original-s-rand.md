---
number: 21
title: The title's peeking babies, in step with the original's rand
date: 2026-10-14
area: ui, decomp, test
files: crates/rugrats/src/front.rs, crates/rugrats/src/rand.rs, crates/rugrats/src/main.rs, crates/rugrats/tests/title.rs, crates/rugrats/tests/gte.rs, crates/rugrats/examples/babies_sim.rs, crates/rugrats/examples/stage_shot.rs, crates/rugrats-hle/examples/title_babies.rs, crates/rugrats-hle/examples/rand_calls.rs, docs/engine/front-end.md
resolves: 18.1, 19.1, 20.3
started: 2026-10-14T23:34:29-06:00
took: 3m
---

# 21. The title's peeking babies, in step with the original's rand

The two babies on the title are character objects. Each holds one of DB00's
hierarchies 1-5, sits at rest (their parts' quaternions are zero, which the
parts' matrix function turns into the identity), and is turned by its base
angles. They peek in from a random corner and back out again, or sit out
for a while. The state machine (0x80043098) and the easing (0x80042f34) are
ported as `front::Baby`, and the game's rand is now one shared `rand::Rand`
that the babies and main's chooser draw on.

To line up with the original, rand's state at the title had to match. The
first port run started one step behind. A hook on rand in the oracle
(`rand_calls`) found one call before the title's first frame, from an
object the front end makes (0x80048b78), and the title's setup now draws
that once. The oracle's per-frame dump of both babies (`title_babies`) and
the port's (`babies_sim`) then agree for 860 frames, rand's state included,
covering states 0-3 and 7. `tests/title.rs` checks 150 frames against the
oracle (about 20 s). A frame with Tommy and Chuckie peeking looks the same in
both, backdrop included.

Also: 0x80020d10 (a jointed object's base) is exactly the transpose of
0x80020ee0's matrix, product for product. A test shows a zero quaternion
gives the identity through 0x8001fc48.

**Still unknown:** the leaning states (4-6) and what sets them; the giggle's sound
