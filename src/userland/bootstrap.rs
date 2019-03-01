use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::Sub;
use crate::pow::Pow;
use crate::userland::cap::UnassignedPageDirectory;
use crate::userland::{
    memory_kind, role, ASIDControl, ASIDPool, AssignedPageDirectory, CNode, Cap, IRQControl,
    LocalCap, MappedPage, MappedPageTable, SeL4Error, ThreadControlBlock, UnmappedPageTable,
    Untyped,
};
use sel4_sys::*;
use typenum::operator_aliases::{Diff, Prod, Sub1};
use typenum::{Unsigned, B1, U0, U1024, U12, U19};

// The root CNode radix is 19. Conservatively set aside 2^12 (the default root
// cnode size) for system use. TODO: verify at build time that this is enough /
// compute a better number
type RootCNodeSize = Pow<U19>;
type SystemProvidedCapCount = Pow<U12>;
type RootCNodeAvailableSlots = Diff<RootCNodeSize, SystemProvidedCapCount>;

// of random things in the bootinfo.
// TODO: ideally, this should only be callable once in the process. Is that possible?
pub fn root_cnode(
    bootinfo: &'static seL4_BootInfo,
) -> LocalCap<CNode<RootCNodeAvailableSlots, role::Local>> {
    Cap {
        cptr: seL4_CapInitThreadCNode as usize,
        _role: PhantomData,
        cap_data: CNode {
            radix: 19,
            next_free_slot: bootinfo.empty.start,
            _free_slots: PhantomData,
            _role: PhantomData,
        },
    }
}

pub mod paging {
    use crate::pow::Pow;
    use typenum::operator_aliases::{Diff, Prod};
    use typenum::{U1, U1024, U12, U16, U20, U24, U26, U6, U8, U9};

    pub type BaseASIDPoolFreeSlots = U1024;

    // Arm32 address structure
    pub type PageDirectoryBits = U12;
    pub type PageTableBits = U8;
    pub type PageBits = U12; // 4kb

    pub type LargePageBits = U16; // 64 KB
    pub type SectionBits = U20; // 1 MB
    pub type SuperSectionBits = U24; // 16 MB

    // PageTableBits + PageBits
    pub type PageTableTotalBits = U20;

    pub type CodePageTableBits = U6;
    pub type CodePageTableCount = Pow<CodePageTableBits>; // 64 page tables == 64 mb
    pub type CodePageCount = Prod<CodePageTableCount, BasePageTableFreeSlots>; // 2^14
    pub type TotalCodeSizeBits = U26;

    // 0xe00000000 and up is reserved to the kernel; this translates to the last
    // 2^9 (512) pagedir entries.
    pub type BasePageDirFreeSlots = Diff<Pow<PageDirectoryBits>, Pow<U9>>;

    pub type BasePageTableFreeSlots = Pow<PageTableBits>;

    // The root task has a stack size configurable by the fel4.toml
    // in the `[fel4.executable]` table's `root-task-stack-bytes` property.
    // This configuration is turned into a generated Rust type that implements
    // `typenum::Unsigned` in the `build.rs` file.
    include!(concat!(
        env!("OUT_DIR"),
        "/ROOT_TASK_STACK_PAGE_TABLE_COUNT"
    ));
    // The first N page tables are already mapped for the user image in the root
    // task. Add in the stack-reserved page tables (minimum of 1 more)
    pub type RootTaskReservedPageDirSlots = Sum<CodePageTableCount, RootTaskStackPageTableCount>;

    pub type RootTaskPageDirFreeSlots = Diff<BasePageDirFreeSlots, RootTaskReservedPageDirSlots>;

    // Useful for constant comparison to data structure size_of results
    pub type PageBytes = Pow<PageBits>;
}

pub mod address_space {
    use crate::pow::Pow;
    use typenum::operator_aliases::Sum;
    use typenum::{U0, U100, U16, U20, U29, U30, U31, U64};

    // TODO this is a magic numbers we got from inspecting the binary.
    /// 0x00010000
    pub type ProgramStart = Pow<U16>;

    /// 0xe0000000
    pub type KernelReservedStart = Sum<Pow<U31>, Sum<Pow<U30>, Pow<U29>>>;
}

/// Currently assume that a BootInfo cannot be handed to child processes
/// and thus its related structures always operate in a "Local" role.
pub struct BootInfo<ASIDPoolFreeSlots: Unsigned, PageDirFreeSlots: Unsigned> {
    pub page_directory: LocalCap<AssignedPageDirectory<PageDirFreeSlots, role::Local>>,
    pub tcb: LocalCap<ThreadControlBlock>,
    pub asid_pool: LocalCap<ASIDPool<ASIDPoolFreeSlots>>,
    pub irq_control: LocalCap<IRQControl>,
    user_image_frames_start: usize,
    user_image_frames_end: usize,
    user_image_paging_start: usize,
    user_image_paging_end: usize,
}

impl<ASIDPoolFreeSlots: Unsigned, PageDirFreeSlots: Unsigned> !Send
    for BootInfo<ASIDPoolFreeSlots, PageDirFreeSlots>
{
}

impl BootInfo<paging::BaseASIDPoolFreeSlots, paging::RootTaskPageDirFreeSlots> {
    pub fn wrap<FreeSlots: Unsigned>(
        bootinfo: &'static seL4_BootInfo,
        asid_pool_ut: LocalCap<Untyped<U12>>,
        dest_cnode: LocalCap<CNode<FreeSlots, role::Local>>,
    ) -> (Self, LocalCap<CNode<Sub1<FreeSlots>, role::Local>>)
    where
        FreeSlots: Sub<B1>,
        Sub1<FreeSlots>: Unsigned,
    {
        let asid_control = Cap::wrap_cptr(seL4_CapASIDControl as usize);
        let (asid_pool, dest_cnode): (Cap<ASIDPool<_>, _>, _) = asid_pool_ut
            .retype_asid_pool(asid_control, dest_cnode)
            .expect("retype asid pool");

        (
            BootInfo {
                page_directory: Cap {
                    cptr: seL4_CapInitThreadVSpace as usize,
                    _role: PhantomData,
                    cap_data: AssignedPageDirectory {
                        next_free_slot: paging::RootTaskReservedPageDirSlots::USIZE,
                        _free_slots: PhantomData,
                        _role: PhantomData,
                    },
                },
                tcb: Cap::wrap_cptr(seL4_CapInitThreadTCB as usize),
                asid_pool,
                irq_control: Cap {
                    cptr: seL4_CapIRQControl as usize,
                    cap_data: IRQControl {
                        known_handled: [false; 256],
                    },
                    _role: PhantomData,
                },
                user_image_frames_start: bootinfo.userImageFrames.start,
                user_image_frames_end: bootinfo.userImageFrames.end,
                user_image_paging_start: bootinfo.userImagePaging.start,
                user_image_paging_end: bootinfo.userImagePaging.end,
            },
            dest_cnode,
        )
    }
}

impl<ASIDPoolFreeSlots: Unsigned, PageDirFreeSlots: Unsigned>
    BootInfo<ASIDPoolFreeSlots, PageDirFreeSlots>
{
    pub fn user_image_page_tables_iter(
        &self,
    ) -> impl Iterator<Item = LocalCap<MappedPageTable<U0, role::Local>>> {
        // TODO break out 100
        let vaddr_iter = (0..100).map(|slot_num| slot_num << paging::PageTableTotalBits::USIZE);

        (self.user_image_paging_start..self.user_image_paging_end)
            .zip(vaddr_iter)
            .map(|(cptr, vaddr)| Cap {
                cptr,
                cap_data: MappedPageTable {
                    vaddr,
                    next_free_slot: 0,
                    _role: PhantomData,
                    _free_slots: PhantomData,
                },
                _role: PhantomData,
            })
    }

    // TODO this doesn't enforce the aliasing constraints we want at the type
    // level. This can be modeled as an array (or other sized thing) once we
    // know how big the user image is.
    pub fn user_image_pages_iter(
        &self,
    ) -> impl Iterator<Item = LocalCap<MappedPage<role::Local, memory_kind::General>>> {
        // Iterate over the entire address space's page addresses, starting at
        // ProgramStart. This is truncated to the number of actual pages in the
        // user image by zipping it with the range of frame cptrs below.
        let vaddr_iter = (address_space::ProgramStart::USIZE..core::usize::MAX)
            .step_by(1 << paging::PageBits::USIZE);

        (self.user_image_frames_start..self.user_image_frames_end)
            .zip(vaddr_iter)
            .map(|(cptr, vaddr)| Cap {
                cptr,
                cap_data: MappedPage {
                    vaddr,
                    _role: PhantomData,
                    _kind: PhantomData,
                },
                _role: PhantomData,
            })
    }

    /// Proxy to page_directory for convenience
    pub fn map_page_table(
        self,
        unmapped_page_table: LocalCap<UnmappedPageTable>,
    ) -> Result<
        (
            LocalCap<MappedPageTable<Pow<paging::PageTableBits>, role::Local>>,
            BootInfo<ASIDPoolFreeSlots, Sub1<PageDirFreeSlots>>,
        ),
        SeL4Error,
    >
    where
        PageDirFreeSlots: Sub<B1>,
        Sub1<PageDirFreeSlots>: Unsigned,
    {
        let (mapped_page_table, page_dir) =
            self.page_directory.map_page_table(unmapped_page_table)?;
        Ok((
            mapped_page_table,
            BootInfo {
                page_directory: page_dir,
                tcb: self.tcb,
                asid_pool: self.asid_pool,
                irq_control: self.irq_control,
                user_image_frames_start: self.user_image_frames_start,
                user_image_frames_end: self.user_image_frames_end,
                user_image_paging_start: self.user_image_paging_start,
                user_image_paging_end: self.user_image_paging_end,
            },
        ))
    }

    /// Convenience wrapper allowing assignment of page dirs to the
    /// ASID Pool while updating the type signature appropriately,
    /// saving the caller from having to de/re-structure BootInfo
    pub fn assign_minimal_page_dir(
        self,
        page_dir: LocalCap<UnassignedPageDirectory>,
    ) -> Result<
        (
            LocalCap<AssignedPageDirectory<paging::BasePageDirFreeSlots, role::Child>>,
            BootInfo<Sub1<ASIDPoolFreeSlots>, PageDirFreeSlots>,
        ),
        SeL4Error,
    >
    where
        ASIDPoolFreeSlots: Sub<B1>,
        Sub1<ASIDPoolFreeSlots>: Unsigned,
    {
        let (assigned_page_dir, asid_pool) = self.asid_pool.assign_minimal(page_dir)?;
        Ok((
            assigned_page_dir,
            BootInfo {
                page_directory: self.page_directory,
                tcb: self.tcb,
                asid_pool: asid_pool,
                irq_control: self.irq_control,
                user_image_frames_start: self.user_image_frames_start,
                user_image_frames_end: self.user_image_frames_end,
                user_image_paging_start: self.user_image_paging_start,
                user_image_paging_end: self.user_image_paging_end,
            },
        ))
    }
}

/// The ASID pool needs an untyped of exactly 4k.
///
/// Note that we fully consume the ASIDControl capability,
/// (which is assumed to be a singleton as well)
/// because there is a lightly-documented seL4 constraint
/// that limits us to a single ASIDPool per application.
impl LocalCap<Untyped<U12, memory_kind::General>> {
    pub fn retype_asid_pool<FreeSlots: Unsigned>(
        self,
        asid_control: LocalCap<ASIDControl>,
        dest_cnode: LocalCap<CNode<FreeSlots, role::Local>>,
    ) -> Result<
        (
            LocalCap<ASIDPool<paging::BaseASIDPoolFreeSlots>>,
            LocalCap<CNode<Sub1<FreeSlots>, role::Local>>,
        ),
        SeL4Error,
    >
    where
        FreeSlots: Sub<B1>,
        Sub1<FreeSlots>: Unsigned,
    {
        let (dest_cnode, dest_slot) = dest_cnode.consume_slot();

        let err = unsafe {
            seL4_ARM_ASIDControl_MakePool(
                asid_control.cptr,              // _service
                self.cptr,                      // untyped
                dest_slot.cptr,                 // root
                dest_slot.offset,               // index
                (8 * size_of::<usize>()) as u8, // depth
            )
        };

        if err != 0 {
            return Err(SeL4Error::UntypedRetype(err));
        }

        Ok((
            Cap {
                cptr: dest_slot.offset,
                cap_data: ASIDPool {
                    next_free_slot: 0,
                    _free_slots: PhantomData,
                },
                _role: PhantomData,
            },
            dest_cnode,
        ))
    }
}
