use selfe_sys::*;

use crate::arch;
use crate::cap::{page_state, CapType, DirectRetype, LocalCap, Page, PhantomCap};
use crate::error::{ErrorExt, KernelError, SeL4Error};
use crate::userland::CapRights;
use crate::vspace::{MappingError, Maps};

#[derive(Debug)]
pub struct PageTable {}

impl Maps<Page<page_state::Unmapped>> for PageTable {
    fn map_granule(
        &mut self,
        page: &LocalCap<Page<page_state::Unmapped>>,
        addr: usize,
        root: &mut LocalCap<arch::PagingRoot>,
        rights: CapRights,
        vm_attributes: arch::VMAttributes,
    ) -> Result<(), MappingError> {
        if is_aligned(addr) {
            match unsafe {
                seL4_ARM_Page_Map(
                    page.cptr,
                    root.cptr,
                    addr,
                    seL4_CapRights_t::from(rights),
                    vm_attributes,
                )
            }
            .as_result()
            {
                Ok(_) => Ok(()),
                Err(KernelError::FailedLookup) => Err(MappingError::Overflow),
                Err(e) => Err(MappingError::PageMapFailure(SeL4Error::PageMap(e))),
            }
        } else {
            Err(MappingError::AddrNotPageAligned)
        }
    }
}

impl CapType for PageTable {}
impl PhantomCap for PageTable {
    fn phantom_instance() -> Self {
        PageTable {}
    }
}

impl DirectRetype for PageTable {
    type SizeBits = super::super::PageTableBits;
    fn sel4_type_id() -> usize {
        _object_seL4_ARM_PageTableObject as usize
    }
}

fn is_aligned(addr: usize) -> bool {
    addr % (1 << 12) == 0
}
