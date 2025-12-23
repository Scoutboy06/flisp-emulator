use srec::{ReaderError, Record};
use std::path::PathBuf;

// https://en.wikipedia.org/wiki/Motorola_S-record#Record_types

#[derive(Debug)]
pub enum S19ParseError {
    ReaderError(ReaderError),
    IOError(std::io::Error),
    UnsupportedS19RecordType(Record),
    AddrTooLarge(Record),
}

pub fn parse_s19(path: PathBuf) -> Result<[u8; 256], S19ParseError> {
    let src = std::fs::read_to_string(&path).map_err(S19ParseError::IOError)?;

    let records: Vec<_> = srec::read_records(&src).collect();

    let mut mem = [0_u8; 256];
    for record in records {
        match record {
            Ok(rec) => match rec {
                Record::S0(_s) => todo!(),
                Record::S1(s) => {
                    for (i, byte) in s.data.iter().enumerate() {
                        let adr = if s.address.0 <= 0xFF {
                            s.address.0 as u8 + i as u8
                        } else {
                            return Err(S19ParseError::AddrTooLarge(Record::S1(s)));
                        };
                        mem[adr as usize] = *byte;
                    }
                }
                Record::S2(s) => {
                    for (i, byte) in s.data.iter().enumerate() {
                        let adr = if s.address.0 <= 0xFF {
                            s.address.0 as u8 + i as u8
                        } else {
                            return Err(S19ParseError::AddrTooLarge(Record::S2(s)));
                        };
                        mem[adr as usize] = *byte;
                    }
                }
                Record::S3(s) => {
                    for (i, byte) in s.data.iter().enumerate() {
                        let adr = if s.address.0 <= 0xFF {
                            s.address.0 as u8 + i as u8
                        } else {
                            return Err(S19ParseError::AddrTooLarge(Record::S3(s)));
                        };
                        mem[adr as usize] = *byte;
                    }
                }
                Record::S7(s) => {
                    let adr = if s.0 <= 0xFF {
                        s.0 as u8
                    } else {
                        return Err(S19ParseError::AddrTooLarge(Record::S7(s)));
                    };
                    mem[0xFF] = adr;
                }
                Record::S8(s) => {
                    let adr = if s.0 <= 0xFF {
                        s.0 as u8
                    } else {
                        return Err(S19ParseError::AddrTooLarge(Record::S8(s)));
                    };
                    mem[0xFF] = adr;
                }
                Record::S9(s) => {
                    let adr = if s.0 <= 0xFF {
                        s.0 as u8
                    } else {
                        return Err(S19ParseError::AddrTooLarge(Record::S9(s)));
                    };
                    mem[0xFF] = adr;
                }
                rec => {
                    return Err(S19ParseError::UnsupportedS19RecordType(rec));
                }
            },
            Err(e) => {
                return Err(S19ParseError::ReaderError(e));
            }
        }
    }

    Ok(mem)
}
