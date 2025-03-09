extern crate std;
use std::io::Cursor;
use std::sync;

use exhaust::Exhaust;

#[test]
fn impl_cursor() {
    assert_eq!(
        Cursor::<[u8; 2]>::exhaust()
            .take(7)
            .map(|cursor| { (cursor.position(), cursor.into_inner()) })
            .collect::<Vec<_>>(),
        vec![
            (0, [0, 0]),
            (1, [0, 0]),
            (2, [0, 0]),
            (0, [0, 1]),
            (1, [0, 1]),
            (2, [0, 1]),
            (0, [0, 2]),
            // .. and more
        ]
    );
    assert_eq!(Cursor::<[u8; 2]>::exhaust().count(), 256 * 256 * 3);
}

#[test]
fn impl_sync_once_lock() {
    assert_eq!(
        sync::OnceLock::<bool>::exhaust()
            .map(|cell| cell.get().copied())
            .collect::<Vec<_>>(),
        vec![None, Some(false), Some(true)],
    );
}
