use core::marker::PhantomData;
use core::ops::{Add, Sub};

use sel_claw::*;

use typenum::operator_aliases::Diff;
use typenum::*;

use crate::cap::{role, CNodeRole, Cap, CapType, ChildCap, LocalCap};
use crate::error::SeL4Error;
use crate::userland::CapRights;

// | (| guard | radix |) |
// We're always using U12, so we have 2^12 slots
