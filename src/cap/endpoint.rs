use typenum::*;

use sel_claw::*;

use crate::cap::{CapType, CopyAliasable, DirectRetype, Mintable, PhantomCap};

#[derive(Debug)]
pub struct Endpoint {}

impl CapType for Endpoint {}

impl PhantomCap for Endpoint {
    fn phantom_instance() -> Self {
        Self {}
    }
}

impl CopyAliasable for Endpoint {
    type CopyOutput = Self;
}
impl<'a> From<&'a Endpoint> for Endpoint {
    fn from(_val: &'a Endpoint) -> Self {
        PhantomCap::phantom_instance()
    }
}

impl Mintable for Endpoint {}

impl DirectRetype for Endpoint {
    type SizeBits = U4;
    fn sel4_type_id() -> usize {
        api_object_seL4_EndpointObject as usize
    }
}
