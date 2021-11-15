use std::env;
use std::fs;
use std::thread;
use std::time::Duration;
mod chip8;
mod display;
mod input;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("You must specify a file location.");
    }

    let rom = fs::read(&args[1]);

    let filename = match rom {
        Ok(file) => file,
        Err(e) => panic!("File could not be loaded: {}", e),
    };

    let mut chip = chip8::Chip8::new();
    chip.load_rom(filename);
    chip.load_font();

    let mut display = display::Display::new();
    let mut input_driver = input::InputDriver::new(&display.context);

    while let Ok(keypad) = input_driver.poll() {
        let output = chip.tick(keypad);

        if output.video_ram_changed {
            display.draw(output.video_ram);
        }

        thread::sleep(Duration::from_millis(2));
    }
}
