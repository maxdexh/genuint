//! Items related to implementation details of arrays.

use crate::{Uint, array::*};

pub struct IntoIter<T, N: Uint> {
    pub(crate) deq: ArrDeq<T, N>,
}

#[derive(Debug, Clone, Copy)]
pub struct TryFromSliceError(pub(crate) ());
