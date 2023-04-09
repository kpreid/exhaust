use core::hash;
use core::iter;

use crate::Exhaust;

impl<H> Exhaust for hash::BuildHasherDefault<H> {
    type Iter = iter::Once<hash::BuildHasherDefault<H>>;

    fn exhaust() -> Self::Iter {
        // `BuildHasherDefault` is a ZST; it has exactly one value.
        iter::once(hash::BuildHasherDefault::default())
    }
}
