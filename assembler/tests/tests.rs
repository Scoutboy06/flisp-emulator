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
