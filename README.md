# rugrats

A native PC port of Rugrats: Search for Reptar for the PlayStation (n-Space /
THQ, 1998, `SLUS-00650`), written in Rust from a reverse engineering of the
game, on [retro_rt](https://github.com/Toyz/retro_rt). It plays your own disc.

**Status: starting.** `rugrats` boots on retro_rt and shows the legal
screen, decoded natively and identical to the original's, then stops where
the port ends (the opening cinematic). `rugrats-hle` runs the original as
the oracle the port is checked against.

## Not an emulator

The port will run none of the PlayStation's code. The game is rewritten in
Rust, subsystem by subsystem, and each part is checked against the original
function, run instruction by instruction in retro_rt's R3000A + GTE
interpreter (`rrt-emu`). During play nothing from the PlayStation runs and no
BIOS is needed.

## What is here

| path | what |
| --- | --- |
| `crates/rugrats-psx` | the PS-X EXE format, an R3000A + GTE disassembler, static analysis |
| `crates/rugrats-data` | readers for the game's data: the file table, BS stills, the MDEC's decoding, TIM, VAB |
| `crates/rugrats` | the port: the game's systems in Rust (`lib`), and the program on retro_rt's loop |
| `crates/rugrats-hle` | the oracle: the original run in rrt-emu with the console's I/O modelled; tests only, never the program |
| `crates/rugrats-re` | the reverse engineering command line |
| `docs/` | the [reference](docs/README.md): what is true about the disc, formats and engine |
| `docs/cut/` | cut, missing and unreachable content, one page each |
| `WORKLOG.md` | the [worklog](WORKLOG.md): how each fact was found |
| `plans/` | what comes next |

## Using it

Put the disc where the tools look for it, then extract:

```sh
mkdir -p work/disc && 7z x -owork/disc "original/Rugrats - Search for Reptar.7z"
cargo run -q -p rugrats-re -- disc info      # track, volume, sector kinds
cargo run -q -p rugrats-re -- disc extract   # files -> work/fs
cargo run -q -p rugrats-re -- files          # the game's file table against the ISO
cargo run -q -p rugrats-re -- funcs work/fs/SLUS_006.50
cargo run -p rugrats                          # the port
cargo run -p rugrats-hle -- 600 --every 30    # the original, frames to work/hle
cargo test --workspace                        # ports checked against the original
```

## License

MIT or Apache-2.0, at your option. The game and its data are not included.
