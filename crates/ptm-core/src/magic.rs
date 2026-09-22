// Magic numbers for all artifact types per Section 0000.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Magic {
    /// PTM-IR files
    PTMI = 0x50544D49,
    /// Turing Machine transition tables
    TMTT = 0x544D5454,
    /// Quipper circuit artifacts
    QUIP = 0x51554950,
    /// JCL manifests
    JCLM = 0x4A434C4D,
    /// Audit records
    AUDT = 0x41554454,
    /// Test vectors
    TEST = 0x54455354,
    /// Token streams
    TOKN = 0x544F4B4E,
    /// CFG binary
    CFGB = 0x43464742,
    /// Complexity report
    CMPL = 0x434D504C,
    /// Validation report
    VALR = 0x56414C52,
    /// Validation protocol
    VALP = 0x56414C50,
    /// Diagnostic log
    DIAG = 0x44494147,
    /// Error code table
    ERRT = 0x45525254,
    /// Security boundary
    SECB = 0x53454342,
    /// Security audit log
    SEAU = 0x53454155,
    /// Snapshot
    SNAP = 0x534E4150,
    /// OpenQASM translation
    OQSM = 0x4F51534D,
}

impl Magic {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0x50544D49 => Some(Self::PTMI),
            0x544D5454 => Some(Self::TMTT),
            0x51554950 => Some(Self::QUIP),
            0x4A434C4D => Some(Self::JCLM),
            0x41554454 => Some(Self::AUDT),
            0x54455354 => Some(Self::TEST),
            0x544F4B4E => Some(Self::TOKN),
            0x43464742 => Some(Self::CFGB),
            0x434D504C => Some(Self::CMPL),
            0x56414C52 => Some(Self::VALR),
            0x56414C50 => Some(Self::VALP),
            0x44494147 => Some(Self::DIAG),
            0x45525254 => Some(Self::ERRT),
            0x53454342 => Some(Self::SECB),
            0x53454155 => Some(Self::SEAU),
            0x534E4150 => Some(Self::SNAP),
            0x4F51534D => Some(Self::OQSM),
            _ => None,
        }
    }

    pub fn as_bytes(self) -> [u8; 4] {
        (self as u32).to_be_bytes()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PTMI => "PTMI",
            Self::TMTT => "TMTT",
            Self::QUIP => "QUIP",
            Self::JCLM => "JCLM",
            Self::AUDT => "AUDT",
            Self::TEST => "TEST",
            Self::TOKN => "TOKN",
            Self::CFGB => "CFGB",
            Self::CMPL => "CMPL",
            Self::VALR => "VALR",
            Self::VALP => "VALP",
            Self::DIAG => "DIAG",
            Self::ERRT => "ERRT",
            Self::SECB => "SECB",
            Self::SEAU => "SEAU",
            Self::SNAP => "SNAP",
            Self::OQSM => "OQSM",
        }
    }
}
