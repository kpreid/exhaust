extern crate std;
use std::io::Cursor;

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
