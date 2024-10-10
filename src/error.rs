#[derive(Debug, PartialEq)]
pub enum SeL4Error {
    UntypedRetype(KernelError),
    TCBConfigure(KernelError),
    PageTableMap(KernelError),
    PageUpperDirectoryMap(KernelError),
    PageDirectoryMap(KernelError),
    ASIDControlMakePool(KernelError),
    ASIDPoolAssign(KernelError),
    PageGetAddress(KernelError),
    PageMap(KernelError),
    PageUnmap(KernelError),
    CNodeCopy(KernelError),
    CNodeMint(KernelError),
    CNodeSaveCaller(KernelError),
    TCBWriteRegisters(KernelError),
    TCBReadRegisters(KernelError),
    TCBSetPriority(KernelError),
    TCBResume(KernelError),
    CNodeMutate(KernelError),
    CNodeMove(KernelError),
    CNodeDelete(KernelError),
    IRQControlGet(KernelError),
    IRQHandlerSetNotification(KernelError),
    IRQHandlerAck(KernelError),
    GetPageAddr(KernelError),
    PageCleanInvalidateData(KernelError),
    CNodeRevoke(KernelError),
    VCPUInjectIRQ(KernelError),
    VCPUReadRegisters(KernelError),
    VCPUWriteRegisters(KernelError),
    VCPUBindTcb(KernelError),
    TCBBindNotification(KernelError),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KernelError {
    InvalidArgument,
    InvalidCapability,
    IllegalOperation,
    RangeError,
    AlignmentError,
    FailedLookup,
    TruncatedMessage,
    DeleteFirst,
    RevokeFirst,
    NotEnoughMemory,
    /// A kernel error code that was not recognized
    UnknownError(u32),
}

pub trait ErrorExt {
    fn as_result(self) -> Result<(), KernelError>;
}

impl ErrorExt for sel_claw::seL4_Error {
    fn as_result(self) -> Result<(), KernelError> {
        match self {
            sel_claw::seL4_Error_seL4_NoError => Ok(()),
            sel_claw::seL4_Error_seL4_InvalidArgument => Err(KernelError::InvalidArgument),
            sel_claw::seL4_Error_seL4_InvalidCapability => Err(KernelError::InvalidCapability),
            sel_claw::seL4_Error_seL4_IllegalOperation => Err(KernelError::IllegalOperation),
            sel_claw::seL4_Error_seL4_RangeError => Err(KernelError::RangeError),
            sel_claw::seL4_Error_seL4_AlignmentError => Err(KernelError::AlignmentError),
            sel_claw::seL4_Error_seL4_FailedLookup => Err(KernelError::FailedLookup),
            sel_claw::seL4_Error_seL4_TruncatedMessage => Err(KernelError::TruncatedMessage),
            sel_claw::seL4_Error_seL4_DeleteFirst => Err(KernelError::DeleteFirst),
            sel_claw::seL4_Error_seL4_RevokeFirst => Err(KernelError::RevokeFirst),
            sel_claw::seL4_Error_seL4_NotEnoughMemory => Err(KernelError::NotEnoughMemory),
            unknown => Err(KernelError::UnknownError(unknown)),
        }
    }
}
