# CHIP-8 Interpreter

This is a chip-8 interpreter build with Rust. The repo also includes a couple of roms you can try, feel free to find and use more.

<img width="1275" alt="Screenshot 2021-11-15 at 18 15 19" src="https://user-images.githubusercontent.com/54840294/141833345-46df9062-4ea1-4b69-a91d-293479d5fc4c.png">

## Requirements
You need to install sdl2 on your system along with Rust.

**Mac**
`brew install sdl2`

**Linux**
`apt-get install libsdl2-dev`

**Windows**
Download from: https://www.libsdl.org/download-2.0.php

## Usage

Clone the repo then run the project while pointing to a rom.

E.g. `cargo run roms/IBM\ Logo.ch8`

## Resources

[CHIP-8 Technical Reference](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference#storage-in-memory)

[CHIP‐8 Instruction Set](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set)

[Cowgod's Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.1)

[Starrhorne CHIP-8 rust](https://github.com/starrhorne/chip8-rust)
