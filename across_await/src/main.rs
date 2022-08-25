use async_std::task;
use std::sync::Arc;
use std::{rc::Rc, time::Duration};
async fn reluctant() -> String {
    let string = Rc::new("ref-counted string".to_string());

    task::sleep(Duration::new(2, 10)).await;

    format!("your splendid spl_string: {}", string)

}
async fn willing() -> String {
    let string = Arc::new("atomically ref-counted string".to_string());

    task::sleep(Duration::new(2, 10)).await;

    format!("your splendid spl_string: {}", string)

}
fn main() {
    println!("Hello, world!");

    let jh_reluctant = task::spawn_local(reluctant());

    let jh_correct = task::spawn(willing());

    let f_str = task::block_on(jh_reluctant);
    let sec_str = task::block_on(jh_correct);

    println!("reluctant (no Send; spawn_local) {}", f_str);
    println!("willing (Send; spawn works) {}", sec_str);
}
