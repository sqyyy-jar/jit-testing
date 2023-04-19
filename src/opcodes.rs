pub const SMALLOP: u16 = 0x0000;
pub const NOOP: u16 = 0x0000;
pub const MOVE: u16 = 0x0100;
pub const MEMLOAD: u16 = 0x0200;
pub const MEMSTORE: u16 = 0x0300;
pub const RETURN: u16 = 0x0400;
pub const ADD: u16 = 0x0500;
pub const SUB: u16 = 0x0600;
pub const MUL: u16 = 0x0700;
pub const IMUL: u16 = 0x0800;
pub const DIV: u16 = 0x0900;
pub const IDIV: u16 = 0x0a00;
pub const REM: u16 = 0x0b00;
pub const IREM: u16 = 0x0c00;
pub const PRINT: u16 = 0x0d00;
pub const HALT: u16 = 0x0e00;

pub const LOAD: u16 = 0x1000;
pub const ILOAD: u16 = 0x2000;
pub const JUMP: u16 = 0xb000;
pub const JUMPZ: u16 = 0xc000;
pub const JUMPNZ: u16 = 0xd000;
pub const CALL: u16 = 0xe000;

pub fn __noop() -> u16 {
    NOOP
}

pub fn __move(dst: u16, src: u16) -> u16 {
    MOVE | dst & 7 | (src & 7) << 3
}

pub fn __memload(dst: u16, src: u16) -> u16 {
    MEMLOAD | dst & 7 | (src & 7) << 3
}

pub fn __memstore(dst: u16, src: u16) -> u16 {
    MEMSTORE | dst & 7 | (src & 7) << 3
}

pub fn __return() -> u16 {
    RETURN
}

pub fn __add(dst: u16, src: u16) -> u16 {
    ADD | dst & 7 | (src & 7) << 3
}

pub fn __sub(dst: u16, src: u16) -> u16 {
    SUB | dst & 7 | (src & 7) << 3
}

pub fn __mul(dst: u16, src: u16) -> u16 {
    MUL | dst & 7 | (src & 7) << 3
}

pub fn __imul(dst: u16, src: u16) -> u16 {
    IMUL | dst & 7 | (src & 7) << 3
}

pub fn __div(dst: u16, src: u16) -> u16 {
    DIV | dst & 7 | (src & 7) << 3
}

pub fn __idiv(dst: u16, src: u16) -> u16 {
    IDIV | dst & 7 | (src & 7) << 3
}

pub fn __rem(dst: u16, src: u16) -> u16 {
    REM | dst & 7 | (src & 7) << 3
}

pub fn __irem(dst: u16, src: u16) -> u16 {
    IREM | dst & 7 | (src & 7) << 3
}

pub fn __print(src: u16) -> u16 {
    PRINT | src & 7
}

pub fn __halt() -> u16 {
    HALT
}

pub fn __load(dst: u16, value: u16) -> u16 {
    LOAD | dst & 7 | (value & 0x1ff) << 3
}

pub fn __iload(dst: u16, value: i16) -> u16 {
    ILOAD | dst & 7 | (value as u16 & 0x1ff) << 3
}

pub fn __jump(offset: i16) -> u16 {
    JUMP | offset as u16 & 0xfff
}

pub fn __jumpz(dst: u16, value: i16) -> u16 {
    JUMPZ | dst & 7 | (value as u16 & 0x1ff) << 3
}

pub fn __jumpnz(dst: u16, value: i16) -> u16 {
    JUMPNZ | dst & 7 | (value as u16 & 0x1ff) << 3
}

pub fn __call(index: u16) -> u16 {
    CALL | index & 0xfff
}
