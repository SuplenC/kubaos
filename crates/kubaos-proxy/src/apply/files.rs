//! An OpenVPN VPN's certificates and keys as files, for as long as NetworkManager's plugin reads
//! them. The plugin takes them as files alone, and copies each, read as the user who owns the
//! connection, into a file of its own only root reads, before it starts OpenVPN. So they are
//! written from the keyring at each `on`, into the session's folder (`$XDG_RUNTIME_DIR`, in memory
//! and gone at logout) where only the user reads them, and removed once NetworkManager has the VPN
//! up or has given up on it.
//!
//! # Owns
//!
//! - [`write`], [`remove`] and the folder they are in ([`Paths::vpn_files`]).

use std::fs;
use std::io;
use std::os::unix::fs::DirBuilderExt as _;
use std::os::unix::fs::PermissionsExt as _;

use super::StepError;
use crate::openvpn;
use crate::openvpn::Credentials;
use crate::paths::Paths;
use crate::profile::write_atomic;
use crate::secrets::Secret;
use crate::tunnel::Files;

/// Each file by its name, and what it is called in the folder.
const NAMES: [(&str, &str); 6] = [
    ("ca", "ca.pem"),
    ("cert", "cert.pem"),
    ("key", "key.pem"),
    ("tls-auth", "tls-auth.key"),
    ("tls-crypt", "tls-crypt.key"),
    ("tls-crypt-v2", "tls-crypt-v2.key"),
];

/// Writes `config`'s CA and the certificates and keys among `credentials` into a folder only the
/// user reads, in place of any left there, and says where each is.
///
/// # Errors
///
/// The folder or a file couldn't be written; what was written is removed again.
pub(super) fn write(paths: &Paths, config: &openvpn::Config, credentials: &Credentials) -> Result<Files, StepError> {
    remove(paths);
    let dir = paths.vpn_files();
    let made = fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .and_then(|()| fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)));
    made.map_err(|err| StepError::File(dir.clone(), err))?;
    let mut files = Vec::new();
    for (name, file) in NAMES {
        let text = match name {
            "ca" => config.ca.as_deref(),
            _ => credentials.get(name).map(Secret::expose),
        };
        let Some(text) = text else {
            continue;
        };
        let path = dir.join(file);
        if let Err(err) = write_atomic(&path, text, 0o600) {
            remove(paths);
            return Err(StepError::File(path, err));
        }
        files.push((name, path));
    }
    Ok(Files(files))
}

/// Removes the folder and every file in it; none there is fine.
pub(super) fn remove(paths: &Paths) {
    let dir = paths.vpn_files();
    if let Err(err) = fs::remove_dir_all(&dir)
        && err.kind() != io::ErrorKind::NotFound
    {
        log::warn!("couldn't remove {}: {err}", dir.display());
    }
}
