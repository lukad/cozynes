use crate::{
    cpu::{AddressingMode, Cpu},
    instruction::{Size, INSTRUCTIONS},
    mem::Mem,
};

pub fn trace(cpu: &Cpu) -> String {
    let opcode = &INSTRUCTIONS[cpu.read_byte(cpu.pc) as usize];

    let begin = cpu.pc;
    let mut hex_dump = vec![opcode.opcode];

    let (mem_addr, stored_value) = match opcode.mode {
        AddressingMode::None => (0, 0),
        _ => {
            let addr = cpu.get_operand_address(&opcode.mode, begin + 1);
            (addr, cpu.read_byte(addr))
        }
    };

    let tmp = match opcode.bytes {
        Size::One => match opcode.opcode {
            0x0a | 0x2a | 0x4a | 0x6a => "A ".to_string(),
            _ => String::from(""),
        },
        Size::Two => {
            let address = cpu.read_byte(begin + 1);
            hex_dump.push(address);

            match opcode.mode {
                AddressingMode::Immediate => format!("#${:02x}", stored_value),
                AddressingMode::ZeroPage => format!("${:02x} = {:02x}", mem_addr, stored_value),
                AddressingMode::ZeroPageX => format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::ZeroPageY => format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::IndirectX => format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    address,
                    (address.wrapping_add(cpu.x)),
                    mem_addr,
                    stored_value
                ),
                AddressingMode::IndirectY => format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    address,
                    (mem_addr.wrapping_sub(cpu.y as u16)),
                    mem_addr,
                    stored_value
                ),
                AddressingMode::None | AddressingMode::Relative => {
                    // assuming local jumps: BNE, BVS, etc....
                    let address: usize =
                        (begin as usize + 2).wrapping_add((address as i8) as usize);
                    format!("${:04x}", address)
                }
                _ => panic!("unexpected addressing mode {:?}", opcode.mode),
            }
        }
        Size::Three => {
            let address_lo = cpu.read_byte(begin + 1);
            let address_hi = cpu.read_byte(begin + 2);
            hex_dump.push(address_lo);
            hex_dump.push(address_hi);

            let address = cpu.read_word(begin + 1);

            match opcode.mode {
                AddressingMode::None => format!("${:04x}", address),
                AddressingMode::Absolute => format!("${:04x} = {:02x}", mem_addr, stored_value),
                AddressingMode::AbsoluteX => format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::AbsoluteY => format!(
                    "${:04x},Y @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::Indirect => {
                    //jmp indirect
                    let jmp_addr = if address & 0x00FF == 0x00FF {
                        let lo = cpu.read_byte(address);
                        let hi = cpu.read_byte(address & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        cpu.read_word(address)
                    };

                    // let jmp_addr = cpu.read_byte_u16(address);
                    format!("(${:04x}) = {:04x}", address, jmp_addr)
                }
                _ => panic!(
                    "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                    opcode.mode, opcode.opcode
                ),
            }
        }
    };

    let hex_str = hex_dump
        .iter()
        .map(|z| format!("{:02x}", z))
        .collect::<Vec<String>>()
        .join(" ");
    let asm_str = format!(
        "{:04x}  {:8} {: >4} {}",
        begin, hex_str, opcode.mnemonic, tmp
    )
    .trim()
    .to_string();

    format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
        asm_str,
        cpu.a,
        cpu.x,
        cpu.y,
        u8::from(&cpu.status),
        cpu.sp,
    )
    .to_ascii_uppercase()
}
