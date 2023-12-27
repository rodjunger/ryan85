use libc::c_int;

use crate::vm::arch::*;
use std::ffi::CString;
use std::fmt;

use std::error;

#[derive(Clone, Copy, Debug)]
pub enum EmulationError {
    InvalidRegister { register: reg },
    InvalidInstruction { instruction: u8 },
    InvalidMemoryAddress { address: u16 },
    InvalidSyscall { syscall: u8 },
    OtherError,
}

// Implement the Error trait for the custom error type
impl fmt::Display for EmulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            EmulationError::InvalidRegister { register } => {
                write!(f, "Invalid register access. Invalid register: {}", register)
            }
            EmulationError::InvalidInstruction { instruction } => {
                write!(f, "Invalid instruction: {}", instruction)
            }
            EmulationError::InvalidMemoryAddress { address } => {
                write!(f, "Invalid memory address: {}", address)
            }
            EmulationError::InvalidSyscall { syscall } => {
                write!(f, "Invalid syscall: {}", syscall)
            }
            EmulationError::OtherError => write!(f, "Other error occurred"),
        }
    }
}

impl error::Error for EmulationError {}

pub struct Emulator {
    mem: Vec<u8>,
    consts: VMConsts,
}

impl Emulator {
    pub fn new(mem: Vec<u8>, consts: VMConsts) -> Self {
        Self { mem, consts }
    }

    pub fn dump_registers(&self) -> Result<(), EmulationError> {
        println!(
            "a: {}, b: {}, c: {}, d: {}, s: {}, i: {}, f: {}",
            self.read_register(self.consts.registers.a)?,
            self.read_register(self.consts.registers.b)?,
            self.read_register(self.consts.registers.c)?,
            self.read_register(self.consts.registers.d)?,
            self.read_register(self.consts.registers.s)?,
            self.read_register(self.consts.registers.i)?,
            self.read_register(self.consts.registers.f)?
        );
        Ok(())
    }

    //reads a null terminated string starting at 0x300 (RAM) + offset
    pub fn read_string(&self, offset: u8) -> Result<String, EmulationError> {
        let mut result = String::new();
        let mut current_offset = offset;
        loop {
            let c = self.read_memory(current_offset)?;
            if c == 0 {
                return Ok(result);
            }
            result.push(c as char);
            match current_offset.checked_add(1) {
                Some(v) => current_offset = v,
                None => return Err(EmulationError::OtherError), //String never null terminated
            }
        }
    }

    pub fn read_register(&self, register: reg) -> Result<u8, EmulationError> {
        let register_location = self
            .consts
            .registers
            .reg_to_mem_location(register)
            .ok_or(EmulationError::InvalidRegister { register })?;
        self.read_memory_raw(register_location)
    }

    pub fn read_memory_raw(&self, location: u16) -> Result<u8, EmulationError> {
        match self.mem.get(location as usize) {
            Some(val) => Ok(val.to_owned()),
            None => Err(EmulationError::InvalidMemoryAddress { address: location }),
        }
    }

    pub fn read_memory(&self, location: u8) -> Result<u8, EmulationError> {
        self.read_memory_raw(location as u16 + 0x300)
    }

    //Returns new register value or None is register not found
    pub fn write_register(&mut self, register: reg, val: u8) -> Result<(), EmulationError> {
        let register_location = self
            .consts
            .registers
            .reg_to_mem_location(register)
            .ok_or(EmulationError::InvalidRegister { register })?;

        self.write_memory_raw(register_location, val)
    }

    //Writes to a raw memory address
    pub fn write_memory_raw(&mut self, location: u16, val: u8) -> Result<(), EmulationError> {
        //Memory base is 0x300
        match self.mem.get_mut(location as usize) {
            Some(mem) => {
                *mem = val;
                Ok(())
            }
            None => Err(EmulationError::InvalidMemoryAddress { address: location }),
        }
    }

    //Actual writable memory is 0x300-0x400 so any actual write_memory is addr + 0x300 offset
    pub fn write_memory(&mut self, location: u8, val: u8) -> Result<(), EmulationError> {
        println!("Writing memory at offset {}", location);
        self.write_memory_raw(location as u16 + 0x300, val)
    }

    pub fn parse_instruction(
        &self,
        instruction_bytes: &[u8; 3],
    ) -> Result<Instruction, EmulationError> {
        match Instruction::from_bytes(
            instruction_bytes,
            self.consts.instruction_indices,
            self.consts.opcodes,
        ) {
            Some(instruction) => Ok(instruction),
            None => Err(EmulationError::InvalidInstruction {
                instruction: instruction_bytes[0],
            }),
        }
    }

    pub fn execute_next_instruction(&mut self) -> Result<(), EmulationError> {
        let ip = self.read_register(self.consts.registers.i)? as usize;

        // Check for potential overflow when incrementing `ip`
        let incremented_ip = ip.checked_add(1).ok_or(EmulationError::OtherError)?;
        self.write_register(self.consts.registers.i, incremented_ip as u8)?;

        // Fetch the instruction bytes and handle errors
        let instruction_bytes = self.mem.get(ip * 3..(ip * 3) + 3).ok_or_else(|| {
            EmulationError::InvalidMemoryAddress {
                address: (ip * 3) as u16,
            }
        })?;

        // Match on the instruction bytes
        match Instruction::from_bytes(
            instruction_bytes,
            self.consts.instruction_indices,
            self.consts.opcodes,
        ) {
            Some(instruction) => self.interpret_instruction(instruction),
            None => Err(EmulationError::InvalidInstruction {
                instruction: instruction_bytes[0],
            }),
        }
    }

    pub fn interpret_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), EmulationError> {
        self.dump_registers()?;
        instruction.pretty_print(self.consts.registers);
        match instruction {
            Instruction::Imm { dst, val } => self.write_register(dst, val),
            Instruction::Add { dst, src } => self.write_register(
                dst,
                self.read_register(dst)?
                    .wrapping_add(self.read_register(src)?),
            ),
            Instruction::Stk { pop, push } => {
                if push != 0 {
                    // Increase stack pointer
                    self.write_register(
                        self.consts.registers.s,
                        self.read_register(self.consts.registers.s)? + 1,
                    )?;
                    let val = self.read_register(push)?; //Read the register we are going to push
                    self.write_memory(self.read_register(self.consts.registers.s)?, val)?;
                    // Write the value from the register at the stack pointer
                }
                if pop != 0 {
                    // Increase stack pointer
                    let val = self.read_memory(self.read_register(self.consts.registers.s)?)?; // Read the memory value stored at the stack pointer
                    self.write_register(pop, val)?; //Write that value to the register
                                                    // Decrease the stack pointer
                    self.write_register(
                        self.consts.registers.s,
                        self.read_register(self.consts.registers.s)? - 1,
                    )?;
                }
                Ok(())
            }
            Instruction::Stm { dst, src } => {
                let offset = self.read_register(dst)?;
                self.write_memory(offset, self.read_register(src)?)
            }
            Instruction::Ldm { dst, src } => {
                let val = self.read_memory(self.read_register(src)?)?; //Read memory pointed by the register src
                self.write_register(dst, val)
            }
            Instruction::Cmp { left, right } => {
                let mut new_flags: u8 = 0;
                let left_value = self.read_register(left)?;
                let right_value = self.read_register(right)?;

                println!(
                    "\nbig boy comp: Left: {}, right: {}\n",
                    left_value, right_value
                );

                if (left_value == 0) && (right_value == 0) {
                    new_flags |= self.consts.cmp_flags.zero;
                }
                if left_value < right_value {
                    new_flags |= self.consts.cmp_flags.smaller
                }
                if left_value == right_value {
                    new_flags |= self.consts.cmp_flags.equals
                }
                if left_value > right_value {
                    new_flags |= self.consts.cmp_flags.bigger
                }
                if left_value != right_value {
                    //This is an else on the ghidra decompilation, which makes sense since if it's not equal it must be different, but making it explicit looks better
                    new_flags |= self.consts.cmp_flags.not_equals
                }

                self.write_register(self.consts.registers.f, new_flags)
            }
            Instruction::Jmp { flags, dst } => {
                if flags == 0 || self.read_register(self.consts.registers.f)? & flags != 0 {
                    //take the jump
                    return self.write_register(self.consts.registers.i, self.read_register(dst)?);
                }
                Ok(())
            }
            Instruction::Sys { num, dst } => {
                match num {
                    num if num == self.consts.syscalls.write => {
                        // write

                        let fd = self.read_register(self.consts.registers.a)?;
                        let origin_offset = self.read_register(self.consts.registers.b)?;
                        let mut n_bytes = self.read_register(self.consts.registers.c)? as usize;
                        let max_bytes = 0x100 - origin_offset as usize;

                        // Ensure we don't write after our memory region, which goes up to 0x400
                        n_bytes = n_bytes.min(max_bytes);

                        let buffer = &self.mem[0x300 + origin_offset as usize..][..n_bytes];

                        println!("Attempting to write '{:#?}' to fd {}", buffer, fd);

                        unsafe {
                            let num_written = libc::write(
                                fd.into(),
                                buffer.as_ptr() as *mut std::ffi::c_void,
                                n_bytes,
                            ) as isize; // Changed to isize to match libc::write return type
                            if num_written >= 0 {
                                println!("Wrote {} bytes into fd {}", num_written, fd);
                                self.write_register(dst, num_written as u8)?
                            } else {
                                println!("Error writing into fd {}", fd)
                            }
                        }
                        Ok(())
                    }
                    num if num == self.consts.syscalls.read_memory => {
                        //read_memory
                        println!("[s] ... read_memory");

                        let fd = self.read_register(self.consts.registers.a)?;
                        let dest_offset = self.read_register(self.consts.registers.b)?;
                        let n_bytes = self.read_register(self.consts.registers.c)? as usize; // Use usize for buffer size
                        let max_bytes = 0x100 - dest_offset as usize;

                        let n_bytes = n_bytes.min(max_bytes); // Use min function for clarity

                        let mut buffer: Vec<u8> = vec![0; n_bytes]; // Initialize buffer with zeros directly

                        unsafe {
                            let num_read = libc::read(
                                fd.into(),
                                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                                n_bytes,
                            );

                            if num_read >= 0 {
                                let num_read = num_read as usize; // Cast to usize for indexing
                                for i in 0..num_read {
                                    self.write_memory(dest_offset + i as u8, buffer[i])?;
                                }
                                println!(
                                    "Read {} bytes from fd {} into offset {}",
                                    num_read, fd, dest_offset
                                );
                            } else {
                                println!("Error reading from fd {}", fd);
                            }
                            Ok(())
                        }
                    }
                    num if num == self.consts.syscalls.open => {
                        //Open
                        println!("[s] ... open");
                        let path: String =
                            self.read_string(self.read_register(self.consts.registers.a)?)?;
                        let flags = self.read_register(self.consts.registers.b)?;
                        let mode = self.read_register(self.consts.registers.c)?;
                        unsafe {
                            let path_c = CString::new(path).unwrap();
                            let fd: u8 = match libc::open(
                                path_c.as_ptr().into(),
                                flags.into(),
                                mode as c_int,
                            )
                            .try_into()
                            {
                                Ok(fd) => fd,
                                Err(_err) => return Err(EmulationError::OtherError),
                            };
                            self.write_register(dst, fd)?;
                        };
                        Ok(())
                    }
                    _ => return Err(EmulationError::InvalidSyscall { syscall: num }),
                }
            }
        }
    }
}
