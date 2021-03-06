#![allow(dead_code)]

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::fmt::{Debug, Formatter, Result as FmtResult};

use rand::{thread_rng, Rng};
use sdl2::Sdl;
use sdl2::rect::Rect;
use ep::FromPrimitive;

use screen::Screen;
use instruction::{Opcodes, Instruction};

const FONT_SET: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
    0x90, 0x90, 0xf0, 0x10, 0x10, // 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
    0xf0, 0x10, 0x20, 0x40, 0x40, // 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
    0xf0, 0x90, 0xf0, 0x90, 0x90, // A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
    0xf0, 0x80, 0x80, 0x80, 0x80, // C
    0xe0, 0x90, 0x90, 0x90, 0xe0, // D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
    0xf0, 0x80, 0xf0, 0x80, 0x80, // F
];

pub struct Chip8 {
    regs:   [u8; 16],
    i:      u16, 
    dt:     u8,
    st:     u8,
    pc:     usize,

    inst:   Instruction,
    jmp:    bool,

    keys:   [u8; 16],
    mem:    [u8; 4096],
    stack:  Vec<usize>,
    screen: Screen,
}

impl Debug for Chip8 {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.regs[..].fmt(f)
    }
}

impl Chip8 {
    pub fn new(sdl: &Sdl) -> Chip8 {
        Chip8 {
            regs:   [0; 16],
            i:      0,
            dt:     0,
            st:     0,
            pc:     0x200,
            inst:   Instruction::new(),
            jmp:    false,
            keys:   [0; 16],
            mem:    [0; 4096], 
            stack:  vec![],
            screen: Screen::new(sdl),
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let mut rom: Vec<u8> = Vec::new();
        let mut file = File::open(Path::new(path)).unwrap();
        file.read_to_end(&mut rom).unwrap();
        for i in 0usize..rom.len() {
            self.mem[0x200 + i] = rom[i]; 
        }
        // TODO: move this
        for i in 0usize..FONT_SET.len() {
            self.mem[0x00 + i] = FONT_SET[i];
        }
    }

    pub fn run(&mut self) {
        let raw_data = (self.mem[self.pc], self.mem[self.pc +1]); 
        self.inst.decode(raw_data); 

        self.jmp  = false;

        // println!("{:#x}: {:#x}", self.pc, self.inst.opcode);

        match Opcodes::from_u16(self.inst.opcode).unwrap() {
            Opcodes::CLS    => self.cls(),
            Opcodes::RET    => self.ret(),
            Opcodes::JMP    => self.jmp(),
            Opcodes::JMP_VA => self.jmp_va(),
            Opcodes::CALL   => self.call(),
            Opcodes::SE_VB  => self.se_vb(),
            Opcodes::SE_VV  => self.se_vv(),
            Opcodes::SNE_VB => self.sne_vb(),
            Opcodes::SNE_VV => self.sne_vv(),
            Opcodes::OR     => self.or(),
            Opcodes::ADD_VB => self.add_vb(),
            Opcodes::ADD_VV => self.add_vv(),
            Opcodes::ADD_IV => self.add_iv(),
            Opcodes::SUB    => self.sub(),
            Opcodes::SUBN   => self.subn(),
            Opcodes::XOR    => self.xor(),
            Opcodes::AND    => self.and(),
            Opcodes::LD_VB  => self.ld_vb(),
            Opcodes::LD_BV  => self.ld_bv(),
            Opcodes::LD_VV  => self.ld_vv(),
            Opcodes::LD_VI  => self.ld_vi(),
            Opcodes::LD_VK  => self.ld_vk(),
            Opcodes::LD_IV  => self.ld_iv(),
            Opcodes::LD_FV  => self.ld_fv(),
            Opcodes::LD_IA  => self.ld_ia(),
            Opcodes::LD_VDT => self.ld_vdt(),
            Opcodes::LD_DTV => self.ld_dtv(),
            Opcodes::LD_STV => self.ld_stv(),
            Opcodes::SHL    => self.shl(),
            Opcodes::SKP    => self.skp(),
            Opcodes::SKNP   => self.sknp(),
            Opcodes::RND    => self.rnd(),
            Opcodes::SHR    => self.shr(),
            Opcodes::DRW    => self.drw(),
            _               => panic!("Unrecognized opcode: {:#x}", self.inst.opcode),
        }

        if !self.jmp { self.inc_pc(); }

        if self.dt > 0 { self.dt -= 1; }
        if self.st > 0 { self.st -= 1; }
    }

    fn set_pc(&mut self, addr: u16) {
        self.pc = addr as usize;
    }

    fn inc_pc(&mut self) {
        self.pc += 2;
    }

    fn dec_pc(&mut self) {
        self.pc -= 2;
    }

    fn cls(&mut self) {
        self.screen.clear();
    }

    fn ret(&mut self) {
        let addr = self.stack.pop().unwrap();
        self.set_pc(addr as u16);
    }

    fn jmp(&mut self) {
        let addr = self.inst.nnn;
        self.set_pc(addr);
        self.jmp = true;
    }

    fn jmp_va(&mut self) {
        let offset = self.regs[0] as u16;
        let addr = self.inst.nnn + offset;
        self.set_pc(addr);
        self.jmp = true;
    }

    fn call(&mut self) {
        let new_addr = self.inst.nnn;
        self.stack.push(self.pc);
        self.set_pc(new_addr);
        self.jmp = true;
    }

    fn se_vb(&mut self) {
        if self.regs[self.inst.x] == self.inst.kk {
            self.inc_pc();
        } 
    }

    fn se_vv(&mut self) {
        if self.regs[self.inst.x] == self.regs[self.inst.y] {
            self.inc_pc();
        }
    }

    fn sne_vb(&mut self) {
        if self.regs[self.inst.x] != self.inst.kk {
            self.inc_pc();
        }
    }

    fn sne_vv(&mut self) {
        if self.regs[self.inst.x] != self.regs[self.inst.y] {
            self.inc_pc();
        }
    }

    fn or(&mut self) {
        self.regs[self.inst.x] |= self.regs[self.inst.y];
    }

    fn and(&mut self) {
        self.regs[self.inst.x] &= self.regs[self.inst.y];
    }

    fn add_vb(&mut self) {
        let idx = self.inst.x;
        self.regs[idx] = self.regs[idx].wrapping_add(self.inst.kk);
    }

    fn add_vv(&mut self) {
        let idx_x = self.inst.x;
        let x = self.regs[idx_x] as u16;
        let y = self.regs[self.inst.y] as u16;

        let res = x + y;
        self.regs[0xF]   = (res > 255) as u8;
        self.regs[idx_x] = res as u8;
    }

    fn add_iv(&mut self) {
        let x   = self.regs[self.inst.x] as u16;
        self.regs[0xF] = ((self.i + x) > 255) as u8;
        self.i += x;
    }

    fn sub(&mut self) {
        let idx_x = self.inst.x;
        let x = self.regs[idx_x];
        let y = self.regs[self.inst.y];

        self.regs[0xF] = (x > y) as u8;
        self.regs[idx_x] = x.wrapping_sub(y);
    }

    fn subn(&mut self) {
        let x = self.regs[self.inst.x];
        let y = self.regs[self.inst.y];

        self.regs[0xF] = (y > x) as u8;
        self.regs[self.inst.x] = y.wrapping_sub(x);
    }

    fn xor(&mut self) {
        self.regs[self.inst.x] ^= self.regs[self.inst.y];
    }

    fn shr(&mut self) {
        let idx_x = self.inst.x;
        let x     = self.regs[idx_x];
        self.regs[0xF] = x & 0x1;
        self.regs[idx_x] = x >> 1;
    }

    fn shl(&mut self) {
        let x = self.regs[self.inst.x];
        let y = self.regs[self.inst.y];

        self.regs[0xF] = ((x & 0xF0) >> 7 == 1) as u8;
        self.regs[self.inst.x] *= 2;
    }

    fn ld_vv(&mut self) {
        self.regs[self.inst.x] = self.regs[self.inst.y];
    }

    fn ld_vb(&mut self) {
        self.regs[self.inst.x] = self.inst.kk;
    }

    fn ld_bv(&mut self) {
        let x = self.regs[self.inst.x];

        self.mem[self.i as usize]     = x / 100;
        self.mem[self.i as usize + 1] = (x / 100) % 10;
        self.mem[self.i as usize + 2] = (x % 100) % 10; 
    }

    fn ld_fv(&mut self) {
        let x = self.regs[self.inst.x];
        self.i = (x * 5) as u16;
    }

    fn ld_vi(&mut self) {
        let x = self.inst.x as usize;

        for i in 0usize..x {
            self.regs[i] = self.mem[self.i as usize + i];  
        }
    }

    fn ld_iv(&mut self) {
        let x = self.inst.x as usize;

        for i in 0usize..x {
            self.mem[self.i as usize + i] = self.regs[i]; 
        }
    }

    fn ld_ia(&mut self) {
        self.i = self.inst.nnn;
    }

    fn ld_vk(&mut self) {
        let mut key_pressed = false;

        for i in 0..self.keys.len() {
            if self.keys[i] == 1 {
                self.regs[self.inst.x] = i as u8;
                key_pressed = true;
            }
        }

        if !key_pressed {
            self.dec_pc();
        }
    }

    fn ld_vdt(&mut self) {
        self.regs[self.inst.x] = self.dt;
    }

    fn ld_dtv(&mut self) {
        self.dt = self.regs[self.inst.x];
    }

    fn ld_stv(&mut self) {
        self.st = self.regs[self.inst.x];
    }

    fn rnd(&mut self) {
        self.regs[self.inst.x] = thread_rng().gen::<u8>() & self.inst.kk;
    }

    fn drw(&mut self) {
        let idx_x = self.inst.x;
        let idx_y = self.inst.y;
        let (x, y) = (self.regs[idx_x] as usize, self.regs[idx_y] as usize);
        let n = self.inst.n as usize;

        self.screen.renderer.clear();
        self.regs[0xF] = 0;

        for i in 0..n {
            let px = self.mem[(self.i + i as u16) as usize];
            for j in 0..8 {
                if px & (0x80 >> j) != 0 {
                    let mut offset = (x+j+(y+i)*64) as u16;

                    offset = if offset >= 64*32 { 
                        64*32-1 
                    } else {
                        offset
                    };

                    if self.screen.buffer[offset as usize] == 1 {
                        self.regs[0xF] = 1;
                    }               
                    self.screen.buffer[offset as usize] ^= 1;
                }
            } 
        } 

        for i in 0usize..32*10 {
            for j in 0usize..64*10 {
                if self.screen.buffer[(j/10)+(i/10)*64] == 1 {
                    self.screen.renderer.fill_rect(
                        Rect::new(j as i32, i as i32, 1, 1)
                        );
                }
            }
        }

        self.screen.draw();
    }

    fn skp(&mut self) {
        let x = self.regs[self.inst.x];
        if self.keys[x as usize] == 1{
            self.inc_pc();
        }
    }

    fn sknp(&mut self) {
        let x = self.regs[self.inst.x];
        if self.keys[x as usize] == 0 {
            self.inc_pc();
        }
    }
    pub fn reset_keys(&mut self) {
        self.keys = [0;16];
    }

    pub fn press(&mut self, key: u8) {
        self.keys[key as usize] = 1;
    }
}
