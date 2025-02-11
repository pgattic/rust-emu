
# Rust-nes

Just a hobby NES emulator. Trying to be cycle-accurate.

## Purpose

I am attempting to make this emulator as hardware-accurate as possible, for two reasons:

- To see how good I actually am at modelling a real world system in code
- I have no idea how accurate an NES emulator needs to be for it to play games well. I guess I will figure that out along the way, but I like to play things safe.

## Roadmap

- [X] Basic hardware layout
- [ ] Header decoding
- [ ] Memory Bus
    - [X] Correct hardware mapping
    - Mappers:
        - none yet! :P
- [ ] 6502 core (56 instructions)
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
    - [X] BRK
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
    - [ ] LDA
    - [ ] LDX
    - [ ] LDY
    - [ ] LSR
    - [X] NOP
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
    - [ ] STA
    - [ ] STX
    - [ ] STY
    - [ ] TAX
    - [ ] TAY
    - [ ] TSX
    - [ ] TXA
    - [ ] TXS
    - [ ] TYA
- [ ] PPU
- [ ] APU

