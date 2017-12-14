use std::env;
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;

const IMAX: u16 = 32767;

fn read_num(file: &mut File) -> Result<u16, Error> {
    let mut bytes = [0; 2];
    file.read_exact(&mut bytes)?;
    return Ok((bytes[0] as u16 + bytes[1] as u16 * 256) as u16);
}

struct VM {
    regs: [u16; 8],
    mem: [u16; (IMAX + 1) as usize],
    curr: usize,
}

impl VM {

    fn read_val(&self, x: u16) -> u16 {
        if x > IMAX {
            self.regs[(x - IMAX - 1) as usize]
        } else {
            x
        }
    }

    fn write_val(&mut self, x: u16, val: u16) {

    }

    fn load_bin(&mut self, bin_file: &str) -> Result<usize, Error> {
        let mut file = File::open(bin_file)?;
        let mut curr = 0;
        while let Ok(num) = read_num(&mut file) {
            self.mem[curr] = num;
            println!("{}: {}", curr, num);
            curr += 1;
        }
        Ok(curr)
    }

    fn run(&mut self) {
        println!("Running...");
        self.curr = 0;
        loop {
            match self.mem[self.curr] {
                0 => return,
                6 => self.jmp(),
                19 => self.out(),
                21 => self.noop(),
                x => {
                    println!("{} not implemented!", x);
                    break;
                }
            }
        }
    }

    fn jmp(&mut self) {
        let a = self.mem[self.curr + 1];
        println!("jumping to {}", a);
        self.curr = a as usize;
    }

    fn out(&mut self) {
        let a = self.mem[self.curr + 1];
        print!("{}", a as u8 as char);
        self.curr += 2;
    }

    fn noop(&mut self) {
        self.curr += 1;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Include the binary filename as the first arg.");
        return;
    }

    let mut vm = VM {
        regs: [0; 8],
        mem: [0; (IMAX + 1) as usize],
        curr: 0
    };
    let bin_file = &args[1];

    println!("Loading program...");
    match vm.load_bin(bin_file) {
        Ok(size) => {
            println!("Loaded {} values.", size);
            vm.run();
        },
        Err(e) => println!("Error loading program: {:?}", e)
    }
}
