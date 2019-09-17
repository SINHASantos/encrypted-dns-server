use crate::crypto::*;
use crate::dns::*;
use crate::dnscrypt_certs::*;
use crate::errors::*;

use libsodium_sys::*;
use std::ffi::CStr;
use std::ptr;

pub const DNSCRYPT_CLIENT_MAGIC_SIZE: usize = 8;
pub const DNSCRYPT_CLIENT_PK_SIZE: usize = 32;
pub const DNSCRYPT_CLIENT_NONCE_SIZE: usize = 12;

pub fn decrypt(
    wrapped_packet: &[u8],
    dnscrypt_encryption_params_set: &[DNSCryptEncryptionParams],
) -> Result<Vec<u8>, Error> {
    ensure!(
        wrapped_packet.len()
            >= DNSCRYPT_CLIENT_MAGIC_SIZE
                + DNSCRYPT_CLIENT_PK_SIZE
                + DNSCRYPT_CLIENT_NONCE_SIZE
                + DNS_HEADER_SIZE,
        "Short packet"
    );
    let client_magic = &wrapped_packet[..DNSCRYPT_CLIENT_MAGIC_SIZE];
    let client_pk = &wrapped_packet
        [DNSCRYPT_CLIENT_MAGIC_SIZE..DNSCRYPT_CLIENT_MAGIC_SIZE + DNSCRYPT_CLIENT_PK_SIZE];
    let client_nonce = &wrapped_packet[DNSCRYPT_CLIENT_MAGIC_SIZE + DNSCRYPT_CLIENT_PK_SIZE
        ..DNSCRYPT_CLIENT_MAGIC_SIZE + DNSCRYPT_CLIENT_PK_SIZE + DNSCRYPT_CLIENT_NONCE_SIZE];
    let encrypted_packet = &wrapped_packet
        [DNSCRYPT_CLIENT_MAGIC_SIZE + DNSCRYPT_CLIENT_PK_SIZE + DNSCRYPT_CLIENT_NONCE_SIZE..];
    let encrypted_packet_len = encrypted_packet.len();

    let dnscrypt_encryption_params = dnscrypt_encryption_params_set
        .iter()
        .find(|p| p.client_magic() == client_magic)
        .ok_or_else(|| format_err!("Client magic not found"))?;

    let mut nonce = vec![0u8; crypto_box_curve25519xchacha20poly1305_NONCEBYTES as usize];
    &mut nonce[..crypto_box_curve25519xchacha20poly1305_HALFNONCEBYTES]
        .copy_from_slice(client_nonce);
    let resolver_kp = dnscrypt_encryption_params.resolver_kp();
    let shared_secret = resolver_kp.compute_shared_key(client_pk)?;
    let packet = shared_secret.decrypt(&nonce, encrypted_packet)?;
    Ok(packet)
}
