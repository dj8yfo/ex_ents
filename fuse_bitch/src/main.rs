use std::time::Duration;

use async_std::prelude::*;
use async_std::stream;

fn main() {
    let task = async_std::task::spawn(async {
        let mut s = stream::once(1).fuse();
        assert_eq!(s.next().await, Some(1));
        assert_eq!(s.next().await, None);
        assert_eq!(s.next().await, None);
        println!("start");
        async_std::task::sleep(Duration::new(2, 0)).await;
        println!("finish");
        std::thread::current().name().map(|val| val.to_owned())

    });

    let thr_name = async_std::task::block_on(task);
    println!("aync block thread {:?}", thr_name);
    println!("current thread name {:?} ", std::thread::current().name())
}
