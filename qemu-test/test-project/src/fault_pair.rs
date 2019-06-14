//! Test demonstrating that child processes can listen for faults happening on each other
use core::marker::PhantomData;

use selfe_sys::*;

use typenum::*;

use ferros::alloc::{smart_alloc, ut_buddy};
use ferros::arch::fault::Fault;
use ferros::bootstrap::UserImage;
use ferros::cap::{
    retype, retype_cnode, role, ASIDPool, CNodeRole, LocalCNode, LocalCNodeSlots, LocalCap,
    ThreadPriorityAuthority, Untyped,
};
use ferros::userland::{
    fault_or_message_channel, setup_fault_endpoint_pair, FaultOrMessage, FaultSink, ReadyProcess,
    RetypeForSetup, Sender,
};
use ferros::vspace::{ProcessCodeImageConfig, ScratchRegion, VSpace};

use super::TopLevelError;

type U33768 = Sum<U32768, U1000>;

#[ferros_test::ferros_test]
pub fn fault_pair(
    local_slots: LocalCNodeSlots<U33768>,
    local_ut: LocalCap<Untyped<U20>>,
    asid_pool: LocalCap<ASIDPool<U2>>,
    local_vspace_scratch: &mut ScratchRegion,
    root_cnode: &LocalCap<LocalCNode>,
    user_image: &UserImage<role::Local>,
    tpa: &LocalCap<ThreadPriorityAuthority>,
) -> Result<(), TopLevelError> {
    let uts = ut_buddy(local_ut);

    smart_alloc!(|slots: local_slots, ut: uts| {
        let (mischief_maker_asid, asid_pool) = asid_pool.alloc();
        let mischief_maker_root = retype(ut, slots)?;
        let mischief_maker_vspace_slots: LocalCNodeSlots<U256> = slots;
        let mischief_maker_vspace_ut: LocalCap<Untyped<U12>> = ut;
        let mischief_maker_vspace = VSpace::new(
            mischief_maker_root,
            mischief_maker_asid,
            mischief_maker_vspace_slots.weaken(),
            mischief_maker_vspace_ut.weaken(),
            ProcessCodeImageConfig::ReadOnly,
            user_image,
            root_cnode,
        )?;
        let (mischief_maker_cnode, mischief_maker_slots) = retype_cnode::<U12>(ut, slots)?;

        let (fault_handler_asid, _asid_pool) = asid_pool.alloc();
        let fault_handler_root = retype(ut, slots)?;
        let fault_handler_vspace_slots: LocalCNodeSlots<U256> = slots;
        let fault_handler_vspace_ut: LocalCap<Untyped<U12>> = ut;
        let fault_handler_vspace = VSpace::new(
            fault_handler_root,
            fault_handler_asid,
            fault_handler_vspace_slots.weaken(),
            fault_handler_vspace_ut.weaken(),
            ProcessCodeImageConfig::ReadOnly,
            user_image,
            root_cnode,
        )?;
        let (fault_handler_cnode, fault_handler_slots) = retype_cnode::<U12>(ut, slots)?;

        let (slots_source, _mischief_maker_slots) = mischief_maker_slots.alloc();
        let (slots_sink, fault_handler_slots) = fault_handler_slots.alloc();
        let (fault_source, fault_sink) =
            setup_fault_endpoint_pair(&root_cnode, ut, slots, slots_source, slots_sink)?;

        let mischief_maker_params = MischiefMakerParams { _role: PhantomData };

        let (outcome_sender_slots, _) = fault_handler_slots.alloc();
        let (fault_source_for_the_handler, outcome_sender, handler) =
            fault_or_message_channel(&root_cnode, ut, slots, outcome_sender_slots, slots)?;
        let fault_handler_params = MischiefDetectorParams::<role::Child> {
            fault_sink,
            outcome_sender,
        };

        let mischief_maker_process = ReadyProcess::new(
            &mut mischief_maker_vspace,
            mischief_maker_cnode,
            local_vspace_scratch,
            mischief_maker_proc,
            mischief_maker_params,
            ut,
            ut,
            ut,
            slots,
            tpa,
            None, // fault
        )?;
        mischief_maker_process.start()?;

        let fault_handler_process = ReadyProcess::new(
            &mut fault_handler_vspace,
            fault_handler_cnode,
            local_vspace_scratch,
            fault_handler_proc,
            fault_handler_params,
            ut,
            ut,
            ut,
            slots,
            tpa,
            None, // fault
        )?;
        fault_handler_process.start()?;
    });

    match handler.await_message()? {
        FaultOrMessage::Message(true) => Ok(()),
        _ => Err(TopLevelError::TestAssertionFailure(
            "Child process should have reported success",
        )),
    }
}

#[derive(Debug)]
pub struct MischiefMakerParams<Role: CNodeRole> {
    pub _role: PhantomData<Role>,
}

impl RetypeForSetup for MischiefMakerParams<role::Local> {
    type Output = MischiefMakerParams<role::Child>;
}

#[derive(Debug)]
pub struct MischiefDetectorParams<Role: CNodeRole> {
    pub fault_sink: FaultSink<Role>,
    pub outcome_sender: Sender<bool, Role>,
}

impl RetypeForSetup for MischiefDetectorParams<role::Local> {
    type Output = MischiefDetectorParams<role::Child>;
}

pub extern "C" fn mischief_maker_proc(_p: MischiefMakerParams<role::Local>) {
    unsafe {
        seL4_Send(
            314159, // bogus cptr to nonexistent endpoint
            seL4_MessageInfo_new(0, 0, 0, 0),
        )
    }
    debug_println!("This is after the capability fault inducing code, and should not be printed.");
}

pub extern "C" fn fault_handler_proc(p: MischiefDetectorParams<role::Local>) {
    let fault = p.fault_sink.wait_for_fault();
    p.outcome_sender
        .blocking_send(&match fault {
            Fault::CapFault(_) => true,
            _ => false,
        })
        .expect("Failed to send test outcome");
}
