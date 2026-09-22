// ptm-core: Binary foundation for Polynomial Turing Machine.
// Every format is defined at the bit and byte level per BLRD-PTM-2026-001.

pub mod header;
pub mod crc32;
pub mod magic;
pub mod tape;
pub mod state;

pub use header::FileHeader;
pub use magic::Magic;
pub use tape::{Tape, TapeAlphabet, TapeHead};
pub use state::PTMState;
