mod common;
use common::make_test;

#[test]
fn all_adda() {
    make_test(
        "
    ORG $20
    ; #Data
    ADDA #1
    ADDA #16
    ADDA #$10
    ADDA #%00010000

    ; Adr
    ADDA 16
    ADDA $10
    ADDA %00010000

    ; n,SP
    ADDA 10,SP
    ADDA $0A,SP
    ADDA %00001010,SP

    ; n,X
    ADDA 10,X
    ADDA $0A,X
    ADDA %00001010,X

    ; n,Y
    ADDA 10,Y
    ADDA $0A,Y
    ADDA %00001010,Y

    ORG $FF
    FCB $20",
    )
}

#[test]
fn all_suba() {
    make_test(
        "
    ORG $20
    ; #Data
    SUBA #1
    SUBA #16
    SUBA #$10
    SUBA #%00010000

    ; Adr
    SUBA 16
    SUBA $10
    SUBA %00010000

    ; n,SP
    SUBA 10,SP
    SUBA $0A,SP
    SUBA %00001010,SP

    ; n,X
    SUBA 10,X
    SUBA $0A,X
    SUBA %00001010,X

    ; n,Y
    SUBA 10,Y
    SUBA $0A,Y
    SUBA %00001010,Y

    ORG $FF
    FCB $20",
    )
}

#[test]
fn read_ascii_from_input() {
    make_test(
        "
DIPSWITCH:      EQU $FC
SEGMENT7:       EQU $FB
SEG_ERROR:      EQU %01111001
                ORG $0
seg_table:      FCB 63,6,91,79,102,109,125,7,127,111
                ORG $20
DisplaySegE:    LDX Segmentkod      ; X <- Segmentkod
DisplaySegE_1:  LDA DIPSWITCH       ; A <- M(DIPSWITCH)
                CMPA #10            ; A<10 ?
                BLO DisplaySegE_2 	; YES
                LDA #SEG_ERROR		; NO
                JMP DisplaySegE_3
DisplaySegE_2:  LDA A,X             ; A <- M(A+X)
DisplaySegE_3:  STA SEGMENT7        ; M(SEGMENT7) <- A
                JMP DisplaySegE_1
                ORG SEGMENT7
Segmentkod:     FCB $0
                FCB $0
                ORG $FF
                FCB $20
",
    )
}
