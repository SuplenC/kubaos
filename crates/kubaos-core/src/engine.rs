//! The install engine (owner: engine). Emits `Event`s through `emit` in the
//! order docs/INSTALL-INTERNALS.md describes.
//!
//! Layout: `machine` (probe), `validate`, `disk` (partition math), `files`
//! (text of the files written to the target), `unpack` (squashfs or
//! tar.zst), `runner` (the one way commands run), `install` (the steps).

pub mod disk;
pub mod files;
pub mod install;
pub mod machine;
pub mod runner;
pub mod unpack;
pub mod validate;

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::{Error, Event, InstallConfig, Manifest, Probe, Secrets, StepId};
use runner::{DryRunner, Runner, SystemRunner};

pub const DEFAULT_TARGET: &str = "/mnt/kubaos";

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Print what each step would do, touch nothing.
    pub dry_run: bool,
    /// Mount point for the target while installing (default `/mnt/kubaos`).
    pub target_root: Option<String>,
    /// The live system's log; `finish` copies it to the target's
    /// `/var/log/kubaos/install.log`.
    pub log_file: Option<PathBuf>,
}

/// Probe with the payload in the default place (`/run/kubaos/payload`).
pub fn probe() -> Result<Probe, Error> {
    probe_with(Path::new(machine::DEFAULT_PAYLOAD_DIR))
}

pub fn probe_with(payload_dir: &Path) -> Result<Probe, Error> {
    machine::probe(payload_dir).map_err(Error::Config)
}

/// Every problem, one per line, in `Error::Config`.
pub fn validate(config: &InstallConfig, probe: &Probe) -> Result<(), Error> {
    let p = validate::problems(config, probe);
    if p.is_empty() {
        Ok(())
    } else {
        Err(Error::Config(p.join("\n")))
    }
}

pub fn validate_secrets(config: &InstallConfig, secrets: &Secrets) -> Result<(), Error> {
    let p = validate::secret_problems(config, secrets);
    if p.is_empty() {
        Ok(())
    } else {
        Err(Error::Config(p.join("\n")))
    }
}

pub fn plan(config: &InstallConfig) -> Vec<StepId> {
    StepId::ALL
        .into_iter()
        .filter(|s| *s != StepId::Encrypt || config.disk.encrypt)
        .collect()
}

pub fn run(
    config: &InstallConfig,
    secrets: &Secrets,
    opts: &RunOptions,
    emit: &mut dyn FnMut(Event),
) -> Result<(), Error> {
    if opts.dry_run {
        let mut r = DryRunner::new();
        run_with(config, secrets, opts, &mut r, emit)
    } else {
        let mut r = SystemRunner::new();
        run_with(config, secrets, opts, &mut r, emit)
    }
}

/// A manifest for dry runs on a machine without the payload.
fn placeholder_manifest(config: &InstallConfig) -> Manifest {
    Manifest {
        version: "<version>".into(),
        codename: None,
        arch: machine::arch(),
        built_at: String::new(),
        commit: "<commit>".into(),
        username: config.user.username.clone(),
        hostname: config.host.hostname.clone(),
        rootfs: "rootfs.sqfs".into(),
        rootfs_bytes: 0,
        installed_bytes: 12 * disk::GIB,
        sha256: String::new(),
        kernel: "<kernel>".into(),
        home_generation: "/nix/store/<hash>-home-manager-generation".into(),
        wallpaper_sets: Vec::new(),
    }
}

/// `run` with a given runner (tests pass a `DryRunner` with canned output).
pub fn run_with(
    config: &InstallConfig,
    secrets: &Secrets,
    opts: &RunOptions,
    r: &mut dyn Runner,
    emit: &mut dyn FnMut(Event),
) -> Result<(), Error> {
    let started = Instant::now();
    let steps = plan(config);
    emit(Event::Plan {
        steps: steps.iter().map(|s| s.info()).collect(),
    });

    let manifest = match machine::load_manifest(Path::new(&config.payload.dir)) {
        Ok(m) => Ok(m),
        Err(_) if r.is_dry_run() => Ok(placeholder_manifest(config)),
        Err(e) => Err(e),
    };
    let disk = machine::read_lsblk().ok().and_then(|l| {
        machine::disks_from_lsblk(&l, &Default::default(), true)
            .into_iter()
            .find(|d| d.path == config.disk.device)
    });
    let target = PathBuf::from(
        opts.target_root
            .clone()
            .unwrap_or_else(|| DEFAULT_TARGET.into()),
    );

    let manifest = match manifest {
        Ok(m) => m,
        Err(e) => {
            emit(Event::StepStarted {
                id: StepId::Preflight,
            });
            emit(Event::Failed {
                id: StepId::Preflight,
                error: e.clone(),
                hint: Some("No payload: is the install stick complete?".into()),
            });
            return Err(Error::Step {
                step: StepId::Preflight,
                message: e,
                hint: None,
            });
        }
    };

    let mut inst = install::Install::new(
        config,
        secrets,
        manifest,
        disk,
        r,
        emit,
        target,
        opts.log_file.clone(),
    );
    for id in steps {
        (inst.emit)(Event::StepStarted { id });
        let t = Instant::now();
        match inst.step(id) {
            Ok(()) => (inst.emit)(Event::StepFinished {
                id,
                elapsed_ms: t.elapsed().as_millis() as u64,
            }),
            Err(e) => {
                inst.cleanup();
                (inst.emit)(Event::Failed {
                    id,
                    error: e.message.clone(),
                    hint: e.hint.clone(),
                });
                return Err(Error::Step {
                    step: id,
                    message: e.message,
                    hint: e.hint,
                });
            }
        }
    }
    (inst.emit)(Event::Done {
        elapsed_ms: started.elapsed().as_millis() as u64,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use runner::DryRunner;

    fn config(payload: &Path) -> InstallConfig {
        InstallConfig::from_toml(&format!(
            r#"
            [locale]
            lang = "en_US.UTF-8"
            keyboard_layout = "us"
            timezone = "Europe/Rome"
            [host]
            hostname = "vm"
            [user]
            username = "kuba"
            full_name = "Kuba Example"
            [disk]
            device = "/dev/vdz"
            target = {{ mode = "whole" }}
            [git]
            name = "Kuba"
            email = "j@example.com"
            [wifi]
            ssid = "Home 5G"
            [payload]
            dir = "{}"
            "#,
            payload.display()
        ))
        .unwrap()
    }

    fn secrets() -> Secrets {
        Secrets {
            luks_passphrase: Some("correct horse battery".into()),
            user_password: "hunter22".into(),
            wifi_psk: Some("wifi-secret-psk".into()),
        }
    }

    fn payload() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kubaos-engine-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let m = serde_json::json!({
            "version": "1", "arch": "x86_64", "built_at": "2026-10-16T10:00:00Z",
            "commit": "abc", "username": "kuba", "hostname": "vm",
            "rootfs": "rootfs.tar.zst", "rootfs_bytes": 1, "installed_bytes": 10737418240u64,
            "sha256": "00", "kernel": "6.17.0-5-generic",
            "home_generation": "/nix/store/abc-home-manager-generation"
        });
        // Tests run in parallel and share this dir: replace the file in one
        // step, so nobody reads it half-written.
        let tmp = dir.join(format!("manifest.json.{:?}", std::thread::current().id()));
        std::fs::write(&tmp, m.to_string()).unwrap();
        std::fs::rename(&tmp, dir.join("manifest.json")).unwrap();
        dir
    }

    fn dry(r: &mut DryRunner, fail: bool) -> (Result<(), Error>, Vec<Event>) {
        let p = payload();
        let cfg = config(&p);
        let mut events = Vec::new();
        if fail {
            r.fail_on.push("grub-install".into());
        }
        let opts = RunOptions {
            dry_run: true,
            target_root: Some("/mnt/t".into()),
            log_file: None,
        };
        let res = run_with(&cfg, &secrets(), &opts, r, &mut |e| events.push(e));
        (res, events)
    }

    #[test]
    fn dry_run_follows_the_event_contract() {
        let mut r = DryRunner::new();
        let (res, events) = dry(&mut r, false);
        res.unwrap();
        let Event::Plan { steps } = &events[0] else {
            panic!("first event must be plan")
        };
        assert_eq!(steps.len(), 11);
        // started -> (progress|log)* -> finished, per step, in plan order.
        let mut expect = steps.iter().map(|s| s.id);
        let mut current = None;
        for e in &events[1..events.len() - 1] {
            match e {
                Event::StepStarted { id } => {
                    assert!(current.is_none());
                    assert_eq!(Some(*id), expect.next());
                    current = Some(*id);
                }
                Event::Progress { id, fraction, .. } => {
                    assert_eq!(Some(*id), current);
                    assert!(fraction.is_none_or(|f| (0.0..=1.0).contains(&f)));
                }
                Event::Log { id, .. } => assert_eq!(Some(*id), current),
                Event::StepFinished { id, .. } => {
                    assert_eq!(Some(*id), current);
                    current = None;
                }
                other => panic!("unexpected {other:?}"),
            }
        }
        assert!(expect.next().is_none());
        assert!(matches!(events.last(), Some(Event::Done { .. })));

        // Secrets never show up anywhere; the Wi-Fi psk only in its keyfile.
        let all = format!("{events:?}{:?}", r.actions);
        assert!(!all.contains("correct horse"));
        assert!(!all.contains("hunter22"));
        assert!(!format!("{events:?}").contains("wifi-secret-psk"));
        let wifi: Vec<_> = r
            .actions
            .iter()
            .filter(|a| format!("{a:?}").contains("wifi-secret-psk"))
            .collect();
        assert_eq!(wifi.len(), 1);
        let runner::Action::Write {
            path,
            mode,
            contents,
        } = wifi[0]
        else {
            panic!()
        };
        assert_eq!(
            path,
            Path::new("/mnt/t/etc/NetworkManager/system-connections/Home_5G.nmconnection")
        );
        assert_eq!(*mode, 0o600);
        assert!(contents.contains("ssid=Home 5G\n"));

        let cmds = r.commands();
        let has = |s: &str| cmds.iter().any(|c| c.contains(s));
        assert!(has("sgdisk --new=1:2048:2099199 --typecode=1:ef00"));
        assert!(has(
            "cryptsetup luksFormat --batch-mode --type luks2 --pbkdf argon2id"
        ));
        assert!(has(
            "mount -o noatime,compress=zstd:1,subvol=@home /dev/mapper/cryptroot /mnt/t/home"
        ));
        assert!(has("tar --zstd"));
        assert!(has("grub-install --target=x86_64-efi"));
        assert!(has(
            "snapper --no-dbus -c root create --cleanup-algorithm number --description factory"
        ));
        assert!(has(
            "setpriv --reuid=1000 --regid=1000 --init-groups -- env -i HOME=/home/kuba"
        ));
        assert!(r.files.contains_key(Path::new("/mnt/t/etc/crypttab")));
        assert!(r.files[Path::new("/mnt/t/etc/fstab")].contains("subvol=@nix"));
        // The identity goes where config/git/config includes it, owned by
        // the user all the way down from ~/.config (home-manager writes there).
        let id = Path::new("/mnt/t/home/kuba/.config/git-identity/identity.gitconfig");
        assert!(r.files[id].contains("email = \"j@example.com\""));
        assert!(
            !r.files
                .contains_key(Path::new("/mnt/t/home/kuba/.gitconfig"))
        );
        assert!(has(
            "chown 1000:1000 /mnt/t/home/kuba/.config /mnt/t/home/kuba/.config/git-identity \
             /mnt/t/home/kuba/.config/git-identity/identity.gitconfig"
        ));
        // Finish leaves nothing mounted or open.
        assert_eq!(cmds.last().unwrap(), "cryptsetup close cryptroot");
    }

    /// The payload's user-flake template and lock, as install/image ships them.
    fn user_flake_payload(r: &mut DryRunner) {
        r.files.insert(
            "/mnt/t/usr/share/kubaos/user-flake/flake.nix.in".into(),
            "homeConfigurations.\"@USER@@@HOST@\" = kubaos.lib.mkHome { \
             pkgs = nixpkgs.legacyPackages.@SYSTEM@; username = \"@USER@\"; };\n"
                .into(),
        );
        r.files.insert(
            "/mnt/t/usr/share/kubaos/user-flake/home.nix".into(),
            "{ pkgs, ... }: { home.packages = [ ]; }\n".into(),
        );
        r.files.insert(
            "/mnt/t/usr/share/kubaos-image/user-flake.lock".into(),
            "{\"version\": 7}\n".into(),
        );
    }

    #[test]
    fn the_user_gets_a_flake_of_their_own() {
        let p = payload();
        let mut cfg = config(&p);
        cfg.user.username = "alice".into();
        let mut r = DryRunner::new();
        user_flake_payload(&mut r);
        let opts = RunOptions {
            dry_run: true,
            target_root: Some("/mnt/t".into()),
            log_file: None,
        };
        run_with(&cfg, &secrets(), &opts, &mut r, &mut |_| {}).unwrap();
        let dir = "/mnt/t/home/alice/.config/kubaos/home";
        let flake = &r.files[Path::new(&format!("{dir}/flake.nix"))];
        assert_eq!(
            flake,
            "homeConfigurations.\"alice@vm\" = kubaos.lib.mkHome { \
             pkgs = nixpkgs.legacyPackages.x86_64-linux; username = \"alice\"; };\n"
        );
        assert_eq!(
            r.files[Path::new(&format!("{dir}/home.nix"))],
            "{ pkgs, ... }: { home.packages = [ ]; }\n"
        );
        assert_eq!(
            r.files[Path::new(&format!("{dir}/flake.lock"))],
            "{\"version\": 7}\n"
        );
        let cmds = r.commands();
        assert!(
            cmds.iter()
                .any(|c| c == "chown -R 1000:1000 /mnt/t/home/alice/.config/kubaos"),
            "{cmds:#?}"
        );
        // KubaOS's checkout stays in /usr/share/kubaos: nothing copies it to
        // the home or commits there.
        assert!(
            !cmds.iter().any(|c| c.contains("Projects/dotfiles")),
            "{cmds:#?}"
        );
        assert!(!cmds.iter().any(|c| c.contains("git ")), "{cmds:#?}");
    }

    #[test]
    fn another_username_builds_its_own_generation() {
        let activation = |cmds: &[String]| {
            cmds.iter()
                .find(|c| c.contains("/activate"))
                .cloned()
                .expect("an activation")
        };

        // The payload's user: its prebuilt generation, nothing to build.
        let mut r = DryRunner::new();
        dry(&mut r, false).0.unwrap();
        let cmds = r.commands();
        assert!(!cmds.iter().any(|c| c.contains("homeConfigurations")));
        assert!(activation(&cmds).contains("/nix/store/abc-home-manager-generation/activate"));

        // Anyone else: Nix builds <user>@<host> from their user flake,
        // offline, as that user, and that's what activates.
        let p = payload();
        let mut cfg = config(&p);
        cfg.user.username = "alice".into();
        let mut r = DryRunner::new();
        let opts = RunOptions {
            dry_run: true,
            target_root: Some("/mnt/t".into()),
            log_file: None,
        };
        run_with(&cfg, &secrets(), &opts, &mut r, &mut |_| {}).unwrap();
        let cmds = r.commands();
        let build = cmds
            .iter()
            .find(|c| c.contains("homeConfigurations"))
            .expect("a build");
        for want in [
            "--reuid=1000",
            "USER=alice",
            "HOME=/home/alice",
            "cd \"$HOME/.config/kubaos/home\"",
            "nix --offline",
            "alice@vm",
            "--out-link /tmp/kubaos-home-generation",
        ] {
            assert!(build.contains(want), "{want} not in {build}");
        }
        let act = activation(&cmds);
        assert!(
            act.contains("/tmp/kubaos-home-generation/activate"),
            "{act}"
        );
        let i = |c: &String| cmds.iter().position(|x| x == c).unwrap();
        assert!(i(build) < i(&act));
    }

    #[test]
    fn system_files_that_name_the_user_are_rendered_for_them() {
        // system/50-hyprland.sh and 40-apps.sh rendered these for the build
        // user; the installed system needs them for the real one.
        let p = payload();
        let mut cfg = config(&p);
        cfg.user.username = "alice".into();
        let mut r = DryRunner::new();
        let tpl = "/mnt/t/usr/share/kubaos/system/files";
        r.files.insert(
            format!("{tpl}/getty/autologin.conf.in").into(),
            "ExecStart=-/sbin/agetty --autologin @USER@ %I $TERM\n".into(),
        );
        r.files.insert(
            format!("{tpl}/chrome/dot-chrome-theme.path.in").into(),
            "PathChanged=@HOME@/.local/state/theme/colors.json\n".into(),
        );
        // Installed by the image: the getty drop-in and the path unit, not
        // the service (as if 40-apps.sh had skipped it).
        let autologin = "/mnt/t/etc/systemd/system/getty@tty1.service.d/autologin.conf";
        let path_unit = "/mnt/t/etc/systemd/system/dot-chrome-theme.path";
        r.files
            .insert(autologin.into(), "--autologin kuba\n".into());
        r.files.insert(path_unit.into(), "/home/kuba\n".into());
        let opts = RunOptions {
            dry_run: true,
            target_root: Some("/mnt/t".into()),
            log_file: None,
        };
        run_with(&cfg, &secrets(), &opts, &mut r, &mut |_| {}).unwrap();
        assert_eq!(
            r.files[Path::new(autologin)],
            "ExecStart=-/sbin/agetty --autologin alice %I $TERM\n"
        );
        assert_eq!(
            r.files[Path::new(path_unit)],
            "PathChanged=/home/alice/.local/state/theme/colors.json\n"
        );
        assert!(!r.files.contains_key(Path::new(
            "/mnt/t/etc/systemd/system/dot-chrome-theme.service"
        )));
    }

    #[test]
    fn the_install_records_its_release_for_kubaos_update() {
        let p = payload();
        let cfg = config(&p);
        let mut r = DryRunner::new();
        user_flake_payload(&mut r);
        r.files.insert("/mnt/t/usr/share/kubaos/VERSION".into(), "0.1.0\n".into());
        r.files.insert(
            "/mnt/t/usr/share/kubaos/.git/refs/heads/release".into(),
            "1a2b3c4d5e6f\n".into(),
        );
        let chroot = "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
                      LANG=C.UTF-8 DEBIAN_FRONTEND=noninteractive chroot /mnt/t find";
        r.responses.push((
            format!("{chroot} /usr/share/kubaos/migrations/system"),
            "1790000000-fix-apt.sh\nREADME.md\n".into(),
        ));
        r.responses.push((
            format!("{chroot} /usr/share/kubaos/migrations/user"),
            "1790000001-move-config.sh\n".into(),
        ));
        let opts = RunOptions {
            dry_run: true,
            target_root: Some("/mnt/t".into()),
            log_file: None,
        };

        run_with(&cfg, &secrets(), &opts, &mut r, &mut |_| {}).unwrap();

        assert_eq!(r.files[Path::new("/mnt/t/etc/kubaos/channel")], "edge\n");
        let release = &r.files[Path::new("/mnt/t/var/lib/kubaos/release")];
        assert!(release.starts_with("version=0.1.0\n"), "{release}");
        assert!(release.contains("commit=1a2b3c4d5e6f\nchannel=edge\n"), "{release}");
        assert!(r.files.contains_key(Path::new(
            "/mnt/t/var/lib/kubaos/migrations/1790000000-fix-apt.sh"
        )));
        assert!(!r.files.contains_key(Path::new("/mnt/t/var/lib/kubaos/migrations/README.md")));
        assert!(r.files.contains_key(Path::new(
            "/mnt/t/home/kuba/.local/state/kubaos/migrations/1790000001-move-config.sh"
        )));
        let cmds = r.commands();
        assert!(
            cmds.iter().any(|c| c == "chown -R 1000:1000 /mnt/t/home/kuba/.local"),
            "{cmds:#?}"
        );
    }

    #[test]
    fn failure_cleans_up_in_reverse_and_ends_with_failed() {
        let mut r = DryRunner::new();
        let (res, events) = dry(&mut r, true);
        assert!(matches!(
            res,
            Err(Error::Step {
                step: StepId::Bootloader,
                ..
            })
        ));
        let Some(Event::Failed { id, hint, .. }) = events.last() else {
            panic!("failed must be last")
        };
        assert_eq!(*id, StepId::Bootloader);
        assert!(hint.is_some());

        let cmds = r.commands();
        let pos = cmds
            .iter()
            .position(|c| c.contains("grub-install"))
            .unwrap();
        let after: Vec<&String> = cmds[pos + 1..].iter().collect();
        let umounts: Vec<&str> = after
            .iter()
            .filter_map(|c| c.strip_prefix("umount "))
            .collect();
        // Newest mount first: the chroot binds, then /boot/efi, /boot, the
        // subvolumes, and / last.
        assert_eq!(umounts.first(), Some(&"/mnt/t/run"));
        assert_eq!(umounts.last(), Some(&"/mnt/t"));
        let efi = umounts
            .iter()
            .position(|m| *m == "/mnt/t/boot/efi")
            .unwrap();
        let boot = umounts.iter().position(|m| *m == "/mnt/t/boot").unwrap();
        assert!(efi < boot);
        assert_eq!(after.last().unwrap().as_str(), "cryptsetup close cryptroot");
    }
}
