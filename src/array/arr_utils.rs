use crate::array::TryFromSliceError;

use crate::{uint, utils};

use super::Array;

pub type CanonArr<T, N> = crate::array::Arr<T, N>;
pub type CanonVec<T, N> = crate::array::ArrVec<CanonArr<T, N>>;
pub type CanonDeq<T, N> = crate::array::ArrDeq<CanonArr<T, N>>;

pub const fn arr_len<A: Array>() -> usize {
    match uint::to_usize::<A::Length>() {
        Some(n) => n,
        None => panic!("{}", uint::to_str::<A::Length>()),
    }
}
pub const fn arr_from_slice<A: Array>(slice: &[A::Item]) -> Result<&A, TryFromSliceError> {
    if arr_len::<A>() == slice.len() {
        Ok(unsafe { &*slice.as_ptr().cast() })
    } else {
        Err(TryFromSliceError(()))
    }
}
pub const fn arr_from_mut_slice<A: Array>(
    slice: &mut [A::Item],
) -> Result<&mut A, TryFromSliceError> {
    if arr_len::<A>() == slice.len() {
        Ok(unsafe { &mut *slice.as_mut_ptr().cast() })
    } else {
        Err(TryFromSliceError(()))
    }
}

pub const fn arr_conv<Dst: Array>(src: impl Array<Item = Dst::Item, Length = Dst::Length>) -> Dst {
    unsafe { utils::transmute(src) }
}

pub const fn phys_idx(logical: usize, cap: usize) -> usize {
    debug_assert!(logical == 0 || logical < 2 * cap);
    let phys = if logical >= cap {
        logical - cap
    } else {
        logical
    };
    debug_assert!(phys == 0 || phys < cap);
    phys
}
