//! Keep provider keys protected on disk. On Windows: DPAPI, tied to the signed-in user.

use anyhow::{anyhow, Result};
use base64::Engine as _;

#[cfg(windows)]
pub fn protect(plain: &str) -> Result<String> {
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
    unsafe {
        let mut input = CRYPT_INTEGER_BLOB { cbData: plain.len() as u32, pbData: plain.as_ptr() as *mut u8 };
        let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
        let ok = CryptProtectData(&mut input, std::ptr::null(), std::ptr::null(), std::ptr::null(), std::ptr::null(), 0, &mut out);
        if ok == 0 {
            return Err(anyhow!("Windows could not protect the key"));
        }
        let bytes = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        windows_sys::Win32::Foundation::LocalFree(out.pbData as _);
        Ok(format!("dpapi:{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
    }
}

#[cfg(windows)]
pub fn reveal(stored: &str) -> Result<String> {
    use windows_sys::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
    let Some(b64) = stored.strip_prefix("dpapi:") else { return Ok(String::new()) };
    let mut bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
    unsafe {
        let mut input = CRYPT_INTEGER_BLOB { cbData: bytes.len() as u32, pbData: bytes.as_mut_ptr() };
        let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
        let ok = CryptUnprotectData(&mut input, std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), std::ptr::null(), 0, &mut out);
        if ok == 0 {
            return Err(anyhow!("this key was saved by another Windows user and cannot be read; enter it again"));
        }
        let plain = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        windows_sys::Win32::Foundation::LocalFree(out.pbData as _);
        Ok(String::from_utf8_lossy(&plain).to_string())
    }
}

// Development builds on other systems only (TUNAS ships for Windows).
#[cfg(not(windows))]
pub fn protect(plain: &str) -> Result<String> {
    Ok(format!("plain:{}", base64::engine::general_purpose::STANDARD.encode(plain)))
}
#[cfg(not(windows))]
pub fn reveal(stored: &str) -> Result<String> {
    let Some(b64) = stored.strip_prefix("plain:") else { return Ok(String::new()) };
    Ok(String::from_utf8(base64::engine::general_purpose::STANDARD.decode(b64)?)?)
}

pub fn hash_key(key: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(key.as_bytes()))
}

pub fn new_key(prefix: &str) -> String {
    use rand::Rng;
    let bytes: [u8; 24] = rand::thread_rng().gen();
    format!("{prefix}{}", hex::encode(bytes))
}
