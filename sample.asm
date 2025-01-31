  .db $4E, $45, $53, $1A  ; "NES" magic number
  .db $02                 ; PRG ROM: 2 x 16 KiB = 32 KiB (NROM-256)
  .db $01                 ; CHR ROM: 1 x 8 KiB
  .db $00                 ; Flags 6: Mapper 0, horizontal mirroring
  .db $00                 ; Flags 7: iNES 1.0 format (not NES 2.0)
  .db $00                 ; Flags 8: No PRG RAM
  .db $00                 ; Flags 9: No CHR RAM
  .db $00                 ; Flags 10: NTSC
  .db $00, $00, $00, $00, $00  ; Unused padding bytes

  .org $8000  ; PRG ROM starts at $8000 in NROM

Start:
  lda #69    ; Load immediate value into A
  sta $00    ; Store in zero-page RAM
  brk        ; Break (used to test CPU exceptions)

  .align $fffc
  .dw Start, 0  ; Set reset vector at $FFFC and clear NMI

