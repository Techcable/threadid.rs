extern crate core;

use std::any::Any;
use std::collections::HashSet;
use std::sync::{Barrier, Mutex};

use crossbeam_utils::thread;
use threadid::{IThreadId, LiveThreadId, StdThreadId, UniqueThreadId};

#[test]
fn death_reuse() {
    let seen_unique_ids = Mutex::new(HashSet::<UniqueThreadId>::new());
    let seen_std_ids = Mutex::new(HashSet::<StdThreadId>::new());
    let seen_live_ids = Mutex::new(HashSet::<LiveThreadId>::new());
    fn add_new<T: IThreadId>(lock: &Mutex<HashSet<T>>) {
        let id = threadid::current();
        assert!(lock.lock().unwrap().insert(id), "unexpected reuse of {id:?}");
    }
    fn propagate_panic(payload: Box<dyn Any + Send>) -> ! {
        if let Some(payload) = payload.downcast_ref::<&'static str>() {
            panic!("panic in subthread: {payload}");
        } else if let Some(payload) = payload.downcast_ref::<String>() {
            panic!("panic in subthread: {payload}");
        } else {
            panic!("Unexpected panic payload {payload:?}")
        }
    }
    add_new(&seen_unique_ids);
    add_new(&seen_std_ids);
    add_new(&seen_live_ids);
    {
        let start = Barrier::new(2);
        let end = Barrier::new(2);
        thread::scope(|scope| {
            let t1 = scope.spawn(|_scope| {
                start.wait();
                add_new(&seen_unique_ids);
                add_new(&seen_std_ids);
                add_new(&seen_live_ids);
                end.wait();
            });
            let t2 = scope.spawn(|_scope| {
                start.wait();
                add_new(&seen_unique_ids);
                add_new(&seen_std_ids);
                add_new(&seen_live_ids);
                end.wait();
            });
            t1.join().unwrap_or_else(|payload| propagate_panic(payload));
            t2.join().unwrap_or_else(|payload| propagate_panic(payload));
        })
        .unwrap();
    }
    // above threads have died
    thread::scope(|scope| {
        scope
            .spawn(|_scope| {
                add_new(&seen_unique_ids);
                add_new(&seen_std_ids);
                let live_id: LiveThreadId = threadid::current();
                assert!(seen_live_ids.lock().unwrap().contains(&live_id));
            })
            .join()
            .unwrap_or_else(|payload| propagate_panic(payload));
    })
    .unwrap();
}
