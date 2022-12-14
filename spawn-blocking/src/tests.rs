use super::*;

pub static PASSWORD: &str = "P@ssw0rd";
pub static HASH: &str = "$argon2id$v=19$m=4096,t=192,p=4$\
                     o2y5PU86Vt+sr93N7YUGgC7AMpTKpTQCk4tNGUPZMY4$\
                     yzP/ukZRPIbZg6PvgnUUobUMbApfF9RH6NagL9L4Xr4\
                     ";
pub static SECRET_KEY: &str = "secret key that you should really store in a .env file \
                           instead of in code, but this is just an example\
                           ";

use super::agronautica::verify_password;

#[test]
fn argonautica() {
    async_std::task::block_on(async {
        assert!(verify_password(PASSWORD, HASH, SECRET_KEY).await.unwrap());
    });
}

#[test]
fn many_serial() {
    async_std::task::block_on(async {
        for i in 0..1000 {
            assert_eq!(spawn_blocking(move || i).await, i);
        }
    });
}

#[test]
fn many_parallel() {
    async_std::task::block_on(async {
        let futures: Vec<_> = (0..100)
            .map(|i| (i, spawn_blocking(move || i)))
            .collect();

        for (i, f) in futures {
            assert_eq!(f.await, i);
        }
    });
}
