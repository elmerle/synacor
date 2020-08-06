use std::env;
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;
use std::io::stdin;

const IMAX: u16 = 32767;
const MOD: u32 = 32768;

static DEBUG: bool = false;

enum State {
    RUNNING,
    HALT,
    ERROR(String)
}

macro_rules! debug {
    () => {{
        if DEBUG {
            println!("\n");
        }
    }};
    ($msg:expr) => {{
        if DEBUG {
            println!($msg);
        }
    }};
    ($msg:expr, $($arg:expr),*) => {{
        if DEBUG {
            println!($msg, $($arg),*);
        }
    }};
}

fn read_num(file: &mut File) -> Result<u16, Error> {
    let mut bytes = [0; 2];
    file.read_exact(&mut bytes)?; 
    return Ok(bytes[0] as u16 + bytes[1] as u16 * 256);
}

struct VM {
    regs: [u16; 8],
    mem: [u16; (IMAX + 1) as usize],
    curr: usize,
    stack: Vec<u16>,
    state: State,
    stdin_buf: Vec<u8>
}

impl VM {

    fn is_reg(&self, x: u16) -> bool {
        x >= 32768 && x <= 32775
    }

    fn error<S: Into<String>>(&mut self, msg: S) {
        self.state = State::ERROR(msg.into());
    }

    fn halt_impl(&mut self) {
        self.state = State::HALT;
    }

    fn read_val(&self, x: u16) -> u16 {
        if x > IMAX {
            let val = self.regs[(x - IMAX - 1) as usize];
            assert!(val <= IMAX);
            val
        } else {
            assert!(x <= IMAX);
            x
        }
    }

    fn read_mem(&self, x: u16) -> u16 {
        let val = self.mem[x as usize];
        assert!(val <= IMAX);
        val
    }

    fn write_val(&mut self, x: u16, val: u16) {
        if x > IMAX {
            self.regs[(x - IMAX - 1) as usize] = val;
        } else {
            self.mem[x as usize] = val;
        }
    }

    fn load_bin(&mut self, bin_file: &str) -> Result<(), Error> {
        println!("Loading program...");
        let mut file = File::open(bin_file)?;
        let mut curr = 0;
        while let Ok(num) = read_num(&mut file) {
            self.mem[curr] = num;
            debug!("{}: {}", curr, num);
            curr += 1;
        }
        println!("Loaded {} values.", curr);
        Ok(())
    }

    fn load_input(&mut self, input_file: &str) -> Result<(), Error> {
        println!("Reading input...");
        let mut file = File::open(input_file)?;
        file.read_to_end(&mut self.stdin_buf)?;
        // todo only retain non-13 ascii (carriage)
        debug!("{:?}", self.stdin_buf);
        Ok(())
    }

    fn run(&mut self) {
        println!("Running...");
        self.curr = 0;
        loop {
            match &self.state {
                State::HALT => return,
                State::ERROR(message) => {
                    println!("ERROR: {}", message);
                    return;
                },
                State::RUNNING =>
                    match self.mem[self.curr] {
                        0 => self.halt(),
                        1 => self.set(),
                        2 => self.push(),
                        3 => self.pop(),
                        4 => self.eq(),
                        5 => self.gt(),
                        6 => self.jmp(),
                        7 => self.jt(),
                        8 => self.jf(),
                        9 => self.add(),
                        10 => self.mult(),
                        11 => self.mod_(),
                        12 => self.and(),
                        13 => self.or(),
                        14 => self.not(),
                        15 => self.rmem(),
                        16 => self.wmem(),
                        17 => self.call(),
                        18 => self.ret(),
                        19 => self.out(),
                        20 => self.in_(),
                        21 => self.noop(),
                        x => self.error(format!("{} not implemented!", x))
                }
            }
        }
    }

    fn halt(&mut self) {
        debug!("{} 0: halt", self.curr);
        self.halt_impl();
    }

    fn set(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        assert!(self.is_reg(a));
        debug!("{} 1: setting {} to {}", self.curr, a, b);
        self.write_val(a, b);
        self.curr += 3;
    }

    fn push(&mut self) {
        let a = self.read_val(self.mem[self.curr + 1]);
        debug!("{} 2: pushing {}", self.curr, a);
        self.stack.push(a);
        self.curr += 2;
    }

    fn pop(&mut self) {
        let a = self.mem[self.curr + 1];
        assert!(self.is_reg(a));
        debug!("{} 3: popping to {}", self.curr, a);
        match self.stack.pop() {
            Some(val) => {
                self.write_val(a, val);
                self.curr += 2;
            },
            None => self.error("Popping off empty stack!")
        }
    }

    fn eq(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        let c = self.read_val(self.mem[self.curr + 3]);
        assert!(self.is_reg(a));
        if b == c {
            self.write_val(a, 1);
        } else {
            self.write_val(a, 0);
        }
        debug!("{} 4: {} eq {}, setting {}", self.curr, b, c, a);
        self.curr += 4;
    }

    fn gt(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        let c = self.read_val(self.mem[self.curr + 3]);
        assert!(self.is_reg(a));
        if b > c {
            self.write_val(a, 1);
        } else {
            self.write_val(a, 0);
        }
        debug!("{} 5: {} gt {}, setting {}", self.curr, b, c, a);
        self.curr += 4;
    }


    fn jmp(&mut self) {
        let a = self.read_val(self.mem[self.curr + 1]);
        debug!("{} 6: jumping to {}", self.curr, a);
        self.curr = a as usize;
    }

    fn jt(&mut self) {
        let a = self.read_val(self.mem[self.curr + 1]);
        let b = self.read_val(self.mem[self.curr + 2]);
        if a != 0 {
            debug!("{} 7: {} jt to {}", self.curr, a, b);
            self.curr = b as usize;
        } else {
            debug!("{} 7: no jt", self.curr);
            self.curr += 3;
        }
    }

    fn jf(&mut self) {
        let a = self.read_val(self.mem[self.curr + 1]);
        let b = self.read_val(self.mem[self.curr + 2]);
        if a == 0 {
            debug!("{} 8: jf to {}", self.curr, b);
            self.curr = b as usize;
        } else {
            debug!("{} 8: {} no jf", self.curr, a);
            self.curr += 3;
        }
    }

    fn add(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        let c = self.read_val(self.mem[self.curr + 3]);
        assert!(self.is_reg(a));
        debug!("{} 9: add {} and {} to {}", self.curr, b, c, a);
        self.write_val(a, ((b as u32 + c as u32) % MOD) as u16);
        self.curr += 4;
    }

    fn mult(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        let c = self.read_val(self.mem[self.curr + 3]);
        assert!(self.is_reg(a));
        debug!("{} 10: mult {} and {} to {}", self.curr, b, c, a);
        self.write_val(a, ((b as u32 * c as u32) % MOD) as u16);
        self.curr += 4;
    }

    fn mod_(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        let c = self.read_val(self.mem[self.curr + 3]);
        assert!(self.is_reg(a));
        debug!("{} 11: mod {} and {} to {}", self.curr, b, c, a);
        self.write_val(a, b % c);
        self.curr += 4;
    }

    fn and(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        let c = self.read_val(self.mem[self.curr + 3]);
        assert!(self.is_reg(a));
        debug!("{} 12: {} and {} to {}", self.curr, b, c, a);
        self.write_val(a, b & c);
        self.curr += 4;
    }

    fn or(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        let c = self.read_val(self.mem[self.curr + 3]);
        assert!(self.is_reg(a));
        debug!("{} 13: {} or {} to {}", self.curr, b, c, a);
        self.write_val(a, b | c);
        self.curr += 4;
    }

    fn not(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        assert!(self.is_reg(a));
        debug!("{} 14: not {} to {}", self.curr, b, a);
        self.write_val(a, !b & 0x7fff);
        self.curr += 3;
    }

    fn rmem(&mut self) {
        let a = self.mem[self.curr + 1];
        let b = self.read_val(self.mem[self.curr + 2]);
        assert!(self.is_reg(a));
        let val = self.read_mem(b);
        debug!("{} 15: rmem {} at {} to {}", self.curr, val, b, a);
        self.write_val(a, val);
        self.curr += 3;
    }

    fn wmem(&mut self) {
        let a = self.read_val(self.mem[self.curr + 1]);
        let b = self.read_val(self.mem[self.curr + 2]);
        debug!("{} 16: wmem {} to {}", self.curr, b, a);
        self.write_val(a, b);
        self.curr += 3;
    }

    fn call(&mut self) {
        let a = self.read_val(self.mem[self.curr + 1]);
        self.stack.push((self.curr + 2) as u16);
        debug!("{} 17: call {}", self.curr, a);
        self.curr = a as usize;
    }

    fn ret(&mut self) {
        match self.stack.pop() {
            Some(val) => {
                self.curr = val as usize;
            },
            None => self.halt_impl()
        }
    }

    fn in_(&mut self) {
        let a = self.mem[self.curr + 1];
        assert!(self.is_reg(a));
        if self.stdin_buf.len() == 0 {
            let mut input = String::new();
            match stdin().read_line(&mut input) {
                Ok(_) => self.stdin_buf = input.trim_end().to_string().into_bytes(),
                Err(error) => self.error(format!("stdin error: {}",error)),
            };
            // Windows adds \r\n, so strip the trailing whitespace and append a newline ourselves.
            self.stdin_buf.push(b'\n');
        }
        let ch = self.stdin_buf.remove(0);
        self.write_val(a, ch as u16);
        debug!("{} 20: read {} into {}", self.curr, ch, a);
        self.curr += 2;
    }

    fn out(&mut self) {
        let a = self.read_val(self.mem[self.curr + 1]);
        debug!("{} 19: {}", self.curr, a as u8 as char);
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
        curr: 0,
        stack: Vec::new(),
        state: State::RUNNING,
        stdin_buf: Vec::new()
    };

    if let Err(e) = vm.load_bin(&args[1]) {
        println!("Error loading program: {:?}", e);
        return;
    }

    if args.len() > 2 {
        if let Err(e) = vm.load_input(&args[2]) {
            println!("Error loading input: {:?}", e);
            return;
        }
    }

    vm.run();
}
