use crate::cap::Badge;
use crate::userland::MessageInfo;
use sel_claw::*;

#[derive(Debug)]
pub struct VMFault {
    pub sender: Badge,
    pub program_counter: usize,
    pub address: usize,
    pub is_instruction_fault: bool,
    pub fault_status_register: usize,
}
#[derive(Debug)]
pub struct UnknownSyscall {
    pub sender: Badge,
    pub x0: usize,
    pub x1: usize,
    pub x2: usize,
    pub x3: usize,
    pub x4: usize,
    pub x5: usize,
    pub x6: usize,
    pub x7: usize,
    pub program_counter: usize,
    pub stack_pointer: usize,
    pub list_register: usize,
    pub saved_program_status_register: usize,
    pub syscall: usize,
}
#[derive(Debug)]
pub struct UserException {
    pub sender: Badge,
    pub program_counter: usize,
    pub stack_pointer: usize,
    pub saved_program_status_register: usize,
    pub number: usize,
    pub code: usize,
}
#[derive(Debug)]
pub struct NullFault {
    pub sender: Badge,
}
#[derive(Debug)]
pub struct CapFault {
    pub sender: Badge,
    pub in_receive_phase: bool,
    pub cap_address: usize,
}
/// Grab bag for faults that don't fit the regular classification
#[derive(Debug)]
pub struct UnidentifiedFault {
    pub sender: Badge,
}

#[cfg(KernelArmHypervisorSupport)]
#[derive(Debug)]
pub struct VGICMaintenanceFault {
    pub sender: Badge,
    pub index: usize,
}

#[cfg(KernelArmHypervisorSupport)]
#[derive(Debug)]
pub struct VCPUFault {
    pub sender: Badge,
    pub hyp_syndrome_register: usize,
}

#[derive(Debug)]
pub enum Fault {
    VMFault(VMFault),
    UnknownSyscall(UnknownSyscall),
    UserException(UserException),
    NullFault(NullFault),
    CapFault(CapFault),
    UnidentifiedFault(UnidentifiedFault),
    #[cfg(KernelArmHypervisorSupport)]
    VGICMaintenanceFault(VGICMaintenanceFault),
    #[cfg(KernelArmHypervisorSupport)]
    VCPUFault(VCPUFault),
}

impl Fault {
    pub fn sender(&self) -> Badge {
        match self {
            Fault::VMFault(f) => f.sender,
            Fault::UnknownSyscall(f) => f.sender,
            Fault::UserException(f) => f.sender,
            Fault::NullFault(f) => f.sender,
            Fault::CapFault(f) => f.sender,
            Fault::UnidentifiedFault(f) => f.sender,
            #[cfg(KernelArmHypervisorSupport)]
            Fault::VGICMaintenanceFault(f) => f.sender,
            #[cfg(KernelArmHypervisorSupport)]
            Fault::VCPUFault(f) => f.sender,
        }
    }
}

impl From<(MessageInfo, Badge)> for Fault {
    fn from(info_and_sender: (MessageInfo, Badge)) -> Self {
        let (info, sender) = info_and_sender;
        let buffer: &mut seL4_IPCBuffer = unsafe { &mut *seL4_GetIPCBuffer() };
        const VM_FAULT: usize = seL4_Fault_tag_seL4_Fault_VMFault as usize;
        const UNKNOWN_SYSCALL: usize = seL4_Fault_tag_seL4_Fault_UnknownSyscall as usize;
        const USER_EXCEPTION: usize = seL4_Fault_tag_seL4_Fault_UserException as usize;
        const NULL_FAULT: usize = seL4_Fault_tag_seL4_Fault_NullFault as usize;
        const CAP_FAULT: usize = seL4_Fault_tag_seL4_Fault_CapFault as usize;
        #[cfg(KernelArmHypervisorSupport)]
        const VGIC_MAINTENANCE_FAULT: usize = seL4_Fault_tag_seL4_Fault_VGICMaintenance as usize;
        #[cfg(KernelArmHypervisorSupport)]
        const VCPU_FAULT: usize = seL4_Fault_tag_seL4_Fault_VCPUFault as usize;
        match info.label() {
            NULL_FAULT => Fault::NullFault(NullFault { sender }),
            VM_FAULT => Fault::VMFault(VMFault {
                sender,
                program_counter: buffer.msg[seL4_VMFault_Msg_seL4_VMFault_IP as usize],
                address: buffer.msg[seL4_VMFault_Msg_seL4_VMFault_Addr as usize],
                is_instruction_fault: 1 == buffer.msg[seL4_VMFault_Msg_seL4_VMFault_PrefetchFault as usize],
                fault_status_register: buffer.msg[seL4_VMFault_Msg_seL4_VMFault_FSR as usize],
            }),
            UNKNOWN_SYSCALL => Fault::UnknownSyscall(UnknownSyscall {
                sender,
                x0: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_X0 as usize],
                x1: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_X1 as usize],
                x2: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_X2 as usize],
                x3: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_X3 as usize],
                x4: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_X4 as usize],
                x5: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_X5 as usize],
                x6: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_X6 as usize],
                x7: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_X7 as usize],
                program_counter: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_FaultIP as usize],
                stack_pointer: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_SP as usize],
                list_register: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_LR as usize],
                saved_program_status_register: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_SPSR as usize],
                syscall: buffer.msg[seL4_UnknownSyscall_Msg_seL4_UnknownSyscall_Syscall as usize],
            }),
            USER_EXCEPTION => Fault::UserException(UserException {
                sender,
                program_counter: buffer.msg[seL4_UserException_Msg_seL4_UserException_FaultIP as usize],
                stack_pointer: buffer.msg[seL4_UserException_Msg_seL4_UserException_SP as usize],
                saved_program_status_register: buffer.msg[seL4_UserException_Msg_seL4_UserException_SPSR as usize],
                number: buffer.msg[seL4_UserException_Msg_seL4_UserException_Number as usize],
                code: buffer.msg[seL4_UserException_Msg_seL4_UserException_Code as usize],
            }),
            CAP_FAULT => Fault::CapFault(CapFault {
                sender,
                cap_address: buffer.msg[seL4_CapFault_Msg_seL4_CapFault_Addr as usize],
                in_receive_phase: 1 == buffer.msg[seL4_CapFault_Msg_seL4_CapFault_InRecvPhase as usize],
            }),
            #[cfg(KernelArmHypervisorSupport)]
            VGIC_MAINTENANCE_FAULT => Fault::VGICMaintenanceFault(VGICMaintenanceFault {
                sender,
                index: buffer.msg[seL4_VGICMaintenance_IDX as usize],
            }),
            #[cfg(KernelArmHypervisorSupport)]
            VCPU_FAULT => Fault::VCPUFault(VCPUFault {
                sender,
                hyp_syndrome_register: buffer.msg[seL4_VCPUFault_HSR as usize],
            }),
            _ => Fault::UnidentifiedFault(UnidentifiedFault { sender }),
        }
    }
}
