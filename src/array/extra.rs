//! Items related to implementation details of arrays.

use crate::Uint;
use crate::array::*;

pub struct IntoIter<T, N: Uint> {
    pub(crate) deq: ArrDeq<T, N>,
}

#[derive(Debug, Copy, Clone)]
pub struct TryFromSliceError(pub(crate) ());

/// Maps `ArrApi<A>` to `A`.
///
/// This can be used to get the "internal" inner types of type alias wrappers for `ArrApi`,
/// such as [`Arr`]. Note that while these are not private as they can be accessed using
/// traits (see below), they implement nothing except [`Copy`], [`Clone`] and [`Array`].
///
/// This type alias is not magic; it is literally defined as
/// ```
/// # use generic_uint::array::{Array, ArrApi};
/// trait _ArrApi { type Inner; }
/// impl<A: Array> _ArrApi for ArrApi<A> { type Inner = A; }
/// type ArrApiInner<ArrApi> = <ArrApi as _ArrApi>::Inner;
/// ```
pub type ArrApiInner<ArrApi> = <ArrApi as crate::internals::_ArrApi>::Inner;
