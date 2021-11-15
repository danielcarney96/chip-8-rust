use rand;
use rand::Rng;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct OutputState<'a> {
    pub video_ram: &'a [[u8; 64]; 32],
    pub video_ram_changed: bool,
}

pub struct Chip8 {
    video_ram: [[u8; 64]; 32],
    video_ram_changed: bool,
    keypad: [bool; 16],
    keypad_waiting: bool,
    keypad_register: usize,
    memory: [u8; 4096],
    program_counter: usize,
    index_register: usize,
    stack: [usize; 16],
    registers: [u8; 16],
    stack_pointer: u8,
    delay_timer: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        return Chip8 {
            video_ram: [[0; 64]; 32],
            video_ram_changed: false,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
            memory: [0; 4096],
            program_counter: 0x200,
            index_register: 0,
            stack: [0; 16],
            registers: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
        };
    }

    pub fn load_rom(&mut self, contents: Vec<u8>) {
        self.memory[0x200..0x200 + contents.len()].copy_from_slice(&contents[..]);
    }

    pub fn load_font(&mut self) {
        self.memory[0x050..0x050 + FONT.len()].copy_from_slice(&FONT[..]);
    }

    pub fn tick(&mut self, keypad: [bool; 16]) -> OutputState {
        self.keypad = keypad;
        self.video_ram_changed = false;

        if self.keypad_waiting {
            for i in 0..keypad.len() {
                if keypad[i] {
                    self.keypad_waiting = false;
                    self.registers[self.keypad_register] = i as u8;
                    break;
                }
            }
        } else {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            let opcode = self.get_opcode();
            self.handle_opcode(opcode);
        }

        OutputState {
            video_ram: &self.video_ram,
            video_ram_changed: self.video_ram_changed,
        }
    }

    /*
     * An opcode is 2 bytes long, so we grab two bytes from memory starting
     * at the `program_counter` value.
     * To combine the bytes into a single opcode we do the following:
     *   - shift the first value 8 bits to the left (creating 8 0's to the right)
     *   - merge the second value using bitwise OR
     *
     * Example:
     *   - 1 = 11111111
     *   - 1 shifted 8 bits = 1111111100000000
     *   - 2 = 10101010
     *   - value after merge with OR = 1111111110101010
     */
    fn get_opcode(&mut self) -> u16 {
        return (self.memory[self.program_counter as usize] as u16) << 8
            | (self.memory[(self.program_counter + 1) as usize] as u16);
    }

    fn handle_opcode(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8 as usize;
        let y = (opcode & 0x00F0) >> 4 as usize;
        let n = (opcode & 0x000F) as usize;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = (opcode & 0x0FFF) as usize;

        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        match nibbles {
            // 00E0 - CLS
            (0x00, 0x00, 0x0e, 0x00) => {
                for y in 0..32 {
                    for x in 0..64 {
                        self.video_ram[y as usize][x as usize] = 0;
                    }
                }
                self.video_ram_changed = true;
                self.program_counter += 2;
            }
            // 00EE - RET
            (0x00, 0x00, 0x0e, 0x0e) => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize];
            }
            // 1nnn - JP addr
            (0x01, _, _, _) => {
                self.program_counter = nnn;
            }
            // 2nnn - CALL addr
            (0x02, _, _, _) => {
                self.stack[self.stack_pointer as usize] = self.program_counter + 2;
                self.stack_pointer += 1;
                self.program_counter = nnn;
            }
            // 3xkk - SE Vx, byte
            (0x03, _, _, _) => {
                if self.registers[x as usize] == nn {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            // 4xkk - SNE Vx, byte
            (0x04, _, _, _) => {
                if self.registers[x as usize] != nn {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            // 5xy0 - SE Vx, Vy
            (0x05, _, _, 0x00) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            // 6xkk - LD Vx, byte
            (0x06, _, _, _) => {
                self.registers[x as usize] = nn;
                self.program_counter += 2;
            }
            // 7xkk - ADD Vx, byte
            (0x07, _, _, _) => {
                let vx = self.registers[x as usize] as u16;
                let val = nn as u16;
                let result = vx + val;
                self.registers[x as usize] = result as u8;
                self.program_counter += 2;
            }
            // 8xy0 - LD Vx, Vy
            (0x08, _, _, 0x00) => {
                self.registers[x as usize] = self.registers[y as usize];
                self.program_counter += 2;
            }
            // 8xy1 - OR Vx, Vy
            (0x08, _, _, 0x01) => {
                self.registers[x as usize] |= self.registers[y as usize];
                self.program_counter += 2;
            }
            // 8xy2 - AND Vx, Vy
            (0x08, _, _, 0x02) => {
                self.registers[x as usize] &= self.registers[y as usize];
                self.program_counter += 2;
            }
            // 8xy3 - XOR Vx, Vy
            (0x08, _, _, 0x03) => {
                self.registers[x as usize] ^= self.registers[y as usize];
                self.program_counter += 2;
            }
            // 8xy4 - ADD Vx, Vy
            (0x08, _, _, 0x04) => {
                let vx = self.registers[x as usize] as u16;
                let vy = self.registers[y as usize] as u16;
                let result = vx + vy;
                self.registers[x as usize] = result as u8;
                self.registers[0x0F as usize] = if result > 0xFF { 1 } else { 0 };
                self.program_counter += 2;
            }
            // 8xy5 - SUB Vx, Vy
            (0x08, _, _, 0x05) => {
                self.registers[0x0F as usize] =
                    if self.registers[x as usize] > self.registers[y as usize] {
                        1
                    } else {
                        0
                    };
                self.registers[x as usize] =
                    self.registers[x as usize].wrapping_sub(self.registers[y as usize]);
                self.program_counter += 2;
            }
            // 8xy6 - SHR Vx {, Vy}
            (0x08, _, _, 0x06) => {
                self.registers[0xF as usize] = self.registers[x as usize] & 0x1;
                self.registers[x as usize] >>= 1;
                self.program_counter += 2;
            }
            // 8xy7 - SUBN Vx, Vy
            (0x08, _, _, 0x07) => {
                self.registers[0x0F as usize] =
                    if self.registers[y as usize] > self.registers[x as usize] {
                        1
                    } else {
                        0
                    };
                self.registers[x as usize] =
                    self.registers[y as usize].wrapping_sub(self.registers[x as usize]);
                self.program_counter += 2;
            }
            // 8xyE - SHL Vx {, Vy}
            (0x08, _, _, 0x0e) => {
                self.registers[0xF as usize] = (self.registers[x as usize] & 0x80) >> 7;
                self.registers[x as usize] <<= 1;
                self.program_counter += 2;
            }
            // 9xy0 - SNE Vx, Vy
            (0x09, _, _, 0x00) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            // Annn - LD I, addr
            (0x0a, _, _, _) => {
                self.index_register = nnn;
                self.program_counter += 2;
            }
            // Bnnn - JP V0, addr
            (0x0b, _, _, _) => {
                self.program_counter = self.registers[0] as usize + nnn;
            }
            // Cxkk - RND Vx, byte
            (0x0c, _, _, _) => {
                let mut rng = rand::thread_rng();
                self.registers[x as usize] = rng.gen::<u8>() & nn;
                self.program_counter += 2;
            }
            // Dxyn - DRW Vx, Vy, nibble
            (0x0d, _, _, _) => {
                self.registers[0x0F] = 0;
                for byte in 0..n {
                    let y = (self.registers[y as usize] as usize + byte) % 32;
                    for bit in 0..8 {
                        let x = (self.registers[x as usize] as usize + bit) % 64;
                        let color =
                            (self.memory[self.index_register + byte as usize] >> (7 - bit)) & 1;
                        self.registers[0x0F] |= color & self.video_ram[y][x];
                        self.video_ram[y as usize][x as usize] ^= color;
                    }
                }
                self.video_ram_changed = true;
                self.program_counter += 2;
            }
            // Ex9E - SKP Vx
            (0x0e, _, 0x09, 0x0e) => {
                if self.keypad[self.registers[x as usize] as usize] {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            // ExA1 - SKNP Vx
            (0x0e, _, 0x0a, 0x01) => {
                if !self.keypad[self.registers[x as usize] as usize] {
                    self.program_counter += 2;
                }
                self.program_counter += 2;
            }
            // Fx07 - LD Vx, DT
            (0x0f, _, 0x00, 0x07) => {
                self.registers[x as usize] = self.delay_timer;
                self.program_counter += 2;
            }
            // Fx0A - LD Vx, K
            (0x0f, _, 0x00, 0x0a) => {
                self.keypad_waiting = true;
                self.keypad_register = x as usize;
                self.program_counter += 2;
            }
            // Fx15 - LD DT, Vx
            (0x0f, _, 0x01, 0x05) => {
                self.delay_timer = self.registers[x as usize];
                self.program_counter += 2;
            }
            // Fx18 - LD ST, Vx
            (0x0f, _, 0x01, 0x08) => {
                self.program_counter += 2;
            }
            // Fx1E - ADD I, Vx
            (0x0f, _, 0x01, 0x0e) => {
                self.index_register += self.registers[x as usize] as usize;
                self.registers[0x0F as usize] = if self.index_register > 0x0F00 { 1 } else { 0 };
                self.program_counter += 2;
            }
            // Fx29 - LD F, Vx
            (0x0f, _, 0x02, 0x09) => {
                self.index_register = ((self.registers[x as usize]) * 5) as usize;
                self.program_counter += 2;
            }
            // Fx33 - LD B, Vx
            (0x0f, _, 0x03, 0x03) => {
                self.memory[self.index_register as usize] = self.registers[x as usize] / 100;
                self.memory[self.index_register + 1 as usize] =
                    (self.registers[x as usize] % 100) / 10;
                self.memory[self.index_register + 2 as usize] = self.registers[x as usize] % 10;
                self.program_counter += 2;
            }
            // Fx55 - LD [I], Vx
            (0x0f, _, 0x05, 0x05) => {
                for i in 0..x + 1 {
                    self.memory[(self.index_register + i as usize) as usize] =
                        self.registers[i as usize];
                }
                self.program_counter += 2;
            }
            // Fx65 - LD Vx, [I]
            (0x0f, _, 0x06, 0x05) => {
                for i in 0..x + 1 {
                    self.registers[i as usize] =
                        self.memory[(self.index_register + i as usize) as usize];
                }
                self.program_counter += 2;
            }
            _ => self.program_counter += 2,
        }
    }
}
