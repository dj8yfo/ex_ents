use super::spawn_blocking;
pub async fn verify_password(password: &str, hash: &str, key: &str)
                        -> Result<bool, argonautica::Error>
{
    let password = password.to_string();
    let hash = hash.to_string();
    let key = key.to_string();

    spawn_blocking(move || {
        argonautica::Verifier::default()
            .with_hash(hash)
            .with_password(password)
            .with_secret_key(key)
            .verify()
    }).await
}

