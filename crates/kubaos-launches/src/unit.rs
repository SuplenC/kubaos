//! An app's launch, read from the name of the unit systemd runs it in. systemd's conventions for
//! desktop environments name it `app-<launcher>-<app id>@<unique>.service`, or
//! `app-<launcher>-<app id>-<unique>.scope`, with `<launcher>` optional and every `-` inside the
//! app id escaped as `\x2d`, as `systemd-escape` does.
//!
//! <https://systemd.io/DESKTOP_ENVIRONMENTS/>
//!
//! # Owns
//!
//! - Telling an app's launch from any other unit, and its app id ([`app_id`]).
//!
//! # Not here
//!
//! - Following the manager for new units: [`crate::service`].

/// The app id of the launch that unit `name` runs, or `None` for a unit that runs no app's
/// launch. An app id is the desktop entry's name without `.desktop`, which is also the window's
/// class for most apps.
pub fn app_id(name: &str) -> Option<String> {
    let rest = name.strip_prefix("app-")?;
    let escaped = if let Some(service) = rest.strip_suffix(".service") {
        service.split_once('@')?.0
    } else {
        rest.strip_suffix(".scope")?.rsplit_once('-')?.0
    };
    let id = escaped.rsplit('-').next()?;
    let id = unescape(id)?;
    (!id.is_empty()).then_some(id)
}

/// `name` with each `\xNN` turned back into its byte, or `None` when it isn't UTF-8 then.
fn unescape(name: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(name.len());
    let mut rest = name.as_bytes();
    while let Some((&byte, after)) = rest.split_first() {
        let escaped = after
            .strip_prefix(b"x")
            .and_then(|hex| hex.get(..2))
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(unescaped) if byte == b'\\' => {
                bytes.push(unescaped);
                rest = &after[3..];
            }
            _ => {
                bytes.push(byte);
                rest = after;
            }
        }
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests;
