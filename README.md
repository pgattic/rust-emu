
# Rust-NES

Just a hobby NES emulator.

## Purpose

I am attempting to make this emulator as hardware-accurate as possible, for two reasons:

- To see how good I actually am at modeling a real-world system in software
- I have no idea how accurate an NES emulator needs to be for it to play games well. I guess I will figure that out along the way, but I like to play things safe.

In addition, I want to learn more about real-time systems that require shared resources.

## Usage

- Compiling my example ROM (A really stupid test program):
    - Ensure you have freem's [asm6f](https://github.com/freem/asm6f) installed (available in the AUR as `asm6f` package)
    - `asm6f sample.asm`
- Running the Emulator:
    - Have the [Rust build system](https://www.rust-lang.org/tools/install) set up
    - `cargo run --release -- [ROM.nes]`

## Roadmap

- [X] Basic hardware layout
- [X] Header decoding
- Memory Bus
    - [X] Correct hardware rerouting
    - Mappers:
        - none yet! :P
- [ ] 6502 core (56 instructions, 151 opcodes, all cycle-accurate)
    - [ ] ADC
    - [ ] AND
    - [ ] ASL
    - [ ] BCC
    - [ ] BCS
    - [ ] BEQ
    - [ ] BIT
    - [ ] BMI
    - [ ] BNE
    - [ ] BPL
    - [X] BRK (1 opcode)
    - [ ] BVC
    - [ ] BVS
    - [ ] CLC
    - [ ] CLD
    - [ ] CLI
    - [ ] CLV
    - [ ] CMP
    - [ ] CPX
    - [ ] CPY
    - [ ] DEC
    - [ ] DEX
    - [ ] DEY
    - [ ] EOR
    - [ ] INC
    - [ ] INX
    - [ ] INY
    - [ ] JMP
    - [ ] JSR
    - [X] LDA (8 opcodes)
    - [X] LDX (5 opcodes)
    - [X] LDY (5 opcodes)
    - [ ] LSR
    - [X] NOP (1 opcode)
    - [ ] ORA
    - [ ] PHA
    - [ ] PHP
    - [ ] PLA
    - [ ] PLP
    - [ ] ROL
    - [ ] ROR
    - [ ] RTI
    - [ ] RTS
    - [ ] SBC
    - [ ] SEC
    - [ ] SED
    - [ ] SEI
    - [X] STA (7 opcodes)
    - [X] STX (3 opcodes)
    - [X] STY (3 opcodes)
    - [ ] TAX
    - [ ] TAY
    - [ ] TSX
    - [ ] TXA
    - [ ] TXS
    - [ ] TYA
- [ ] PPU
- [ ] APU

## Helpful Resources

- Thanks to [NESDEV](https://www.nesdev.org/wiki) for their simply amazing work on documenting the NES hardware
- mass:werk's excellent [Instruction Set](https://www.masswerk.at/6502/6502_instruction_set.html) reference
- doppelganger's [Super Mario Bros. Disassembly](https://gist.github.com/1wErt3r/4048722) for giving me some good assembly code examples to reference.

