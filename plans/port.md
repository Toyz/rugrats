# The port

Phases, in order. Each ends with worklog entries, docs pages and tests.

1. **Map the disc and the code.** The file table, every format on the disc,
   the PsyQ libraries identified so the game's own code stands out, the main
   loop and the database (level) lifecycle.
2. **A reference.** The original running whole in an HLE harness on
   `rrt-emu` (GPU, CD, SPU, pad supplied by the host), able to boot to the
   title, save state and peek memory, so every port can be checked.
3. **The platform.** `rugrats` on retro_rt's loop: window, pad, the boot
   logos (`.STR` movies, `.BS` stills), the PlayStation GPU's primitives
   drawn with wgpu.
4. **Front end.** Menus, fonts, the memory card (a real save file in a
   folder, card always in), options, the activities menu.
5. **The world.** Databases: rooms, models, animation, collision; the
   Pickles' house hub; the camera.
6. **Play.** The babies' controls, objects, NPCs, events and cinematics,
   music (XA, SEP/VAB), every episode and activity; the cut content restored
   where it can be.
