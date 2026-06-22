use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::env;

fn main() {
    let password = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: gen-hash <password>");
        std::process::exit(1);
    });

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("failed to hash password")
        .to_string();

    println!("{}", hash);
}
