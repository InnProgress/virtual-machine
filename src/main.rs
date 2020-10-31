use std::{
    env, fs,
    io::{Read, Write},
};

const INC: u8 = 0x1;
const DEC: u8 = 0x2;
const MOV: u8 = 0x3;
const MOVC: u8 = 0x4;
const LSL: u8 = 0x5;
const LSR: u8 = 0x6;
const JMP: u8 = 0x7;
const JZ: u8 = 0x8;
const JNZ: u8 = 0x9;
const JFE: u8 = 0xA;
const RET: u8 = 0xB;
const ADD: u8 = 0xC;
const SUB: u8 = 0xD;
const XOR: u8 = 0xE;
const OR: u8 = 0xF;
const IN: u8 = 0x10;
const OUT: u8 = 0x11;

const OPERATIONS_WITH_REGISTERS: [u8; 10] = [INC, DEC, MOV, MOVC, LSL, LSR, ADD, SUB, XOR, OR];

struct VirtualMachine {
    registers: [u8; 16],
    memory: [u8; 256],
    flag: u8,
    r#in: u8,
    input: Vec<u8>,
}

impl VirtualMachine {
    pub fn new(memory: [u8; 256], input: Vec<u8>) -> VirtualMachine {
        VirtualMachine {
            registers: [0; 16],
            memory,
            flag: 0,
            r#in: 0,
            input,
        }
    }
    pub fn run(&mut self, output_filename: &str) {
        let mut position_in_memory: usize = 0;
        let mut input = self.input.iter().peekable();

        let mut file = fs::File::create(output_filename).unwrap();

        loop {
            let next_instruction = match self.memory.get(position_in_memory) {
                Some(v) => *v,
                None => {
                    println!("There are no more instructions");
                    break;
                }
            };

            if next_instruction == 0 {
                println!("Undefined instruction");
                break;
            }

            let parameter = match self.memory.get(position_in_memory + 1) {
                Some(v) => *v,
                None => {
                    println!("No parameter provided for instruction");
                    break;
                }
            };

            position_in_memory += 2;

            let rx = (parameter & 0xf) as usize;
            let ry = (parameter >> 4 & 0xf) as usize;

            match next_instruction {
                INC => self.registers[rx] += 1,
                DEC => self.registers[rx] -= 1,
                MOV => self.registers[rx] = self.registers[ry],
                MOVC => self.registers[0] = parameter,
                LSL => self.registers[rx] = self.registers[rx] << 1,
                LSR => self.registers[rx] = self.registers[rx] >> 1,
                JMP => self.jump(&mut position_in_memory, parameter),
                JZ => {
                    if self.flag == 1 {
                        self.jump(&mut position_in_memory, parameter);
                    }
                }
                JNZ => {
                    if self.flag == 0 {
                        self.jump(&mut position_in_memory, parameter);
                    }
                }
                JFE => {
                    if self.r#in == 1 {
                        self.jump(&mut position_in_memory, parameter);
                    }
                }
                RET => {
                    println!("Virtual machine finished working");
                    break;
                }
                ADD => self.registers[rx] += self.registers[ry],
                SUB => self.registers[rx] -= self.registers[ry],
                XOR => self.registers[rx] ^= self.registers[ry],
                OR => self.registers[rx] |= self.registers[ry],
                IN => match input.next() {
                    Some(v) => {
                        self.registers[parameter as usize] = *v;
                    }
                    None => self.r#in = 1,
                },
                OUT => {
                    file.write(&[self.registers[rx]]).unwrap();
                }
                _ => (),
            }

            if OPERATIONS_WITH_REGISTERS.contains(&next_instruction) {
                let register_index = if next_instruction == MOVC { 0 } else { rx };
                if self.registers[register_index] == 0 {
                    self.flag = 1;
                } else {
                    self.flag = 0;
                }
            } else {
                self.flag = 0;
            }
        }
    }
    fn jump(&self, position_in_memory: &mut usize, parameter: u8) {
        *position_in_memory -= 2;
        if parameter > 127 {
            *position_in_memory -= (parameter as i16 - 256).abs() as usize;
        } else {
            *position_in_memory += parameter as usize;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let input = fs::read(&args[1]).unwrap();

    let mut file = fs::File::open(&args[0]).unwrap();
    let mut buffer: [u8; 256] = [0; 256];
    file.read(&mut buffer).unwrap();

    let mut virtual_machine = VirtualMachine::new(buffer, input);
    virtual_machine.run(&args[2]);
}
