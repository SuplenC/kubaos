//! The first-boot setup app goes onto the target from the live system that
//! installs it (docs/INSTALLER.md, "First boot"): the same `kubaos` and
//! `dotinstall-gui`, plus the files in `installer/firstboot/` that make it
//! open at the first login and give it a way to root.

use std::path::{Path, PathBuf};

use super::{Install, R};
use crate::engine::runner::Cmd;

const HELPER: &str = include_str!("../../../../../installer/firstboot/setup-helper");
const WRAPPER: &str = include_str!("../../../../../installer/firstboot/kubaos-setup");
const POLICY: &str = include_str!("../../../../../installer/firstboot/org.kubaos.setup.policy");
const UNIT: &str = include_str!("../../../../../installer/firstboot/kubaos-setup.service");
const DESKTOP: &str = include_str!("../../../../../installer/firstboot/kubaos-setup.desktop");

/// gnome-keyring's plaintext format: a keyring without a password is
/// stored unencrypted and is never locked, so nothing ever asks for it.
pub const KEYRING: &str = "[keyring]\ndisplay-name=Default keyring\nctime=0\nmtime=0\nlock-on-idle=false\nlock-after=false\n";

pub const UNIT_PATH: &str = "/usr/lib/systemd/user/kubaos-setup.service";
/// `systemctl --global enable`, done by hand: no systemd in the chroot.
pub const UNIT_WANTS: &str =
    "/etc/systemd/user/graphical-session.target.wants/kubaos-setup.service";

impl Install<'_> {
    /// The user's default keyring, with an empty password (Omarchy does the
    /// same). The login is tty autologin right after the LUKS passphrase, so
    /// nothing would ever unlock a password-protected login keyring, and the
    /// first app to want a secret (Bitwarden, Chrome, gh, secret-tool) would
    /// pop "Choose password for new keyring" on the first login. The disk
    /// encryption is the boundary instead; docs/INSTALLER.md has the
    /// trade-off. Left alone if the home already has keyrings.
    pub(super) fn install_keyring(&mut self, home: &str, owner: &str) -> R<()> {
        let dir = format!("{home}/.local/share/keyrings");
        if self.exists_on_target(&format!("{dir}/default")) {
            return Ok(());
        }
        self.mkdir(&self.t(&dir))?;
        self.write(&format!("{dir}/Default_keyring.keyring"), KEYRING, 0o600)?;
        self.write(&format!("{dir}/default"), "Default_keyring\n", 0o644)?;
        self.sh(Cmd::new("chmod").args(["700", &self.ts(&dir)]))?;
        self.sh(Cmd::new("chown").args([
            owner,
            &self.ts(&format!("{home}/.local")),
            &self.ts(&format!("{home}/.local/share")),
        ]))?;
        self.sh(Cmd::new("chown").args(["-R", owner, &self.ts(&dir)]))?;
        Ok(())
    }

    /// Copies the binaries and writes the setup files. Without a
    /// `dotinstall-gui` next to us (a CLI-only live system, the engine
    /// tests) setup isn't installed, with a warning: the install itself is
    /// fine, the TUXEDO bits then need `system/30-hardware.sh` by hand.
    pub(super) fn install_setup(&mut self) -> R<()> {
        let dir: Option<PathBuf> = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf));
        let (cli, gui) = match dir {
            Some(d) => (d.join("kubaos"), d.join("dotinstall-gui")),
            None => (PathBuf::new(), PathBuf::new()),
        };
        let found = self.r.exists(&cli) && self.r.exists(&gui);
        if !self.dry() && !found {
            self.warn(format!(
                "no dotinstall-gui next to {}: first-boot setup isn't installed. \
                 Run system/30-hardware.sh by hand for the TUXEDO drivers.",
                cli.display()
            ));
            return Ok(());
        }
        for (src, dst) in [(&cli, "/usr/bin/kubaos"), (&gui, "/usr/bin/dotinstall-gui")] {
            let cmd =
                Cmd::new("install").args(["-m", "0755", &src.display().to_string(), &self.ts(dst)]);
            self.sh(cmd)?;
        }
        self.write("/usr/lib/kubaos/setup-helper", HELPER, 0o755)?;
        self.write("/usr/bin/kubaos-setup", WRAPPER, 0o755)?;
        self.write(
            "/usr/share/polkit-1/actions/org.kubaos.setup.policy",
            POLICY,
            0o644,
        )?;
        self.write(UNIT_PATH, UNIT, 0o644)?;
        self.write(
            "/usr/share/applications/kubaos-setup.desktop",
            DESKTOP,
            0o644,
        )?;
        let wants = self.t(UNIT_WANTS);
        if let Some(d) = wants.parent() {
            self.mkdir(d)?;
        }
        self.sh(Cmd::new("ln").args(["-sfn", UNIT_PATH, &wants.display().to_string()]))?;
        self.info("first-boot setup opens at the first login (kubaos-setup any time)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_files_say_what_the_code_expects() {
        assert!(HELPER.starts_with("#!/bin/bash"));
        assert!(HELPER.contains("SYSTEM=/usr/share/kubaos/system\n"));
        assert!(HELPER.contains(&format!("{}/system", crate::setup::SYSTEM_KUBAOS)));
        assert!(POLICY.contains("/usr/lib/kubaos/setup-helper"));
        assert!(POLICY.contains(crate::setup::HELPER));
        assert!(UNIT.contains("ExecStart=/usr/bin/dotinstall-gui --setup --first-boot"));
        assert!(UNIT.contains("ConditionPathExists=!%S/kubaos/setup-done"));
        assert!(WRAPPER.contains("dotinstall-gui --setup"));
        assert!(DESKTOP.contains("Exec=kubaos-setup"));
        assert!(KEYRING.starts_with("[keyring]\n") && KEYRING.contains("lock-on-idle=false"));
    }
}
