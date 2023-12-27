use std::{error, fmt};

use super::arch::{Registers, VMConsts};

#[derive(Clone, Debug)]
pub enum InvalidInstruction {
    InvalidRegister { register: String, line: usize },
    InvalidOperation { operation: String, line: usize },
    InvalidNumber { number: String, line: usize },
    InvalidNumberOfParts { lines: usize, line: usize },
}

impl error::Error for InvalidInstruction {}

impl fmt::Display for InvalidInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidInstruction::InvalidRegister { register, line } => {
                write!(f, "Invalid register at line {}: {}", line, register)
            }
            InvalidInstruction::InvalidOperation { operation, line } => {
                write!(f, "Invalid Operation at line {}: {}", line, operation)
            }
            InvalidInstruction::InvalidNumberOfParts { lines, line } => {
                write!(
                    f,
                    "Invalid instruction at line {}. Every instruct must have 3 parts. Found {}",
                    line, lines
                )
            }
            InvalidInstruction::InvalidNumber { number, line } => {
                write!(f, "Invalid Number at line {}: {}", line, number)
            }
        }
    }
}

fn parse_num(num: &str, line: usize) -> Result<u8, InvalidInstruction> {
    match num.parse::<u8>() {
        Ok(parsed) => Ok(parsed), //Maybe also check if it's a valid number in the context of syscalls
        Err(_) => Err(InvalidInstruction::InvalidNumber {
            number: num.to_string(),
            line,
        }),
    }
}

fn parse_reg(reg: &str, registers: &Registers, line: usize) -> Result<u8, InvalidInstruction> {
    match registers.reg_str_to_byte(reg) {
        Some(res) => Ok(res),
        None => Err(InvalidInstruction::InvalidRegister {
            register: reg.to_string(),
            line,
        }),
    }
}

pub fn assemble(code: String, ctx: VMConsts) -> Result<Vec<u8>, InvalidInstruction> {
    let mut result: Vec<u8> = vec![];

    for (i, line) in code.lines().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(InvalidInstruction::InvalidNumberOfParts {
                lines: parts.len(),
                line: i,
            });
        }
        let op = parts[0];
        let left = parts[1];
        let right = parts[2];

        //TODO: Fix ordering based on ctx
        match op {
            "SYS" => {
                result.push(ctx.opcodes.sys);
                result.push(parse_num(left, i)?);
                result.push(parse_reg(right, &ctx.registers, i)?)
            }
            "CMP" => {
                result.push(ctx.opcodes.cmp);
                result.push(parse_reg(left, &ctx.registers, i)?);
                result.push(parse_reg(right, &ctx.registers, i)?)
            }
            "STK" => {
                result.push(ctx.opcodes.stk);
                result.push(parse_reg(left, &ctx.registers, i)?);
                result.push(parse_reg(right, &ctx.registers, i)?)
            }
            "LDM" => {
                result.push(ctx.opcodes.ldm);
                result.push(parse_reg(left, &ctx.registers, i)?);
                result.push(parse_reg(right, &ctx.registers, i)?)
            }
            "STM" => {
                result.push(ctx.opcodes.stm);
                result.push(parse_reg(left, &ctx.registers, i)?);
                result.push(parse_reg(right, &ctx.registers, i)?)
            }
            "IMM" => {
                result.push(ctx.opcodes.imm);
                result.push(parse_reg(left, &ctx.registers, i)?);
                result.push(parse_num(right, i)?)
            }
            "JMP" => todo!(),
            "ADD" => {
                result.push(ctx.opcodes.add);
                result.push(parse_reg(left, &ctx.registers, i)?);
                result.push(parse_num(right, i)?)
            }

            _ => {
                return Err(InvalidInstruction::InvalidOperation {
                    operation: op.to_string(),
                    line: i,
                })
            }
        }
    }
    return Ok(result);
}
