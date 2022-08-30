use criterion::{criterion_group, criterion_main, Criterion};

pub static PASSWORD: &str = "P@ssw0rd";
pub static HASH: &str = "$argon2id$v=19$m=4096,t=192,p=4$\
                     o2y5PU86Vt+sr93N7YUGgC7AMpTKpTQCk4tNGUPZMY4$\
                     yzP/ukZRPIbZg6PvgnUUobUMbApfF9RH6NagL9L4Xr4\
                     ";
pub static SECRET_KEY: &str = "secret key that you should really store in a .env file \
                           instead of in code, but this is just an example\
                           ";
use spawn_blocking::agronautica::verify_password;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("argonautica", |b| b.iter(|| 
        async_std::task::block_on(async {
            verify_password(PASSWORD, HASH, SECRET_KEY).await.unwrap();
        })
    )
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
