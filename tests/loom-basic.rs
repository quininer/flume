#![cfg(feature = "loom")]

use std::time::{Instant, Duration};
use flume::*;
use loom::thread;

#[test]
fn send_recv() {
    loom::model(|| {
        let (tx, rx) = unbounded();
        for i in 0..10 { tx.send(i).unwrap(); }
        for i in 0..10 { assert_eq!(rx.try_recv().unwrap(), i); }
        assert!(rx.try_recv().is_err());
    });
}

#[test]
fn iter() {
    loom::model(|| {
        let (tx, rx) = unbounded();
        for i in 0..10 { tx.send(i).unwrap(); }
        drop(tx);
        assert_eq!(rx.iter().sum::<u32>(), (0..10).sum());
    });
}

#[test]
fn try_iter() {
    loom::model(|| {
        let (tx, rx) = unbounded();
        for i in 0..3 { tx.send(i).unwrap(); }
        assert_eq!(rx.try_iter().sum::<u32>(), (0..3).sum());
    });
}

#[test]
fn iter_threaded() {
    loom::model(|| {
        let (tx, rx) = unbounded();
        for i in 0..3 {
            let tx = tx.clone();
            thread::spawn(move || tx.send(i).unwrap());
        }
        drop(tx);
        assert_eq!(rx.iter().sum::<u32>(), (0..3).sum());
    });
}

#[test]
fn recv_timeout() {
    loom::model(|| {
        let (tx, rx) = unbounded();

        let dur = Duration::from_millis(350);
        let then = Instant::now();
        assert!(rx.recv_timeout(dur).is_err());
        let now = Instant::now();

        let max_error = Duration::from_millis(1);
        assert!(now.duration_since(then) < dur.checked_add(max_error).unwrap());
        assert!(now.duration_since(then) > dur.checked_sub(max_error).unwrap());

        tx.send(42).unwrap();
        assert_eq!(rx.recv_timeout(dur), Ok(42));
        assert!(Instant::now().duration_since(now) < max_error);
    });
}

#[test]
fn recv_deadline() {
    loom::model(|| {
        let (tx, rx) = unbounded();

        let dur = Duration::from_millis(350);
        let then = Instant::now();
        assert!(rx.recv_deadline(then.checked_add(dur).unwrap()).is_err());
        let now = Instant::now();

        let max_error = Duration::from_millis(1);
        assert!(now.duration_since(then) < dur.checked_add(max_error).unwrap());
        assert!(now.duration_since(then) > dur.checked_sub(max_error).unwrap());

        tx.send(42).unwrap();
        assert_eq!(rx.recv_deadline(now.checked_add(dur).unwrap()), Ok(42));
        assert!(Instant::now().duration_since(now) < max_error);
    });
}

#[test]
fn disconnect_tx() {
    loom::model(|| {
        let (tx, rx) = unbounded::<()>();
        drop(tx);
        assert!(rx.recv().is_err());
    });
}

#[test]
fn disconnect_rx() {
    loom::model(|| {
        let (tx, rx) = unbounded();
        drop(rx);
        assert!(tx.send(0).is_err());
    });
}

#[test]
fn try_send() {
    loom::model(|| {
        let (tx, rx) = bounded(5);

        for i in 0..5 {
            tx.try_send(i).unwrap();
        }

        assert!(tx.try_send(42).is_err());

        assert_eq!(rx.recv(), Ok(0));

        assert_eq!(tx.try_send(42), Ok(()));

        assert_eq!(rx.recv(), Ok(1));
        drop(rx);

        assert!(tx.try_send(42).is_err());
    });
}

#[test]
fn send_bounded() {
    loom::model(|| {
        let (tx, rx) = bounded(5);

        let mut ts = Vec::new();
        for _ in 0..3 {
            let tx = tx.clone();
            ts.push(thread::spawn(move || {
                for i in 0..3 {
                    tx.send(i).unwrap();
                }
            }));
        }

        drop(tx);

        assert_eq!(rx.iter().sum::<u32>(), (0..3).sum::<u32>() * 3);

        for t in ts {
            t.join().unwrap();
        }

        assert!(rx.recv().is_err());
    });
}

#[test]
fn hydra() {
    loom::model(|| {
        let thread_num = 3;
        let msg_num = 3;

        let (main_tx, main_rx) = unbounded::<()>();

        let mut txs = Vec::new();
        for _ in 0..thread_num {
            let main_tx = main_tx.clone();
            let (tx, rx) = unbounded();
            txs.push(tx);

            thread::spawn(move || {
                for msg in rx.iter() {
                    main_tx.send(msg).unwrap();
                }
            });
        }

        drop(main_tx);

        for _ in 0..3 {
            for tx in &txs {
                for _ in 0..msg_num {
                    tx.send(Default::default()).unwrap();
                }
            }

            for _ in 0..thread_num {
                for _ in 0..msg_num {
                    main_rx.recv().unwrap();
                }
            }
        }

        drop(txs);
        assert!(main_rx.recv().is_err());
    });
}

#[test]
fn robin() {
    loom::model(|| {
        let thread_num = 3;
        let msg_num = 3;

        let (mut main_tx, main_rx) = bounded::<()>(1);

        for _ in 0..thread_num {
            let (mut tx, rx) = bounded(3);
            std::mem::swap(&mut tx, &mut main_tx);

            thread::spawn(move || {
                for msg in rx.iter() {
                    tx.send(msg).unwrap();
                }
            });
        }

        for _ in 0..3 {
            let main_tx = main_tx.clone();
            thread::spawn(move || {
                for _ in 0..msg_num {
                    main_tx.send(Default::default()).unwrap();
                }
            });

            for _ in 0..msg_num {
                main_rx.recv().unwrap();
            }
        }
    });
}
