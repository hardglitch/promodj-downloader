use aes_gcm::aead::Aead;
use aes_gcm::{
    aead::{arrayvec::ArrayVec, AeadInOut, Generate, Key, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngExt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

pub fn encrypt_and_write(data: &str) -> anyhow::Result<()> {
    let data = data.as_bytes();

    // 1. Encrypt data
    let key = Key::<Aes256Gcm>::generate();
    let cipher = Aes256Gcm::new(&key);

    let nonce = Nonce::generate(); // MUST be unique per message
    let mut buffer: ArrayVec<u8, 128> = ArrayVec::new(); // Note: buffer needs 16-bytes overhead for auth tag
    let _ = buffer.try_extend_from_slice(data);

    // Encrypt `buffer` in-place, replacing the plaintext contents with ciphertext
    cipher.encrypt_in_place(&nonce, b"", &mut buffer)?;

    // `buffer` now contains the message ciphertext
    assert_ne!(buffer.as_ref(), data);

    // 2. Write nonce + ciphertext to binary file
    let p = PathBuf::from("storage");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(p)?;

    file.write_all(&nonce)?;
    file.write_all(&buffer)?;
    file.flush()?;

    // 3. Write key to file
    let p = PathBuf::from("data");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(p)?;

    // 3-1. Write random thrash
    let mut trash = [0u8; 248];
    trash.iter_mut().for_each(|n| {
        let mut rng = rand::rng();
        *n = rng.random_range(0..=255);
    });
    file.write_all(&trash)?;

    // 3-2. Write payload
    file.write_all(key.as_slice())?;

    // 3-3. Write random thrash
    let mut trash = [0u8; 173];
    trash.iter_mut().for_each(|n| {
        let mut rng = rand::rng();
        *n = rng.random_range(0..=255);
    });
    file.write_all(&trash)?;
    file.flush()?;

    Ok(())
}

pub fn read_and_decrypt() -> anyhow::Result<Vec<u8>> {

    // 1. Read encrypted file
    let mut file = File::open("storage")?;
    let mut nonce_bytes = [0u8; 12];
    file.read_exact(&mut nonce_bytes)?;

    let mut encrypted_text = Vec::new();
    file.read_to_end(&mut encrypted_text)?;

    // 2. Read key file
    let mut file = File::open("data")?;
    file.seek(std::io::SeekFrom::Start(248))?;
    let mut key_bytes = [0u8; 32];
    file.read_exact(&mut key_bytes)?;

    // 3. Decrypt data
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher.decrypt(nonce, encrypted_text.as_ref())?;

    Ok(plaintext)
}