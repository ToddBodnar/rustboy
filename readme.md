# Rust Boy, a Game Boy emulator in Rust

This emulator is capable of running most Game Boy games to some fidelity and 
implements all official instructions and memory banks.

## Samples

![An sample menu](https://raw.githubusercontent.com/ToddBodnar/rustboy/master/screenshots/sample/gary_name.bmp)
![An animation of chatting in pokemon](https://raw.githubusercontent.com/ToddBodnar/rustboy/master/screenshots/sample/gary_talk.gif)
![A Tetris screenshot](https://raw.githubusercontent.com/ToddBodnar/rustboy/master/screenshots/sample/tetris.bmp)
![Passing cpu_instrs test](https://raw.githubusercontent.com/ToddBodnar/rustboy/master/screenshots/sample/cpu_pass.bmp)

## Building

Run `cleanAndMake.sh`. You will need a recent version of Rust and CMake 
installed.

## Running

The emulator can be run through a simple command `cargo run your_rom_here.gb`.

| Gameboy Button | Keyboard Button   |
|----------------|-------------------|
| A              | `F`               |
| B              | `D`               |
| Start          | `R`               |
| Select         | `E`               |
| Up             | `Up`              |
| Down           | `Down`            |
| Left           | `Left`            |
| Right          | `Right`           |

Additionally, screen shots can be created by pressing `space`.

If supported by the game, a `.sav` file will be made in the same directory as
the rom.

## Testing

Unit tests are available in the individual `.rs` files and can be run simply 
with `cargo test`. Additionally, the `integrationTests.sh` script will run 
through each of Blagg's Game Boy test roms and compare the output screen to
a known good result. 