use std::fmt;

pub type reg = u8;

#[derive(Clone, Copy, Debug)]
pub struct InstructionOpcodes {
    pub imm: u8,
    pub add: u8,
    pub stk: u8,
    pub stm: u8,
    pub ldm: u8,
    pub cmp: u8,
    pub jmp: u8,
    pub sys: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Syscalls {
    pub open: u8,
    pub read_memory: u8,
    pub write: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct VMConsts {
    pub opcodes: InstructionOpcodes,
    pub syscalls: Syscalls,
    pub registers: Registers,
    pub instruction_indices: InstructionDecodeIndices,
    pub cmp_flags: CmpFlags,
}

#[derive(Clone, Copy, Debug)]
pub struct CmpFlags {
    pub smaller: u8,    // left < right
    pub bigger: u8,     // left > right
    pub equals: u8,     //left == right
    pub not_equals: u8, // left != right
    pub zero: u8,       // left == 0 && right == 0
}

#[derive(Clone, Copy, Debug)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub s: u8,
    pub i: u8,
    pub f: u8,
    pub none: u8,
}

/*
pub const SYS_OPCODE: u8 = 0x1;
pub const CMP_OPCODE: u8 = 0x2;
pub const STK_OPCODE: u8 = 0x4;
pub const LDM_OPCODE: u8 = 0x8;
pub const STM_OPCODE: u8 = 0x10;
pub const IMM_OPCODE: u8 = 0x20;
pub const JMP_OPCODE: u8 = 0x40;
pub const ADD_OPCODE: u8 = 0x80;


pub const IMM_OPCODE: u8 = 0x10;
pub const ADD_OPCODE: u8 = 0x1;
pub const STK_OPCODE: u8 = 0x4;
pub const STM_OPCODE: u8 = 0x2;
pub const LDM_OPCODE: u8 = 0x20;
pub const CMP_OPCODE: u8 = 0x40;
pub const JMP_OPCODE: u8 = 0x80;
pub const SYS_OPCODE: u8 = 0x8;


pub const REG_A: reg = 0x20;
pub const REG_B: reg = 0x4;
pub const REG_C: reg = 0x10;
pub const REG_D: reg = 0x40;
pub const REG_S: reg = 0x1;
pub const REG_I: reg = 0x2;
pub const REG_F: reg = 0x8;
*/

pub const REG_NONE: reg = 0x0;

impl Registers {
    pub fn reg_byte_to_str(&self, reg_value: reg) -> &'static str {
        match reg_value {
            reg if reg == self.a => "a",
            reg if reg == self.b => "b",
            reg if reg == self.c => "c",
            reg if reg == self.d => "d",
            reg if reg == self.s => "s",
            reg if reg == self.i => "i",
            reg if reg == self.f => "f",
            REG_NONE => "NONE",
            _ => "Unknown",
        }
    }

    pub fn reg_str_to_byte(&self, reg_str: &str) -> Option<u8> {
        Some(match reg_str {
            "a" => self.a,
            "b" => self.b,
            "c" => self.c,
            "d" => self.d,
            "s" => self.s,
            "i" => self.i,
            "f" => self.f,
            "NONE" => 0,
            _ => return None,
        })
    }
    pub fn reg_to_mem_location(&self, reg_value: reg) -> Option<u16> {
        let result = match reg_value {
            reg if reg == self.a => 0x400,
            reg if reg == self.b => 0x401,
            reg if reg == self.c => 0x402,
            reg if reg == self.d => 0x403,
            reg if reg == self.s => 0x404,
            reg if reg == self.i => 0x405,
            reg if reg == self.f => 0x406,
            REG_NONE => 0xffff,
            _ => return None,
        };
        Some(result)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    Sys { num: u8, dst: reg }, // Calls syscall num and stores returned value in dst reg
    Cmp { left: reg, right: reg }, // Compares left reg with right reg
    Stk { pop: reg, push: reg }, // Stack
    Ldm { dst: reg, src: reg }, // Stores value from the memory location pointed by src to the dst reg
    Stm { dst: reg, src: reg }, // Store value from src reg to the memory location pointed by dst reg
    Imm { dst: reg, val: u8 },  // Load immediate value to register
    Jmp { flags: u8, dst: reg }, // Jmps to location stored in dst flag is current flags match flags value
    Add { dst: reg, src: reg },  // I wonder what this could be
}

//Best we can do without info of opcodes
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Instruction::Sys { num, dst } => {
                write!(f, "SYS {{ num: {}, dst: {} }}", num, dst)
            }
            Instruction::Cmp { left, right } => {
                write!(f, "CMP {{ left: {} , right: {} }}", left, right)
            }
            Instruction::Stk { pop, push } => {
                write!(f, "STK {{ pop: {}, push: {} }}", pop, push)
            }
            Instruction::Ldm { dst, src } => {
                write!(f, "LDM {{ dst: {}, src: *{} }}", dst, src)
            }
            Instruction::Stm { dst, src } => {
                write!(f, "STM {{ dst: *{}, src: {} }}", dst, src)
            }
            Instruction::Imm { dst, val } => {
                write!(f, "IMM {{ dst: {}, val: {} }}", dst, val)
            }
            Instruction::Jmp { flags, dst } => {
                write!(f, "JMP {{ flags: {}, dst: {} }}", flags, dst)
            }
            Instruction::Add { dst, src } => {
                write!(f, "ADD {{ dst: {}, src: {} }}", dst, src)
            }
        }
    }
}
impl Instruction {
    pub fn pretty_print(&self, mapping: Registers) {
        match *self {
            Instruction::Sys { num, dst } => {
                println!(
                    "SYS {{ num: {}, dst: {} }}",
                    num,
                    mapping.reg_byte_to_str(dst)
                )
            }
            Instruction::Cmp { left, right } => {
                println!(
                    "CMP {{ left: {} , right: {} }}",
                    mapping.reg_byte_to_str(left),
                    mapping.reg_byte_to_str(right)
                )
            }
            Instruction::Stk { pop, push } => {
                println!(
                    "STK {{ pop: {}, push: {} }}",
                    mapping.reg_byte_to_str(pop),
                    mapping.reg_byte_to_str(push)
                )
            }
            Instruction::Ldm { dst, src } => {
                println!(
                    "LDM {{ dst: {}, src: *{} }}",
                    mapping.reg_byte_to_str(dst),
                    mapping.reg_byte_to_str(src)
                )
            }
            Instruction::Stm { dst, src } => {
                println!(
                    "STM {{ dst: *{}, src: {} }}",
                    mapping.reg_byte_to_str(dst),
                    mapping.reg_byte_to_str(src)
                )
            }
            Instruction::Imm { dst, val } => {
                println!(
                    "IMM {{ dst: {}, val: {} }}",
                    mapping.reg_byte_to_str(dst),
                    val
                )
            }
            Instruction::Jmp { flags, dst } => {
                println!(
                    "JMP {{ flags: {}, dst: {} }}",
                    flags,
                    mapping.reg_byte_to_str(dst)
                )
            }
            Instruction::Add { dst, src } => {
                println!(
                    "ADD {{ dst: {}, src: {} }}",
                    mapping.reg_byte_to_str(dst),
                    mapping.reg_byte_to_str(src)
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InstructionDecodeIndices {
    pub opcode: usize,
    pub left_param: usize,
    pub right_param: usize,
}

//Turns out that instruction are dynamic both on the opcodes and the locations of each one of the 3 parameters (opcode, left_param and right_param)
impl Instruction {
    pub fn from_bytes(
        instruction_bytes: &[u8],
        indices: InstructionDecodeIndices,
        opcodes: InstructionOpcodes,
    ) -> Option<Self> {
        if instruction_bytes.len() != 3 {
            return None;
        }

        let opcode = instruction_bytes[indices.opcode];
        let left_param = instruction_bytes[indices.left_param];
        let right_param = instruction_bytes[indices.right_param];

        match opcode {
            opcode if opcode == opcodes.sys => Some(Instruction::Sys {
                num: left_param,
                dst: right_param,
            }),
            opcode if opcode == opcodes.cmp => Some(Instruction::Cmp {
                left: left_param,
                right: right_param,
            }),
            opcode if opcode == opcodes.stk => Some(Instruction::Stk {
                pop: left_param,
                push: right_param,
            }),
            opcode if opcode == opcodes.ldm => Some(Instruction::Ldm {
                dst: left_param,
                src: right_param,
            }),
            opcode if opcode == opcodes.stm => Some(Instruction::Stm {
                dst: left_param,
                src: right_param,
            }),
            opcode if opcode == opcodes.imm => Some(Instruction::Imm {
                dst: left_param,
                val: right_param,
            }),
            opcode if opcode == opcodes.jmp => Some(Instruction::Jmp {
                flags: left_param,
                dst: right_param,
            }),
            opcode if opcode == opcodes.add => Some(Instruction::Add {
                dst: left_param,
                src: right_param,
            }),
            _ => None,
        }
    }
}
