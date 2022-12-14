use crate::{bus::Bus, instruction::INSTRUCTIONS, mem::Mem};

pub const STACK: u16 = 0x0100;

#[derive(Debug, Default, Clone)]
pub struct Status {
    pub carry: bool,
    pub zero: bool,
    pub disable_interrupts: bool,
    pub decimal: bool,
    pub b1: bool,
    pub b2: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl From<u8> for Status {
    fn from(v: u8) -> Self {
        Self {
            carry: v & 0b0000_0001 != 0,
            zero: v & 0b0000_0010 != 0,
            disable_interrupts: v & 0b0000_0100 != 0,
            decimal: v & 0b0000_1000 != 0,
            b1: v & 0b0001_0000 != 0,
            b2: v & 0b0010_0000 != 0,
            overflow: v & 0b0100_0000 != 0,
            negative: v & 0b1000_0000 != 0,
        }
    }
}

impl From<&Status> for u8 {
    fn from(v: &Status) -> Self {
        let mut result = 0;
        result |= v.carry as u8;
        result |= (v.zero as u8) << 1;
        result |= (v.disable_interrupts as u8) << 2;
        result |= (v.decimal as u8) << 3;
        result |= (v.b1 as u8) << 4;
        result |= (v.b2 as u8) << 5;
        result |= (v.overflow as u8) << 6;
        result |= (v.negative as u8) << 7;
        result
    }
}

impl From<Status> for u8 {
    fn from(v: Status) -> Self {
        let mut result = 0;
        result |= v.carry as u8;
        result |= (v.zero as u8) << 1;
        result |= (v.disable_interrupts as u8) << 2;
        result |= (v.decimal as u8) << 3;
        result |= (v.b1 as u8) << 4;
        result |= (v.b2 as u8) << 5;
        result |= (v.overflow as u8) << 6;
        result |= (v.negative as u8) << 7;
        result
    }
}

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
    None,
}

#[derive(Debug)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub status: Status,
    pub pc: u16,
    pub running: bool,
    pub cycles: usize,
    pub bus: Bus,
}

impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A={:#04x} X={:#04x} Y={:#04x} SP={:#04x} PC={:#06x} {:#010b}",
            self.a,
            self.x,
            self.y,
            self.sp,
            self.pc,
            u8::from(&self.status),
        )
    }
}

impl Mem for Cpu {
    #[inline(always)]
    fn read_byte(&self, addr: u16) -> u8 {
        self.bus.read_byte(addr)
    }

    #[inline(always)]
    fn write_byte(&mut self, addr: u16, value: u8) {
        self.bus.write_byte(addr, value);
    }

    #[inline(always)]
    fn read_word(&self, addr: u16) -> u16 {
        self.bus.read_word(addr)
    }

    #[inline(always)]
    fn write_word(&mut self, addr: u16, value: u16) {
        self.bus.write_word(addr, value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    A,
    X,
    Y,
    S,
}

#[derive(Debug, Clone, Copy)]
pub enum BranchCondition {
    CarrySet,
    CarryClear,
    ZeroSet,
    ZeroClear,
    MinusSet,
    MinusClear,
    OverflowSet,
    OverflowClear,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        let mut cpu = Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            status: 0.into(),
            pc: 0,
            running: true,
            cycles: 0,
            bus,
        };
        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = 0b00100100.into();
        self.pc = self.read_word(0xFFFC);
        self.cycles = 0;
    }

    fn update_zero_and_negative(&mut self, value: u8) {
        self.status.zero = value == 0;
        self.status.negative = value >> 7 == 1;
    }

    fn update_negative(&mut self, value: u8) {
        self.status.negative = value >> 7 == 1;
    }

    fn load(&mut self, register: Register, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        self.set_register(register, value);
        self.update_zero_and_negative(value);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Cpu),
    {
        while self.running {
            callback(self);
            self.step();
        }
    }

    pub fn step(&mut self) {
        let opcode = self.read_byte(self.pc);

        let ins = &INSTRUCTIONS[opcode as usize];

        let pc = self.pc + 1;
        self.pc += 1;
        self.cycles += ins.cycles;

        trace!("{}, {:?}", self, ins);

        match opcode {
            0x00 => {
                self.status.b1 = true;
                self.running = false;
                return;
            }

            0xEA => (),
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA | 0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 | 0x04
            | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 | 0x0C | 0x1C | 0x3C | 0x5C
            | 0x7C | 0xDC | 0xFC => (),

            0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.adc(&ins.mode),
            0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 | 0xEB => self.sbc(&ins.mode),

            0x4A => self.lsr_a(),
            0x46 | 0x56 | 0x4E | 0x5E => {
                self.lsr(&ins.mode);
            }

            0x4C | 0x6C => self.jmp(&ins.mode),
            0x20 => self.jsr(&ins.mode),
            0x40 => self.rti(),
            0x60 => self.rts(),

            0xB0 => self.branch(BranchCondition::CarrySet, &ins.mode),
            0x90 => self.branch(BranchCondition::CarryClear, &ins.mode),
            0xF0 => self.branch(BranchCondition::ZeroSet, &ins.mode),
            0xD0 => self.branch(BranchCondition::ZeroClear, &ins.mode),
            0x30 => self.branch(BranchCondition::MinusSet, &ins.mode),
            0x10 => self.branch(BranchCondition::MinusClear, &ins.mode),
            0x70 => self.branch(BranchCondition::OverflowSet, &ins.mode),
            0x50 => self.branch(BranchCondition::OverflowClear, &ins.mode),

            0x0A => self.asl_a(),
            0x06 | 0x16 | 0x0E | 0x1E => {
                self.asl(&ins.mode);
            }

            0x6A => self.ror_a(),
            0x66 | 0x76 | 0x6E | 0x7E => {
                self.ror(&ins.mode);
            }

            0x2A => self.rol_a(),
            0x26 | 0x36 | 0x2E | 0x3E => {
                self.rol(&ins.mode);
            }

            0x07 | 0x17 | 0x0F | 0x1F | 0x1B | 0x03 | 0x13 => self.slo(&ins.mode),

            0x27 | 0x37 | 0x2F | 0x3F | 0x3b | 0x33 | 0x23 => self.rla(&ins.mode),

            0x47 | 0x57 | 0x4F | 0x5f | 0x5b | 0x43 | 0x53 => self.sre(&ins.mode),

            0x67 | 0x77 | 0x6f | 0x7f | 0x7b | 0x63 | 0x73 => self.rra(&ins.mode),

            0x24 | 0x2C => self.bit(&ins.mode),

            0x18 => self.status.carry = false,
            0x38 => self.status.carry = true,
            0x58 => self.status.disable_interrupts = false,
            0x78 => self.status.disable_interrupts = true,
            0xB8 => self.status.overflow = false,
            0xD8 => self.status.decimal = false,
            0xF8 => self.status.decimal = true,
            0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                self.load(Register::A, &ins.mode)
            }

            0x08 => self.php(),
            0x28 => self.plp(),
            0x48 => self.pha(),
            0x68 => self.pla(),

            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.load(Register::X, &ins.mode),
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.load(Register::Y, &ins.mode),

            0xA7 | 0xB7 | 0xAF | 0xBF | 0xA3 | 0xB3 => self.lax(&ins.mode),

            0x87 | 0x97 | 0x8f | 0x83 => self.sax(&ins.mode),

            0xAA => self.transfer(Register::A, Register::X),
            0x8A => self.transfer(Register::X, Register::A),
            0xA8 => self.transfer(Register::A, Register::Y),
            0x98 => self.transfer(Register::Y, Register::A),
            0x9A => self.transfer(Register::X, Register::S),
            0xBA => self.transfer(Register::S, Register::X),

            0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.store(Register::A, &ins.mode),
            0x86 | 0x96 | 0x8E => self.store(Register::X, &ins.mode),
            0x84 | 0x94 | 0x8C => self.store(Register::Y, &ins.mode),

            0xE8 => self.increment_register(Register::X),
            0xC8 => self.increment_register(Register::Y),
            0xCA => self.decrement_register(Register::X),
            0x88 => self.decrement_register(Register::Y),

            0xE6 | 0xF6 | 0xEE | 0xFE => {
                self.increment_memory(&ins.mode);
            }
            0xC6 | 0xD6 | 0xCE | 0xDE => {
                self.decrement_memory(&ins.mode);
            }

            0xE7 | 0xF7 | 0xEF | 0xFF | 0xFB | 0xE3 | 0xF3 => self.isb(&ins.mode),

            0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and(&ins.mode),
            0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.ora(&ins.mode),
            0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.eor(&ins.mode),
            0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                self.compare(Register::A, &ins.mode)
            }

            0xC7 | 0xD7 | 0xCF | 0xDF | 0xDB | 0xD3 | 0xC3 => self.dcp(&ins.mode),

            0xE0 | 0xE4 | 0xEC => self.compare(Register::X, &ins.mode),
            0xC0 | 0xC4 | 0xCC => self.compare(Register::Y, &ins.mode),

            _ => todo!("{:?}", ins),
        }

        if pc == self.pc {
            self.pc += (ins.bytes as u16) - 1;
        }
    }

    #[inline(always)]
    fn get_register(&self, register: Register) -> u8 {
        match register {
            Register::A => self.a,
            Register::X => self.x,
            Register::Y => self.y,
            Register::S => self.sp,
        }
    }

    #[inline(always)]
    fn set_register(&mut self, register: Register, value: u8) {
        match register {
            Register::A => self.a = value,
            Register::X => self.x = value,
            Register::Y => self.y = value,
            Register::S => self.sp = value,
        }
    }

    pub(crate) fn get_operand_address(&self, mode: &AddressingMode, pc: u16) -> u16 {
        match mode {
            AddressingMode::Immediate => pc,
            AddressingMode::ZeroPage => self.read_byte(pc) as u16,
            AddressingMode::ZeroPageX => {
                let addr = self.read_byte(pc);
                addr.wrapping_add(self.x) as u16
            }
            AddressingMode::ZeroPageY => {
                let addr = self.read_byte(pc);
                addr.wrapping_add(self.y) as u16
            }
            AddressingMode::Absolute => self.read_word(pc),
            AddressingMode::AbsoluteX => {
                let addr = self.read_word(pc);
                addr.wrapping_add(self.x as u16)
            }
            AddressingMode::AbsoluteY => {
                let addr = self.read_word(pc);
                addr.wrapping_add(self.y as u16)
            }
            AddressingMode::Indirect => {
                let mem_address = self.read_word(pc);
                // 6502 bug mode with with page boundary:
                // If address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
                // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
                // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000

                if mem_address & 0x00FF == 0x00FF {
                    let lo = self.read_byte(mem_address);
                    let hi = self.read_byte(mem_address & 0xFF00);
                    (hi as u16) << 8 | (lo as u16)
                } else {
                    self.read_word(mem_address)
                }
            }
            AddressingMode::IndirectX => {
                let base = self.read_byte(pc);
                let ptr = base.wrapping_add(self.x);
                let lo = self.read_byte(ptr as u16) as u16;
                let hi = self.read_byte(ptr.wrapping_add(1) as u16) as u16;
                hi << 8 | lo
            }
            AddressingMode::IndirectY => {
                let base = self.read_byte(pc);
                let lo = self.read_byte(base as u16);
                let hi = self.read_byte(base.wrapping_add(1) as u16);
                let ptr = (hi as u16) << 8 | (lo as u16);
                ptr.wrapping_add(self.y as u16)
            }
            AddressingMode::Relative => {
                let offset = self.read_byte(pc) as i8;
                pc.wrapping_add(offset as u16)
            }
            AddressingMode::None => unreachable!("addressing mode {:?} is not supported", mode),
        }
    }

    fn transfer(&mut self, src: Register, dst: Register) {
        let value = self.get_register(src);
        self.set_register(dst, value);
        if dst != Register::S {
            self.update_zero_and_negative(value);
        }
    }

    fn increment_register(&mut self, register: Register) {
        let value = self.get_register(register).wrapping_add(1);
        self.set_register(register, value);
        self.update_zero_and_negative(value);
    }

    fn decrement_register(&mut self, register: Register) {
        let value = self.get_register(register).wrapping_sub(1);
        self.set_register(register, value);
        self.update_zero_and_negative(value);
    }

    fn increment_memory(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr).wrapping_add(1);
        self.write_byte(addr, value);
        self.update_zero_and_negative(value);
        value
    }

    fn decrement_memory(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr).wrapping_sub(1);
        self.write_byte(addr, value);
        self.update_zero_and_negative(value);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        self.a &= value;
        self.update_zero_and_negative(self.a);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        self.a |= value;
        self.update_zero_and_negative(self.a);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        self.a ^= value;
        self.update_zero_and_negative(self.a);
    }

    fn compare(&mut self, register: Register, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        let register_value = self.get_register(register);
        self.status.carry = value <= register_value;
        self.update_zero_and_negative(register_value.wrapping_sub(value));
    }

    fn store(&mut self, register: Register, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.get_register(register);
        self.write_byte(addr, value);
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        self.status.zero = value & self.a == 0;
        self.status.negative = value & 0x80 != 0;
        self.status.overflow = value & 0x40 != 0;
    }

    fn branch(&mut self, condition: BranchCondition, mode: &AddressingMode) {
        let condition_met = match condition {
            BranchCondition::CarrySet => self.status.carry,
            BranchCondition::CarryClear => !self.status.carry,
            BranchCondition::ZeroSet => self.status.zero,
            BranchCondition::ZeroClear => !self.status.zero,
            BranchCondition::MinusSet => self.status.negative,
            BranchCondition::MinusClear => !self.status.negative,
            BranchCondition::OverflowSet => self.status.overflow,
            BranchCondition::OverflowClear => !self.status.overflow,
        };

        if condition_met {
            self.pc = self.get_operand_address(mode, self.pc);
        }

        self.pc += 1;
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        self.push_word(self.pc.wrapping_add(1));
        self.pc = addr;
    }

    fn rts(&mut self) {
        self.pc = self.pop_word().wrapping_add(1);
    }

    fn push_byte(&mut self, value: u8) {
        self.write_byte(STACK + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop_byte(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read_byte(STACK + self.sp as u16)
    }

    fn push_word(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        self.push_byte(hi);
        self.push_byte(lo);
    }

    fn pop_word(&mut self) -> u16 {
        let lo = self.pop_byte() as u16;
        let hi = self.pop_byte() as u16;
        hi << 8 | lo
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        self.add_to_a(value);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        let value = ((value as i8).wrapping_neg().wrapping_sub(1)) as u8;
        self.add_to_a(value);
    }

    fn add_to_a(&mut self, value: u8) {
        let carry = if self.status.carry { 1 } else { 0 };
        let sum = self.a as u16 + value as u16 + carry;

        self.status.carry = sum > 0xFF;
        let sum = sum as u8;

        self.status.overflow = (value ^ sum) & (sum ^ self.a) & 0x80 != 0;

        self.a = sum;
        self.update_zero_and_negative(self.a);
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        self.pc = self.get_operand_address(mode, self.pc);
    }

    fn lsr(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode, self.pc);
        let mut value = self.read_byte(addr);
        self.status.carry = value & 1 == 1;
        value >>= 1;
        self.write_byte(addr, value);
        self.update_zero_and_negative(value);
        value
    }

    fn lsr_a(&mut self) {
        self.status.carry = self.a & 1 == 1;
        self.a >>= 1;
        self.update_zero_and_negative(self.a);
    }

    fn php(&mut self) {
        let mut status = self.status.clone();
        status.b1 = true;
        status.b2 = true;
        self.push_byte(status.into());
    }

    fn plp(&mut self) {
        self.status = self.pop_byte().into();
        self.status.b1 = false;
        self.status.b2 = true;
    }

    fn pha(&mut self) {
        self.push_byte(self.a);
    }

    fn pla(&mut self) {
        self.a = self.pop_byte();
        self.update_zero_and_negative(self.a);
    }

    fn rti(&mut self) {
        self.status = self.pop_byte().into();
        self.status.b1 = false;
        self.status.b2 = true;
        self.pc = self.pop_word();
    }

    fn asl_a(&mut self) {
        let data = self.a;
        self.status.carry = data >> 7 == 1;
        self.a = data << 1;
        self.update_zero_and_negative(self.a);
    }

    fn asl(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        self.status.carry = value >> 7 == 1;
        let value = value << 1;
        self.write_byte(addr, value);
        self.update_zero_and_negative(value);
        value
    }

    fn ror_a(&mut self) {
        let value = self.a;
        let carry = if self.status.carry { 1 } else { 0 };
        self.status.carry = value & 0x01 == 1;
        self.a = (carry << 7) | (value >> 1);
        self.update_zero_and_negative(self.a);
    }

    fn ror(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        let carry = if self.status.carry { 1 } else { 0 };
        self.status.carry = value & 0x01 == 1;
        let value = (carry << 7) | (value >> 1);
        self.write_byte(addr, value);
        self.update_negative(value);
        value
    }

    fn rol_a(&mut self) {
        let value = self.a;
        let carry = if self.status.carry { 1 } else { 0 };
        self.status.carry = value >> 7 == 1;
        self.a = (value << 1) | carry;
        self.update_zero_and_negative(self.a)
    }

    fn rol(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr);
        let carry = if self.status.carry { 1 } else { 0 };
        self.status.carry = value >> 7 == 1;
        let value = (value << 1) | carry;
        self.write_byte(addr, value);
        self.update_negative(value);
        value
    }

    fn lax(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        self.a = self.read_byte(addr);
        self.x = self.a;
        self.update_zero_and_negative(self.a);
    }

    fn sax(&mut self, mode: &AddressingMode) {
        let value = self.a & self.x;
        let addr = self.get_operand_address(mode, self.pc);
        self.write_byte(addr, value);
    }

    fn dcp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode, self.pc);
        let value = self.read_byte(addr).wrapping_sub(1);
        self.write_byte(addr, value);
        self.status.carry = value <= self.a;
        self.update_zero_and_negative(self.a.wrapping_sub(value));
    }

    fn isb(&mut self, mode: &AddressingMode) {
        let value = self.increment_memory(mode);
        let value = ((value as i8).wrapping_neg().wrapping_sub(1)) as u8;
        self.add_to_a(value);
    }

    fn slo(&mut self, mode: &AddressingMode) {
        self.a |= self.asl(mode);
        self.update_zero_and_negative(self.a);
    }

    fn rla(&mut self, mode: &AddressingMode) {
        self.a &= self.rol(mode);
        self.update_zero_and_negative(self.a);
    }

    fn sre(&mut self, mode: &AddressingMode) {
        self.a ^= self.lsr(mode);
        self.update_zero_and_negative(self.a);
    }

    fn rra(&mut self, mode: &AddressingMode) {
        let value = self.ror(mode);
        self.add_to_a(value);
    }
}
