extern crate std;

use std::io::Cursor;

use exhaust::Exhaust;

mod helper;
use helper::{check_double_exact, check_indexable};

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

mod impl_sync {
    use super::*;
    use std::sync;

    #[test]
    fn impl_once_lock() {
        assert_eq!(
            sync::OnceLock::<bool>::exhaust()
                .map(|cell| cell.get().copied())
                .collect::<Vec<_>>(),
            vec![None, Some(false), Some(true)],
        );
    }

    #[test]
    fn impl_recv_timeout_error() {
        check_double_exact(vec![
            sync::mpsc::RecvTimeoutError::Timeout,
            sync::mpsc::RecvTimeoutError::Disconnected,
        ]);
        check_indexable::<sync::mpsc::RecvTimeoutError>();
    }

    #[test]
    fn impl_try_recv_error() {
        check_double_exact(vec![
            sync::mpsc::TryRecvError::Empty,
            sync::mpsc::TryRecvError::Disconnected,
        ]);
        check_indexable::<sync::mpsc::TryRecvError>();
    }
}
