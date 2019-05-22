use typenum::*;

use selfe_sys::*;

use crate::cap::{Badge, CapType, CopyAliasable, DirectRetype, LocalCap, Mintable, PhantomCap};

#[derive(Debug)]
pub struct Notification {}

impl CapType for Notification {}

impl PhantomCap for Notification {
    fn phantom_instance() -> Self {
        Self {}
    }
}

impl CopyAliasable for Notification {
    type CopyOutput = Self;
}

impl Mintable for Notification {}

impl DirectRetype for Notification {
    type SizeBits = U4;
    fn sel4_type_id() -> usize {
        api_object_seL4_NotificationObject as usize
    }
}

impl LocalCap<Notification> {
    /// Blocking wait on a notification
    pub fn wait(&self) -> Badge {
        let mut sender_badge: usize = 0;
        unsafe {
            seL4_Wait(self.cptr, &mut sender_badge as *mut usize);
        };
        Badge::from(sender_badge)
    }
}
