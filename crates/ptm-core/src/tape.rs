// Tape: binary representation of Turing machine tapes.
// Section 0004: four tapes — input, work, control, output.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TapeAlphabet {
    Zero = 0x00,
    One = 0x01,
    Blank = 0x02,
}

impl TapeAlphabet {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Self::Zero),
            0x01 => Some(Self::One),
            0x02 => Some(Self::Blank),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkAlphabet {
    Zero = 0x00,
    One = 0x01,
    Blank = 0x02,
    X = 0x03,
    Y = 0x04,
    Z = 0x05,
}

impl WorkAlphabet {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Self::Zero),
            0x01 => Some(Self::One),
            0x02 => Some(Self::Blank),
            0x03 => Some(Self::X),
            0x04 => Some(Self::Y),
            0x05 => Some(Self::Z),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HeadMove {
    Left = 0x00,
    Right = 0x01,
    Stay = 0x02,
}

impl HeadMove {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Self::Left),
            0x01 => Some(Self::Right),
            0x02 => Some(Self::Stay),
            _ => None,
        }
    }
}

/// Tape: one-way infinite sequence of cells extending to the right.
#[derive(Debug, Clone)]
pub struct Tape {
    cells: Vec<u8>,
    pub head: usize,
}

impl Tape {
    pub fn new() -> Self {
        Tape { cells: vec![0x02], head: 0 } // initialized with blank
    }

    pub fn with_capacity(cap: usize) -> Self {
        let mut cells = vec![0x02; cap];
        cells[0] = 0x02;
        Tape { cells, head: 0 }
    }

    pub fn read(&self) -> u8 {
        *self.cells.get(self.head).unwrap_or(&0x02)
    }

    pub fn write(&mut self, symbol: u8) {
        if self.head >= self.cells.len() {
            self.cells.resize(self.head + 1, 0x02);
        }
        self.cells[self.head] = symbol;
    }

    pub fn move_head(&mut self, dir: HeadMove) {
        match dir {
            HeadMove::Left => {
                if self.head > 0 { self.head -= 1; }
            }
            HeadMove::Right => {
                self.head += 1;
                if self.head >= self.cells.len() {
                    self.cells.resize(self.head + 1, 0x02);
                }
            }
            HeadMove::Stay => {}
        }
    }

    pub fn init_from_bits(&mut self, bits: &[u8]) {
        self.cells.clear();
        for &b in bits {
            self.cells.push(if b != 0 { 0x01 } else { 0x00 });
        }
        self.cells.push(0x02); // trailing blank
        self.head = 0;
    }

    pub fn content_len(&self) -> usize {
        self.cells.len()
    }

    pub fn cells(&self) -> &[u8] {
        &self.cells
    }
}

/// TapeHead tracks position for a specific tape.
#[derive(Debug, Clone)]
pub struct TapeHead {
    pub position: usize,
    pub tape_idx: usize,
}

impl TapeHead {
    pub fn new(tape_idx: usize) -> Self {
        TapeHead { position: 0, tape_idx }
    }
}
