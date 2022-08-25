use async_std::task;
use std::{fmt, error};
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
type ResultGood<T> = std::result::Result<T, Box< 
    dyn std::error::Error + 'static + Send + Sync 
    >
>;

type ResultBad<T> = std::result::Result<T, Box< 
    dyn std::error::Error + 'static
    >
>;

fn double_first_reluctant(vec: Vec<&str>) -> ResultBad<i32> {
    let first = vec.first().ok_or(EmptyVec)?;
    // Here we implicitly use the `ParseIntError` implementation of `From` (which
    // we defined above) in order to create a `DoubleError`.
    let parsed = first.parse::<i32>()?;

    Ok(2 * parsed)
}

fn double_first_willing(vec: Vec<&str>) -> ResultGood<i32> {
    let first = vec.first().ok_or(EmptyVec)?;
    // Here we implicitly use the `ParseIntError` implementation of `From` (which
    // we defined above) in order to create a `DoubleError`.
    let parsed = first.parse::<i32>()?;

    Ok(2 * parsed)
}


#[derive(Debug)]
struct EmptyVec;



impl fmt::Display for EmptyVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmptyVec")
    }
}

impl error::Error for EmptyVec {
}

async fn reluctant_err() -> i32 {
    let numbers = vec!["42", "93", "18"];
    match double_first_reluctant(numbers) {
        Err(error) => {
            eprintln!("encountered {}", error);
            return -1
        }
        Ok(first) => {
            task::sleep(Duration::new(2, 10)).await;
            return first
        }
    }
}
async fn willing_err() -> i32 {
    let numbers = vec!["42", "93", "18"];
    match double_first_willing(numbers) {
        Err(error) => {
            eprintln!("encountered {}", error);
            return -1
        }
        Ok(first) => {
            task::sleep(Duration::new(2, 10)).await;
            return first
        }
    }
}
fn main() {
    println!("Hello, world!");

    let jh_reluctant = task::spawn_local(reluctant());
    let jh_reluctant_err = task::spawn_local(reluctant_err());

    let jh_willing = task::spawn(willing());
    let jh_willing_err = task::spawn(willing_err());


    let f_str = task::block_on(jh_reluctant);
    let sec_str = task::block_on(jh_willing);

    let res_r_err = task::block_on(jh_reluctant_err);
    let res_w_err = task::block_on(jh_willing_err);

    println!("reluctant (no Send; spawn_local) {}", f_str);
    println!("willing (Send; spawn works) {}", sec_str);

    println!("reluctant Box<dyn Error> (no Send; spawn_local) {}", res_r_err);
    println!("reluctant Box<dyn Error + Send> (no Send; spawn) {}", res_w_err);
}
