use sel_claw::*;

use crate::arch::PagingRoot;
use crate::cap::{page_state, CapType, LocalCap, Page, PhantomCap};
use crate::error::{KernelError, SeL4Error};
use crate::userland::CapRights;
use crate::vspace::{MappingError, Maps};

#[derive(Debug)]
pub struct PageTable {}

impl CapType for PageTable {}
impl PhantomCap for PageTable {
    fn phantom_instance() -> Self {
        PageTable {}
    }
}
impl Maps<Page<page_state::Unmapped>> for PageTable {
    fn map_granule(
        &mut self,
        page: &LocalCap<Page<page_state::Unmapped>>,
        addr: usize,
        root: &mut LocalCap<PagingRoot>,
        rights: CapRights,
        vm_attributes: seL4_ARM_VMAttributes,
    ) -> Result<(), MappingError> {
        if is_aligned(addr) {
            match unsafe { page.unchecked_page_map(addr, root, rights, vm_attributes) } {
                Ok(_) => Ok(()),
                Err(SeL4Error::PageMap(KernelError::FailedLookup)) => Err(MappingError::Overflow),
                Err(e) => Err(MappingError::PageMapFailure(e)),
            }
        } else {
            Err(MappingError::AddrNotPageAligned)
        }
    }
}

fn is_aligned(addr: usize) -> bool {
    use typenum::Unsigned;
    addr % crate::arch::PageBytes::USIZE == 0
}
