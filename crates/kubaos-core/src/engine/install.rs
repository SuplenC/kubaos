//! The install itself: one method per `StepId`, all commands through the
//! `Runner`, and a record of what's mounted/open so a failure anywhere can
//! be undone in reverse order.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::disk::{self, Esp, Layout, PartTable};
use super::files::{self, FsUuids, LUKS_NAME, SUBVOLUMES};
use super::machine::{self, gib};
use super::runner::{Cmd, Output, Runner, Stream, interrupted};
use super::unpack;
use super::validate;
use crate::config::{DiskTarget, InstallConfig, Secrets, Wallpapers};
use crate::keyboard;
use crate::probe::{DiskInfo, Manifest};
use crate::{Event, Level, StepId};

mod firstboot;
mod records;

/// KubaOS's root-owned checkout in the payload's rootfs, on its `release`
/// branch. The user flake takes KubaOS from here.
pub const PAYLOAD_KUBAOS: &str = crate::setup::SYSTEM_KUBAOS;
/// The user flake's lock, generated at payload build: the user's nixpkgs at
/// KubaOS's revision, KubaOS at the payload's commit.
const USER_FLAKE_LOCK: &str = "/usr/share/kubaos-image/user-flake.lock";
/// Out-link of the generation built for a username the payload wasn't
/// built for, inside the target. /tmp: home-manager's profile roots it after.
const HOME_REBUILT: &str = "/tmp/kubaos-home-generation";
/// The target's PATH inside the chroot.
const CHROOT_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
const NIX_BIN: &str = "/nix/var/nix/profiles/default/bin";
const USER_UID: u32 = 1000;

#[derive(Debug)]
pub struct StepError {
    pub message: String,
    pub hint: Option<String>,
}

impl From<String> for StepError {
    fn from(message: String) -> Self {
        StepError {
            message,
            hint: None,
        }
    }
}

impl From<&str> for StepError {
    fn from(message: &str) -> Self {
        StepError {
            message: message.into(),
            hint: None,
        }
    }
}

type R<T> = Result<T, StepError>;

fn hint<T>(r: Result<T, String>, h: &str) -> R<T> {
    r.map_err(|message| StepError {
        message,
        hint: Some(h.into()),
    })
}

#[derive(Debug, Clone)]
struct Parts {
    esp: String,
    esp_new: bool,
    boot: String,
    /// The root partition (the LUKS container when encrypting).
    root: String,
    root_bytes: u64,
    /// What btrfs lives on: `/dev/mapper/cryptroot` or the root partition.
    fs_dev: String,
}

pub struct Install<'a> {
    pub cfg: &'a InstallConfig,
    pub secrets: &'a Secrets,
    pub manifest: Manifest,
    /// The target disk as probe saw it (dry runs plan from this).
    pub disk: Option<DiskInfo>,
    pub r: &'a mut dyn Runner,
    pub emit: &'a mut dyn FnMut(Event),
    pub target: PathBuf,
    pub top: PathBuf,
    pub log_file: Option<PathBuf>,

    step: StepId,
    parts: Option<Parts>,
    mounts: Vec<PathBuf>,
    luks_open: bool,
    daemon: Option<u32>,
    swapfile: bool,
    user: Option<(u32, u32, String)>,
    /// Numbers of the partitions a free-space install added to the disk.
    created: Vec<u32>,
}

impl<'a> Install<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cfg: &'a InstallConfig,
        secrets: &'a Secrets,
        manifest: Manifest,
        disk: Option<DiskInfo>,
        r: &'a mut dyn Runner,
        emit: &'a mut dyn FnMut(Event),
        target: PathBuf,
        log_file: Option<PathBuf>,
    ) -> Self {
        Install {
            cfg,
            secrets,
            manifest,
            disk,
            r,
            emit,
            top: PathBuf::from("/run/kubaos/btrfs-top"),
            target,
            log_file,
            step: StepId::Preflight,
            parts: None,
            mounts: vec![],
            luks_open: false,
            daemon: None,
            swapfile: false,
            user: None,
            created: vec![],
        }
    }

    // ------------------------------------------------------------------
    // plumbing

    fn dry(&self) -> bool {
        self.r.is_dry_run()
    }

    fn log(&mut self, level: Level, line: impl Into<String>) {
        (self.emit)(Event::Log {
            id: self.step,
            level,
            line: line.into(),
        });
    }

    fn info(&mut self, line: impl Into<String>) {
        self.log(Level::Info, line);
    }

    fn warn(&mut self, line: impl Into<String>) {
        self.log(Level::Warn, line);
    }

    fn progress(&mut self, fraction: f32, detail: Option<String>) {
        (self.emit)(Event::Progress {
            id: self.step,
            fraction: Some(fraction.clamp(0.0, 1.0)),
            detail,
        });
    }

    fn cmd_level(&self) -> Level {
        if self.dry() {
            Level::Info
        } else {
            Level::Debug
        }
    }

    fn sh(&mut self, cmd: Cmd) -> R<Output> {
        let lvl = self.cmd_level();
        self.log(lvl, format!("$ {}", cmd.display()));
        let out = self.r.run(&cmd)?;
        if !self.dry() {
            let lines: Vec<String> = out
                .stdout
                .lines()
                .chain(out.stderr.lines())
                .filter(|l| !l.trim().is_empty())
                .take(60)
                .map(str::to_string)
                .collect();
            for l in lines {
                self.log(Level::Debug, l);
            }
        }
        Ok(out)
    }

    /// Like `sh`, but a failure is only a warning.
    fn try_sh(&mut self, cmd: Cmd) -> bool {
        match self.sh(cmd) {
            Ok(_) => true,
            Err(e) => {
                self.warn(e.message);
                false
            }
        }
    }

    /// Run streaming every output line into the log.
    fn sh_stream(&mut self, cmd: Cmd, level: Level) -> R<()> {
        let lvl = self.cmd_level();
        self.log(lvl, format!("$ {}", cmd.display()));
        let step = self.step;
        let emit = &mut *self.emit;
        self.r.stream(&cmd, &mut |s| {
            if let Stream::Line(l) = s {
                emit(Event::Log {
                    id: step,
                    level,
                    line: l,
                });
            }
        })?;
        Ok(())
    }

    fn chroot(&self, args: &[&str]) -> Cmd {
        Cmd::new("chroot")
            .arg(self.target.display().to_string())
            .args(args.iter().copied())
            .env("PATH", CHROOT_PATH)
            .env("LANG", "C.UTF-8")
            .env("DEBIAN_FRONTEND", "noninteractive")
    }

    /// `/etc/fstab` -> `<target>/etc/fstab`.
    fn t(&self, abs: &str) -> PathBuf {
        match abs.trim_start_matches('/') {
            "" => self.target.clone(),
            rel => self.target.join(rel),
        }
    }

    /// Whether `abs` exists on the target, with symlinks followed inside the
    /// target root. The payload's Nix profiles are absolute links into
    /// `/nix/store`, which `<target>/...` alone would resolve on the live
    /// system, where they don't exist.
    fn exists_on_target(&mut self, abs: &str) -> bool {
        let r = &mut self.r;
        let inner = resolve_in_root(&self.target, abs, |p| r.read_link(p));
        let host = match inner.to_str().unwrap_or("/").trim_start_matches('/') {
            "" => self.target.clone(),
            rel => self.target.join(rel),
        };
        self.r.exists(&host)
    }

    fn ts(&self, abs: &str) -> String {
        self.t(abs).display().to_string()
    }

    fn write(&mut self, abs: &str, contents: &str, mode: u32) -> R<()> {
        let p = self.t(abs);
        let lvl = self.cmd_level();
        self.log(lvl, format!("write {} (mode {mode:o})", p.display()));
        if self.dry() {
            for l in contents.lines() {
                self.log(lvl, format!("    | {l}"));
            }
        }
        self.r.write_file(&p, contents.as_bytes(), mode)?;
        Ok(())
    }

    /// Like `write`, root-only, and the content never shows in the log.
    fn write_secret(&mut self, abs: &str, contents: &str) -> R<()> {
        let p = self.t(abs);
        let lvl = self.cmd_level();
        self.log(
            lvl,
            format!("write {} (mode 600, contents not shown)", p.display()),
        );
        self.r.write_file(&p, contents.as_bytes(), 0o600)?;
        Ok(())
    }

    fn mkdir(&mut self, p: &Path) -> R<()> {
        self.r.mkdir_p(p)?;
        Ok(())
    }

    fn mount(&mut self, args: &[&str], mp: &Path) -> R<()> {
        self.mkdir(mp)?;
        let cmd = Cmd::new("mount")
            .args(args.iter().copied())
            .arg(mp.display().to_string());
        self.sh(cmd)?;
        self.mounts.push(mp.to_path_buf());
        Ok(())
    }

    fn umount(&mut self, mp: &Path) -> R<()> {
        let r = self.sh(Cmd::new("umount").arg(mp.display().to_string()));
        if r.is_err() {
            // Something still holds it (a stray process in the chroot):
            // detach it so the LUKS container can still close.
            self.try_sh(
                Cmd::new("umount")
                    .arg("--lazy")
                    .arg(mp.display().to_string()),
            );
        }
        self.mounts.retain(|m| m != mp);
        r.map(|_| ())
    }

    fn umount_all(&mut self) {
        while let Some(mp) = self.mounts.last().cloned() {
            let _ = self.umount(&mp);
        }
    }

    /// Undo whatever is still set up, newest first. Never fails.
    pub fn cleanup(&mut self) {
        // After Ctrl-C the runner refuses new commands; cleanup must run.
        super::runner::INTERRUPTED.store(false, std::sync::atomic::Ordering::SeqCst);
        if let Some(pid) = self.daemon.take() {
            self.info("stopping nix-daemon");
            self.r.kill(pid);
        }
        if !self.mounts.is_empty() {
            self.info("unmounting the target");
        }
        self.umount_all();
        if self.luks_open {
            self.try_sh(Cmd::new("cryptsetup").args(["close", LUKS_NAME]));
            self.luks_open = false;
        }
        // A free-space install that stops before the system can boot gives
        // the region back, or the next attempt finds no free space and only
        // a partition editor gets it back. Once the bootloader is in, the
        // installed system boots from these partitions, so they stay.
        let bootable = matches!(self.step, StepId::Home | StepId::Snapshot | StepId::Finish);
        if !self.created.is_empty() && !bootable {
            self.info("removing the partitions this install made");
            let dev = self.cfg.disk.device.clone();
            let mut args: Vec<String> = self
                .created
                .iter()
                .map(|n| format!("--delete={n}"))
                .collect();
            args.push(dev.clone());
            if self.try_sh(Cmd::new("sgdisk").args(args)) {
                self.try_sh(Cmd::new("partprobe").arg(&dev));
            }
            self.created.clear();
        }
    }

    fn parts(&self) -> R<Parts> {
        self.parts
            .clone()
            .ok_or_else(|| "partitions aren't set up (internal error)".into())
    }

    fn uuid(&mut self, dev: &str) -> R<String> {
        let out = self.sh(Cmd::new("blkid")
            .args(["-c", "/dev/null", "-s", "UUID", "-o", "value", dev])
            .placeholder(format!("<UUID of {dev}>")))?;
        let u = out.stdout.trim().to_string();
        if u.is_empty() {
            return Err(format!("blkid found no UUID on {dev}").into());
        }
        Ok(u)
    }

    fn wait_for(&mut self, p: &Path, secs: u64) -> bool {
        if self.dry() {
            return true;
        }
        let until = Instant::now() + Duration::from_secs(secs);
        while Instant::now() < until {
            if self.r.exists(p) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }

    // ------------------------------------------------------------------

    pub fn step(&mut self, id: StepId) -> R<()> {
        self.step = id;
        if interrupted() {
            return Err("interrupted".into());
        }
        match id {
            StepId::Preflight => self.preflight(),
            StepId::Partition => self.partition(),
            StepId::Encrypt => self.encrypt(),
            StepId::Filesystems => self.filesystems(),
            StepId::Unpack => self.unpack(),
            StepId::Configure => self.configure(),
            StepId::User => self.create_user(),
            StepId::Bootloader => self.bootloader(),
            StepId::Home => self.home(),
            StepId::Snapshot => self.snapshot(),
            StepId::Finish => self.finish(),
        }
    }

    fn preflight(&mut self) -> R<()> {
        let c = self.cfg;
        let mode = match c.disk.target {
            DiskTarget::Whole => "whole disk".to_string(),
            DiskTarget::FreeSpace { start, end } => {
                format!(
                    "free space {}..{} ({})",
                    start,
                    end,
                    gib(end.saturating_sub(start))
                )
            }
        };
        self.info(format!(
            "installing {} ({}, {}) for {}@{} on {} ({mode}), encryption {}",
            self.manifest.version,
            self.manifest.arch,
            self.manifest.commit,
            c.user.username,
            c.host.hostname,
            c.disk.device,
            if c.disk.encrypt { "on" } else { "off" },
        ));
        if self.dry() {
            self.info("dry run: nothing below is executed");
            return Ok(());
        }

        // Config vs machine, again: Kōbō calls `run` directly.
        let probe = machine::probe(Path::new(&c.payload.dir))?;
        let mut problems = validate::problems(c, &probe);
        problems.extend(validate::secret_problems(c, self.secrets));
        if !problems.is_empty() {
            return Err(StepError {
                message: problems.join("; "),
                hint: Some("Go back and fix the settings.".into()),
            });
        }
        self.progress(0.3, None);

        let mut tools = vec![
            "sgdisk",
            "sfdisk",
            "wipefs",
            "partprobe",
            "blkid",
            "mkfs.vfat",
            "mkfs.ext4",
            "mkfs.btrfs",
            "btrfs",
            "mount",
            "umount",
            "chroot",
        ];
        if c.disk.encrypt {
            tools.push("cryptsetup");
        }
        let image = Path::new(&c.payload.dir).join(&self.manifest.rootfs);
        let u = unpack::detect(&image, self.manifest.installed_bytes)?;
        tools.push(if u.name() == "squashfs" {
            "unsquashfs"
        } else {
            "tar"
        });
        let missing: Vec<&str> = tools.into_iter().filter(|t| which(t).is_none()).collect();
        if !missing.is_empty() {
            return Err(StepError {
                message: format!("missing tools on the live system: {}", missing.join(", ")),
                hint: Some(
                    "The ISO build must include them (see docs/INSTALL-INTERNALS.md).".into(),
                ),
            });
        }

        let size = std::fs::metadata(&image)
            .map_err(|e| format!("{}: {e}", image.display()))?
            .len();
        if size != self.manifest.rootfs_bytes {
            return Err(StepError {
                message: format!(
                    "{} is {size} bytes, the manifest says {}",
                    image.display(),
                    self.manifest.rootfs_bytes
                ),
                hint: Some(
                    "The payload is damaged or incomplete. Rebuild or re-copy the stick.".into(),
                ),
            });
        }

        // Nothing of ours may be in use already.
        if Path::new("/dev/mapper").join(LUKS_NAME).exists() {
            return Err(StepError {
                message: format!("/dev/mapper/{LUKS_NAME} is already open"),
                hint: Some(format!(
                    "A previous attempt left it open: cryptsetup close {LUKS_NAME}"
                )),
            });
        }
        let mounts = std::fs::read_to_string("/proc/self/mounts").unwrap_or_default();
        let target = self.target.display().to_string();
        if mounts
            .lines()
            .any(|l| l.split_whitespace().nth(1) == Some(target.as_str()))
        {
            return Err(StepError {
                message: format!("{target} is already mounted"),
                hint: Some(format!(
                    "A previous attempt left it mounted: umount -R {target}"
                )),
            });
        }
        if let Some(path) = self.disk.as_ref().map(|d| d.path.clone())
            && matches!(c.disk.target, DiskTarget::Whole)
        {
            let in_use = machine::in_use(&machine::read_lsblk()?, &path);
            // The live system turned on the swap it found on this disk, which
            // the install is about to erase anyway.
            for swap in &in_use.swaps {
                self.info(format!("turning off swap on {swap}"));
                if let Err(e) = self.sh(Cmd::new("swapoff").arg(swap)) {
                    return Err(StepError {
                        message: e.message,
                        hint: Some(format!("Free it and try again: swapoff {swap}")),
                    });
                }
            }
            if !in_use.mounts.is_empty() {
                return Err(StepError {
                    message: format!(
                        "{path} is in use (mounted at {}), so nothing got written",
                        in_use.mounts.join(", ")
                    ),
                    hint: Some("Free it and try again: unmount it.".into()),
                });
            }
        }
        self.progress(1.0, None);
        Ok(())
    }

    /// The disk's GPT as it is now.
    fn read_table(&mut self) -> R<PartTable> {
        let dev = self.cfg.disk.device.clone();
        if self.dry() {
            let (size, esp) = self
                .disk
                .as_ref()
                .map(|d| {
                    let esp = d
                        .partitions
                        .iter()
                        .find(|p| p.os_hint.as_deref() == Some("EFI system"))
                        .map(|p| (p.path.clone(), p.size_bytes));
                    (d.size_bytes, esp)
                })
                .unwrap_or((512 * disk::GIB, None));
            let mut t = PartTable::empty_gpt(&dev, size, 512);
            if let (DiskTarget::FreeSpace { .. }, Some((node, bytes))) =
                (&self.cfg.disk.target, esp)
            {
                t.partitions.push(disk::Part {
                    node,
                    start: 2048,
                    size: bytes / 512,
                    type_: disk::ESP_GUID.into(),
                    name: None,
                });
            }
            return Ok(t);
        }
        let out = self.sh(Cmd::new("sfdisk").args(["--json", &dev]))?;
        Ok(disk::parse_sfdisk(&out.stdout)?)
    }

    fn partition(&mut self) -> R<()> {
        let c = self.cfg;
        let dev = c.disk.device.clone();
        if matches!(c.disk.target, DiskTarget::Whole) {
            self.sh(Cmd::new("wipefs").args(["--all", "--force", &dev]))?;
            self.sh(Cmd::new("sgdisk").args(["--zap-all", &dev]))?;
            self.sh(Cmd::new("sgdisk").args(["--clear", &dev]))?;
        }
        self.progress(0.2, None);
        let table = self.read_table()?;
        let min_root = self.manifest.installed_bytes
            + (self.manifest.installed_bytes / 5).max(disk::ROOT_HEADROOM);
        let layout: Layout = disk::plan_layout(&c.disk.target, &table, c.disk.encrypt, min_root)?;
        let mut args: Vec<String> = layout
            .new_parts()
            .iter()
            .flat_map(|p| p.sgdisk_args())
            .collect();
        args.push(dev.clone());
        self.sh(Cmd::new("sgdisk").args(args))?;
        if matches!(c.disk.target, DiskTarget::FreeSpace { .. }) {
            self.created = layout.new_parts().iter().map(|p| p.number).collect();
        }
        self.try_sh(Cmd::new("partprobe").arg(&dev));
        if !self.dry() {
            let _ = self.r.run(&Cmd::new("udevadm").arg("settle"));
        }
        self.progress(0.6, None);

        // Nodes of the new partitions, from the table the kernel now has.
        let node_of = |table: &PartTable, p: &disk::NewPart| {
            table
                .partitions
                .iter()
                .find(|x| x.start == p.start)
                .map(|x| x.node.clone())
                .unwrap_or_else(|| disk::partition_path(&dev, p.number))
        };
        let now = if self.dry() {
            table.clone()
        } else {
            self.read_table()?
        };
        let (esp, esp_new) = match &layout.esp {
            Esp::New(p) => (node_of(&now, p), true),
            Esp::Reuse(node) => (node.clone(), false),
        };
        let boot = node_of(&now, &layout.boot);
        let root = node_of(&now, &layout.root);
        let root_bytes = (layout.root.end + 1 - layout.root.start) * table.sectorsize;
        let mut fresh = vec![boot.clone(), root.clone()];
        if esp_new {
            fresh.insert(0, esp.clone());
        } else {
            self.info(format!("reusing the EFI system partition {esp}"));
        }
        for n in &fresh {
            if !self.wait_for(Path::new(n), 10) {
                return Err(format!("{n} didn't appear after partitioning").into());
            }
            // Old signatures at other offsets would confuse blkid later.
            self.sh(Cmd::new("wipefs").args(["--all", "--force", n]))?;
        }
        self.info(format!(
            "ESP {esp}, /boot {boot}, root {root} ({})",
            gib(root_bytes)
        ));
        self.parts = Some(Parts {
            esp,
            esp_new,
            boot,
            fs_dev: root.clone(),
            root,
            root_bytes,
        });
        self.progress(1.0, None);
        Ok(())
    }

    fn encrypt(&mut self) -> R<()> {
        let mut p = self.parts()?;
        let pass = self
            .secrets
            .luks_passphrase
            .clone()
            .filter(|s| !s.is_empty())
            .ok_or("no LUKS passphrase in the secrets")?;
        // --key-file=- reads stdin to EOF: the passphrase exactly, no newline,
        // the same bytes you type at boot.
        self.sh(Cmd::new("cryptsetup")
            .args([
                "luksFormat",
                "--batch-mode",
                "--type",
                "luks2",
                "--pbkdf",
                "argon2id",
                "--label",
                LUKS_NAME,
                "--key-file=-",
                &p.root,
            ])
            .secret_stdin(&pass))?;
        self.progress(0.6, None);
        self.sh(Cmd::new("cryptsetup")
            .args([
                "open",
                "--allow-discards",
                "--key-file=-",
                &p.root,
                LUKS_NAME,
            ])
            .secret_stdin(&pass))?;
        self.luks_open = true;
        p.fs_dev = format!("/dev/mapper/{LUKS_NAME}");
        if !self.wait_for(Path::new(&p.fs_dev), 10) {
            return Err(format!("{} didn't appear", p.fs_dev).into());
        }
        self.parts = Some(p);
        self.progress(1.0, None);
        Ok(())
    }

    fn filesystems(&mut self) -> R<()> {
        let p = self.parts()?;
        if p.esp_new {
            self.sh(Cmd::new("mkfs.vfat").args(["-F", "32", "-n", "EFI", &p.esp]))?;
        }
        self.sh(Cmd::new("mkfs.ext4").args(["-F", "-q", "-L", "boot", &p.boot]))?;
        self.sh(Cmd::new("mkfs.btrfs").args(["-f", "-q", "-L", "root", &p.fs_dev]))?;
        self.progress(0.4, None);

        let top = self.top.clone();
        self.mount(&["-o", "subvolid=5", &p.fs_dev], &top)?;
        for (sv, _) in SUBVOLUMES {
            self.sh(Cmd::new("btrfs").args([
                "subvolume",
                "create",
                &top.join(sv).display().to_string(),
            ]))?;
        }
        self.umount(&top)?;
        self.progress(0.6, None);

        for (sv, mp) in SUBVOLUMES {
            let opts = files::subvol_opts(sv);
            let at = self.t(mp);
            self.mount(&["-o", &opts, &p.fs_dev], &at)?;
            if sv == "@" {
                // Tools that mount the device without options get @ too.
                self.sh(Cmd::new("btrfs").args([
                    "subvolume",
                    "set-default",
                    &at.display().to_string(),
                ]))?;
            }
        }
        let boot = self.t("/boot");
        self.mount(&[&p.boot], &boot)?;
        let efi = self.t("/boot/efi");
        self.mount(&["-o", "umask=0077", &p.esp], &efi)?;

        let swap = disk::swap_bytes(p.root_bytes);
        if swap > 0 {
            self.sh(Cmd::new("btrfs").args([
                "filesystem",
                "mkswapfile",
                "--size",
                &format!("{}g", swap / disk::GIB),
                &self.ts("/swap/swapfile"),
            ]))?;
            self.swapfile = true;
        } else {
            self.info("small disk: no swap file");
        }
        self.progress(1.0, None);
        Ok(())
    }

    fn unpack(&mut self) -> R<()> {
        let image = Path::new(&self.cfg.payload.dir).join(&self.manifest.rootfs);
        let total = self.manifest.installed_bytes;
        let mut u = unpack::detect(&image, total)?;
        let cmd = u.command(&image, &self.target);
        let lvl = self.cmd_level();
        self.log(lvl, format!("$ {}  ({})", cmd.display(), u.name()));
        self.progress(0.0, Some(format!("0.0 / {:.1} GB", total as f64 / 1e9)));

        let step = self.step;
        let emit = &mut *self.emit;
        let mut fraction = 0.0f32;
        let mut last = Instant::now();
        let mut logged = 0;
        let res = self.r.stream(&cmd, &mut |s| {
            let mut changed = false;
            match s {
                Stream::Line(l) => match u.progress(&l) {
                    Some(f) if f >= fraction => {
                        changed = f > fraction;
                        fraction = f;
                    }
                    Some(_) => {}
                    None if logged < 200 => {
                        logged += 1;
                        emit(Event::Log {
                            id: step,
                            level: Level::Debug,
                            line: l,
                        });
                    }
                    None => {}
                },
                Stream::Tick => {}
            }
            // At least every 250 ms (the runner ticks), at most every 100 ms.
            if (changed && last.elapsed() >= Duration::from_millis(100))
                || last.elapsed() >= Duration::from_millis(250)
            {
                last = Instant::now();
                let done = fraction as f64 * total as f64 / 1e9;
                emit(Event::Progress {
                    id: step,
                    fraction: Some(fraction),
                    detail: Some(format!("{done:.1} / {:.1} GB", total as f64 / 1e9)),
                });
            }
        });
        hint(
            res,
            "The payload may be damaged, or the disk is failing or too small.",
        )?;
        self.progress(1.0, Some(format!("{0:.1} / {0:.1} GB", total as f64 / 1e9)));
        Ok(())
    }

    fn bind_mounts(&mut self) -> R<()> {
        let dev = self.t("/dev");
        self.mount(&["--bind", "/dev"], &dev)?;
        let pts = self.t("/dev/pts");
        self.mount(&["--bind", "/dev/pts"], &pts)?;
        let proc_ = self.t("/proc");
        self.mount(&["-t", "proc", "proc"], &proc_)?;
        let sys = self.t("/sys");
        self.mount(&["-t", "sysfs", "sysfs"], &sys)?;
        if Path::new("/sys/firmware/efi/efivars").exists() {
            let ev = self.t("/sys/firmware/efi/efivars");
            self.mount(&["-t", "efivarfs", "efivarfs"], &ev)?;
        }
        // A fresh /run, not the live one: tools in the chroot must not reach
        // the live system's systemd or D-Bus.
        let run = self.t("/run");
        self.mount(
            &["-t", "tmpfs", "-o", "nosuid,nodev,mode=0755", "tmpfs"],
            &run,
        )?;
        Ok(())
    }

    fn configure(&mut self) -> R<()> {
        let c = self.cfg;
        let p = self.parts()?;
        self.bind_mounts()?;

        let u = FsUuids {
            root: self.uuid(&p.fs_dev)?,
            boot: self.uuid(&p.boot)?,
            esp: self.uuid(&p.esp)?,
        };
        let fstab = files::fstab(&u, self.swapfile);
        self.write("/etc/fstab", &fstab, 0o644)?;
        if c.disk.encrypt {
            let luks = self.uuid(&p.root)?;
            self.write("/etc/crypttab", &files::crypttab(&luks), 0o644)?;
        }
        self.progress(0.3, None);

        let h = &c.host.hostname;
        self.write("/etc/hostname", &format!("{h}\n"), 0o644)?;
        self.write("/etc/hosts", &files::hosts(h), 0o644)?;

        let tz = &c.locale.timezone;
        self.write("/etc/timezone", &format!("{tz}\n"), 0o644)?;
        self.sh(Cmd::new("ln").args([
            "-sfn",
            &format!("../usr/share/zoneinfo/{tz}"),
            &self.ts("/etc/localtime"),
        ]))?;

        // Ubuntu 26.04 keeps the locale in /etc/locale.conf, and systemd's
        // tmpfiles (debian.conf, `L+`) turns /etc/default/locale back into a
        // link to it on every boot. Writing only /etc/default/locale left the
        // installed system in the C locale. Write the real file, link the old
        // name to it the same way tmpfiles would.
        self.write("/etc/locale.conf", &files::locale(&c.locale.lang), 0o644)?;
        self.sh(Cmd::new("ln").args(["-sfn", "../locale.conf", &self.ts("/etc/default/locale")]))?;
        if self.dry() || self.r.exists(&self.t("/usr/sbin/locale-gen")) {
            let cmd = self.chroot(&["locale-gen", &c.locale.lang]);
            self.try_sh(cmd);
        }
        // Every layout goes to the one file: Hyprland takes them all (`config/hypr/input.lua`),
        // and the console and the LUKS prompt type with the first, the default.
        self.write(
            keyboard::DEFAULT_KEYBOARD,
            &keyboard::default_keyboard(&c.locale.keyboards()),
            0o644,
        )?;
        self.progress(0.6, None);

        // A real machine-id now: an empty one makes systemd treat the first
        // boot as "first boot" and run `preset-all`, which would undo unit
        // choices system/*.sh made in the image.
        let id = if self.dry() {
            "<random 128-bit hex>".to_string()
        } else {
            machine_id()?
        };
        self.write("/etc/machine-id", &format!("{id}\n"), 0o444)?;
        if self.dry() || self.r.exists(&self.t("/var/lib/dbus")) {
            self.sh(Cmd::new("ln").args([
                "-sfn",
                "/etc/machine-id",
                &self.ts("/var/lib/dbus/machine-id"),
            ]))?;
        }
        if self.r.exists(&self.t("/etc/ssh/sshd_config")) {
            let cmd = self.chroot(&["ssh-keygen", "-A"]);
            self.try_sh(cmd);
        }
        if matches!(c.disk.target, DiskTarget::FreeSpace { .. }) {
            // Dual boot: let update-grub list the other system.
            self.write(
                "/etc/default/grub.d/90-kubaos-os-prober.cfg",
                "# KubaOS: installed next to another system\nGRUB_DISABLE_OS_PROBER=false\n",
                0o644,
            )?;
        }
        if let Some(w) = &c.wifi {
            // The one secret that goes to the target: NM needs the psk, and
            // keeps it the same way (root-only keyfile).
            let uuid = if self.dry() {
                "<random uuid>".to_string()
            } else {
                uuid_v4()?
            };
            let f = format!(
                "/etc/NetworkManager/system-connections/{}",
                files::nmconnection_name(&w.ssid)
            );
            let text = files::nmconnection(&w.ssid, self.secrets.wifi_psk.as_deref(), &uuid);
            self.write_secret(&f, &text)?;
        }
        self.install_setup()?;
        self.progress(1.0, None);
        Ok(())
    }

    fn create_user(&mut self) -> R<()> {
        let c = self.cfg;
        let name = c.user.username.clone();
        let passwd = self.r.read_file(&self.t("/etc/passwd")).unwrap_or_default();
        if files::passwd_entry(&passwd, &name).is_some() {
            return Err(StepError {
                message: format!("the payload already has a user {name}"),
                hint: Some("The image must ship without users (docs/INSTALL-INTERNALS.md).".into()),
            });
        }
        let mut cmd_args: Vec<String> = vec![
            "useradd".into(),
            "--create-home".into(),
            "--user-group".into(),
        ];
        match files::uid_owner(&passwd, USER_UID) {
            None => cmd_args.extend(["--uid".into(), USER_UID.to_string()]),
            Some(other) => self.warn(format!(
                "uid {USER_UID} belongs to {other} in the payload; letting useradd pick"
            )),
        }
        let groups = match self.r.read_file(&self.t("/etc/group")) {
            Some(g) => files::existing_groups(&g, &files::USER_GROUPS),
            None => files::USER_GROUPS.iter().map(|s| s.to_string()).collect(),
        };
        if !groups.is_empty() {
            cmd_args.extend(["--groups".into(), groups.join(",")]);
        }
        let shell = if self.dry() || self.r.exists(&self.t("/usr/bin/zsh")) {
            "/usr/bin/zsh"
        } else {
            self.warn("no /usr/bin/zsh in the payload; the login shell is bash");
            "/bin/bash"
        };
        cmd_args.extend([
            "--shell".into(),
            shell.into(),
            "--comment".into(),
            c.user.full_name.clone(),
            name.clone(),
        ]);
        let refs: Vec<&str> = cmd_args.iter().map(String::as_str).collect();
        let cmd = self.chroot(&refs);
        self.sh(cmd)?;
        self.progress(0.4, None);

        let cmd = self
            .chroot(&["chpasswd"])
            .secret_stdin(&format!("{name}:{}\n", self.secrets.user_password));
        self.sh(cmd)?;

        let passwd = self.r.read_file(&self.t("/etc/passwd")).unwrap_or_default();
        let (uid, gid, home) = match files::passwd_entry(&passwd, &name) {
            Some(e) => e,
            None if self.dry() => (USER_UID, USER_UID, format!("/home/{name}")),
            None => return Err(format!("useradd ran but {name} isn't in /etc/passwd").into()),
        };
        self.user = Some((uid, gid, home.clone()));
        let owner = format!("{uid}:{gid}");

        if let Some(g) = &c.git {
            // ~/.config/git-identity/identity.gitconfig, the file first-boot
            // setup's Git card writes too. Root makes the directories, so
            // hand them over before home-manager wants ~/.config.
            let f = format!("{home}/{}", files::GIT_IDENTITY);
            let dir = format!("{home}/.config/git-identity");
            self.mkdir(&self.t(&dir))?;
            self.write(&f, &files::gitconfig(g.name.trim(), g.email.trim()), 0o644)?;
            self.sh(Cmd::new("chown").args([
                &owner,
                &self.ts(&format!("{home}/.config")),
                &self.ts(&dir),
                &self.ts(&f),
            ]))?;
        }
        // ~/.config/kubaos/wallpapers/sets: the wallpapers the picker shows.
        // Every set of the payload's unless the user picked; an older payload
        // lists none, and without the file dot-theme shows every set on the
        // machine.
        let wallpapers = c.wallpapers.clone().unwrap_or_else(|| Wallpapers {
            sets: self
                .manifest
                .wallpaper_sets
                .iter()
                .map(|s| s.id.clone())
                .collect(),
            files: Vec::new(),
        });
        if c.wallpapers.is_some() || !wallpapers.sets.is_empty() {
            let f = format!("{home}/{}", files::WALLPAPER_SETS);
            let dir = f.rsplit_once('/').map_or("", |(d, _)| d).to_string();
            self.mkdir(&self.t(&dir))?;
            self.write(&f, &files::wallpaper_sets(&wallpapers), 0o644)?;
            self.sh(Cmd::new("chown").args([
                &owner,
                &self.ts(&format!("{home}/.config")),
                &self.ts(&format!("{home}/.config/kubaos")),
                &self.ts(&dir),
                &self.ts(&f),
            ]))?;
        }
        self.install_keyring(&home, &owner)?;

        // What the image's scripts wrote into /etc for the build user.
        for (template, dest) in files::USER_TEMPLATES {
            if !self.r.exists(&self.t(dest)) {
                continue;
            }
            let src = format!("{PAYLOAD_KUBAOS}/{template}");
            match self.r.read_file(&self.t(&src)) {
                Some(t) => {
                    self.write(dest, &files::render_user_template(&t, &name, &home), 0o644)?
                }
                None => self.warn(format!(
                    "{dest} names the payload's user, and {src} isn't there to render it for {name}"
                )),
            }
        }

        self.write_user_flake(&home, &owner)?;
        self.write_release_records(&home, &owner)?;
        self.progress(1.0, None);
        Ok(())
    }

    /// ~/.config/kubaos/home: user-flake/ from KubaOS's checkout with the
    /// name, host and system filled in, and the payload's lock. The name is
    /// a literal in the user's own file, so KubaOS's checkout stays clean.
    fn write_user_flake(&mut self, home: &str, owner: &str) -> R<()> {
        let c = self.cfg;
        let system = format!("{}-linux", self.manifest.arch);
        let dir = format!("{home}/{}", files::USER_FLAKE);
        let template = format!("{PAYLOAD_KUBAOS}/user-flake");
        // (source, file in the user flake, filled in from the template)
        let sources = [
            (format!("{template}/flake.nix.in"), "flake.nix", true),
            (format!("{template}/home.nix"), "home.nix", false),
            (USER_FLAKE_LOCK.to_string(), "flake.lock", false),
        ];
        self.mkdir(&self.t(&dir))?;
        for (src, name, fill) in sources {
            let contents = match self.r.read_file(&self.t(&src)) {
                Some(t) if fill => {
                    files::user_flake(&t, &c.user.username, &c.host.hostname, &system)
                }
                Some(t) => t,
                None if self.dry() => {
                    self.info(format!("would write {dir}/{name} from {src}"));
                    continue;
                }
                None => {
                    return Err(StepError {
                        message: format!("{src} is not in the payload"),
                        hint: Some(
                            "The payload ships the user flake's template and lock (install/image)."
                                .into(),
                        ),
                    });
                }
            };
            self.write(&format!("{dir}/{name}"), &contents, 0o644)?;
        }
        self.sh(Cmd::new("chown").args([owner, &self.ts(&format!("{home}/.config"))]))?;
        self.sh(Cmd::new("chown").args([
            "-R",
            owner,
            &self.ts(&format!("{home}/.config/kubaos")),
        ]))?;
        Ok(())
    }

    fn installed_kernels(&mut self) -> Vec<String> {
        if self.dry() {
            return vec![self.manifest.kernel.clone()];
        }
        let mut v: Vec<String> = std::fs::read_dir(self.t("/lib/modules"))
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        v.retain(|k| {
            self.target
                .join("boot")
                .join(format!("vmlinuz-{k}"))
                .exists()
        });
        v.sort();
        v
    }

    fn bootloader(&mut self) -> R<()> {
        let grub_target = match self.manifest.arch.as_str() {
            "x86_64" => "x86_64-efi",
            "aarch64" => "arm64-efi",
            a => return Err(format!("no GRUB target for {a}").into()),
        };
        // The initramfs must know the new crypttab to unlock root at boot.
        let kernels = self.installed_kernels();
        if kernels.is_empty() {
            self.warn("no kernel in /boot of the payload: skipping update-initramfs");
        }
        for k in &kernels {
            let have = self.dry() || self.r.exists(&self.t(&format!("/boot/initrd.img-{k}")));
            let cmd = self.chroot(&["update-initramfs", if have { "-u" } else { "-c" }, "-k", k]);
            self.sh_stream(cmd, Level::Debug)?;
        }
        self.progress(0.4, None);

        // Another system's EFI/BOOT/BOOT<arch>.EFI (Windows keeps its own
        // there) must survive: Ubuntu's grub-install writes that path too
        // when it can't write NVRAM. Set it aside and put it back after.
        let whole = matches!(self.cfg.disk.target, DiskTarget::Whole);
        let fallback = self.ts("/boot/efi/EFI/BOOT");
        let saved = self
            .top
            .with_file_name("efi-boot-saved")
            .display()
            .to_string();
        let keep_fallback = !whole && (self.dry() || self.r.exists(Path::new(&fallback)));
        if keep_fallback {
            self.sh(Cmd::new("rm").args(["-rf", &saved]))?;
            // -r, not -a: vfat has no owners to keep.
            self.sh(Cmd::new("cp").args(["-r", &fallback, &saved]))?;
        }

        let nvram = Path::new("/sys/firmware/efi/efivars").exists();
        let mut args = vec![
            "grub-install".to_string(),
            format!("--target={grub_target}"),
            "--efi-directory=/boot/efi".into(),
            "--bootloader-id=ubuntu".into(),
            "--recheck".into(),
        ];
        if !nvram {
            self.info(
                "no EFI variables here: not writing a boot entry, relying on the fallback path",
            );
            args.push("--no-nvram".into());
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let cmd = self.chroot(&refs);
        hint(
            self.sh(cmd).map_err(|e| e.message),
            "grub-install failed; is grub-efi in the payload?",
        )?;
        self.progress(0.6, None);

        if keep_fallback {
            self.sh(Cmd::new("rm").args(["-rf", &fallback]))?;
            self.sh(Cmd::new("cp").args(["-r", &saved, &fallback]))?;
            self.sh(Cmd::new("rm").args(["-rf", &saved]))?;
            self.info("kept the other system's EFI/BOOT fallback loader");
        } else {
            // EFI/BOOT/BOOT<arch>.EFI, for firmware that forgets boot
            // entries (VMs).
            let t = format!("--target={grub_target}");
            let cmd = self.chroot(&[
                "grub-install",
                &t,
                "--efi-directory=/boot/efi",
                "--removable",
                "--no-nvram",
            ]);
            self.sh(cmd)?;
        }
        self.progress(0.75, None);
        let cmd = self.chroot(&["update-grub"]);
        self.sh_stream(cmd, Level::Debug)?;
        if !self.dry() && !self.r.exists(&self.t("/boot/grub/grub.cfg")) {
            return Err("update-grub wrote no /boot/grub/grub.cfg".into());
        }
        self.progress(1.0, None);
        Ok(())
    }

    fn home(&mut self) -> R<()> {
        // The payload's generation is built for manifest.username. Anyone
        // else gets theirs built here, offline (rebuild_home).
        let rebuild = self.cfg.user.username != self.manifest.username;
        let generation = if rebuild {
            HOME_REBUILT.to_string()
        } else {
            self.manifest.home_generation.clone()
        };
        let activate = format!("{generation}/activate");
        if !rebuild && !self.dry() && !self.r.exists(&self.t(&activate)) {
            return Err(StepError {
                message: format!("{activate} is not in the payload"),
                hint: Some(
                    "manifest.home_generation must name a store path inside the rootfs.".into(),
                ),
            });
        }
        let (uid, gid, home) = self.user.clone().unwrap_or((
            USER_UID,
            USER_UID,
            format!("/home/{}", self.cfg.user.username),
        ));
        let name = self.cfg.user.username.clone();
        let owner = format!("{uid}:{gid}");

        // What nix-daemon would create on the user's first connection.
        for d in [
            format!("/nix/var/nix/profiles/per-user/{name}"),
            format!("/nix/var/nix/gcroots/per-user/{name}"),
        ] {
            let p = self.t(&d);
            if self.dry() || self.r.exists(p.parent().unwrap_or(&p)) {
                self.mkdir(&p)?;
                self.sh(Cmd::new("chown").args([&owner, &p.display().to_string()]))?;
            }
        }

        // home-manager's activation talks to the store through nix-daemon
        // (multi-user Nix, as system/60-nix.sh installs it). Nothing runs in
        // the chroot, so start one there for the duration.
        let daemon = format!("{NIX_BIN}/nix-daemon");
        let multi_user = self.dry() || self.exists_on_target(&daemon);
        if multi_user {
            // Activation builds the profile's user-environment, and Nix's
            // sandbox can't pivot_root inside a chroot whose / isn't a mount.
            // Only this throwaway daemon runs unsandboxed.
            let cmd = self.chroot(&[&daemon, "--option", "sandbox", "false"]);
            let lvl = self.cmd_level();
            self.log(lvl, format!("$ {} &", cmd.display()));
            let pid = self.r.spawn(&cmd)?;
            self.daemon = Some(pid);
            let sock = self.t("/nix/var/nix/daemon-socket/socket");
            if !self.wait_for(&sock, 20) {
                return Err("nix-daemon didn't open its socket in the chroot".into());
            }
        } else {
            self.warn("no nix-daemon in the payload: activating against the store directly");
        }
        self.progress(0.2, None);

        let user_env = [
            format!("HOME={home}"),
            format!("USER={name}"),
            format!("LOGNAME={name}"),
            "SHELL=/usr/bin/zsh".into(),
            format!("PATH={NIX_BIN}:{CHROOT_PATH}"),
            format!("LANG={}", self.cfg.locale.lang),
            format!("NIX_REMOTE={}", if multi_user { "daemon" } else { "" }),
            // useradd copied /etc/skel; files home-manager wants to own are
            // renamed instead of failing the activation.
            "HOME_MANAGER_BACKUP_EXT=hm-backup".into(),
        ];
        // home-manager's dconf step needs a session bus. Without one it starts
        // Nix's dbus-daemon, which looks for /etc/dbus-1/session.conf, and
        // Ubuntu keeps that file in /usr/share. Ubuntu's dbus-run-session
        // finds its own config.
        let bus = if !self.dry() && self.r.exists(&self.t("/usr/bin/dbus-run-session")) {
            "dbus-run-session -- "
        } else {
            ""
        };
        let as_user = |script: String| -> Vec<String> {
            let mut args: Vec<String> = vec![
                "setpriv".into(),
                format!("--reuid={uid}"),
                format!("--regid={gid}"),
                "--init-groups".into(),
                "--".into(),
                "env".into(),
                "-i".into(),
            ];
            args.extend(user_env.iter().cloned());
            args.extend(["/bin/sh".into(), "-c".into(), script]);
            args
        };
        let mut res = Ok(());
        if rebuild {
            let args = as_user(self.rebuild_home_script());
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            let cmd = self.chroot(&refs);
            self.info(format!(
                "building the home-manager generation for {name} (the payload's is for {})",
                self.manifest.username
            ));
            res = self.sh_stream(cmd, Level::Debug).map_err(|e| StepError {
                message: e.message,
                hint: Some(
                    "Building the generation for another username failed; the log has Nix's output. \
                     The payload's own username needs no build."
                        .into(),
                ),
            });
            self.progress(0.6, None);
        }
        if res.is_ok() {
            let args = as_user(format!("cd \"$HOME\" && exec {bus}{activate}"));
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            let cmd = self.chroot(&refs);
            res = self.sh_stream(cmd, Level::Info).map_err(|e| StepError {
                message: e.message,
                hint: Some("home-manager activation failed; the log has its output.".into()),
            });
        }
        if let Some(pid) = self.daemon.take() {
            self.r.kill(pid);
        }
        res?;
        self.progress(1.0, None);
        Ok(())
    }

    /// As the user, in the user flake (write_user_flake): build
    /// `<user>@<host>` offline. Only the derivations that name the user
    /// rebuild; the payload carries their build inputs (install/image).
    /// The username is validated (a-z, 0-9, - and _), so it goes in as is.
    fn rebuild_home_script(&self) -> String {
        let user = &self.cfg.user.username;
        let flake_out = format!(
            "homeConfigurations.\\\"{user}@{}\\\".activationPackage",
            self.cfg.host.hostname
        );
        format!(
            "set -e; cd \"$HOME/{}\" && exec nix --offline --option experimental-features 'nix-command flakes' build -L \
             --out-link {HOME_REBUILT} \".#{flake_out}\"",
            files::USER_FLAKE
        )
    }

    fn snapshot(&mut self) -> R<()> {
        let snapper = self.dry() || self.r.exists(&self.t("/usr/bin/snapper"));
        if snapper {
            self.snapshot_snapper()?;
        } else {
            self.warn("no snapper in the payload: taking the snapshot with btrfs directly, in snapper's layout");
            self.snapshot_raw()?;
        }
        // grub-btrfs lists snapshots in the menu; refresh it so "factory" is
        // there from the first boot.
        if self.dry() || self.r.exists(&self.t("/etc/grub.d/41_snapshots-btrfs")) {
            let cmd = self.chroot(&["update-grub"]);
            if let Err(e) = self.sh_stream(cmd, Level::Debug) {
                self.warn(e.message);
            }
        }
        self.progress(1.0, None);
        Ok(())
    }

    fn snapshot_snapper(&mut self) -> R<()> {
        let p = self.parts()?;
        let user = self.cfg.user.username.clone();
        let snaps = self.t("/.snapshots");
        if !self.r.exists(&self.t("/etc/snapper/configs/root")) {
            // snapper insists on creating /.snapshots itself (same dance as
            // system/20-base.sh): let it, then put @snapshots back.
            self.umount(&snaps)?;
            self.sh(Cmd::new("rmdir").arg(snaps.display().to_string()))?;
            let cmd = self.chroot(&["snapper", "--no-dbus", "-c", "root", "create-config", "/"]);
            self.sh(cmd)?;
            self.sh(Cmd::new("btrfs").args(["subvolume", "delete", &snaps.display().to_string()]))?;
            self.mount(
                &["-o", &files::subvol_opts("@snapshots"), &p.fs_dev],
                &snaps,
            )?;
            self.sh(Cmd::new("chmod").args(["750", &snaps.display().to_string()]))?;
        }
        let mut args = vec![
            "snapper".to_string(),
            "--no-dbus".into(),
            "-c".into(),
            "root".into(),
            "set-config".into(),
        ];
        args.extend(files::snapper_settings(&user));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let cmd = self.chroot(&refs);
        self.try_sh(cmd);
        self.progress(0.5, None);
        let cmd = self.chroot(&[
            "snapper",
            "--no-dbus",
            "-c",
            "root",
            "create",
            "--cleanup-algorithm",
            "number",
            "--description",
            "factory",
            "--print-number",
        ]);
        let out = self.sh(cmd.placeholder("1"))?;
        let n = out.stdout.trim().to_string();
        self.info(format!("factory snapshot is #{n}"));
        if n != "1" {
            self.warn("expected the factory snapshot to be #1");
        }
        let cmd = self.chroot(&[
            "snapper",
            "--no-dbus",
            "-c",
            "root",
            "set-config",
            "SYNC_ACL=yes",
        ]);
        if !self.try_sh(cmd) {
            self.warn(
                "couldn't turn on SYNC_ACL (no ACLs on this btrfs?); ALLOW_USERS still applies",
            );
            // Leave it off, or every snapper call would fail the same way.
            let cmd = self.chroot(&[
                "snapper",
                "--no-dbus",
                "-c",
                "root",
                "set-config",
                "SYNC_ACL=no",
            ]);
            self.try_sh(cmd);
        }
        Ok(())
    }

    fn snapshot_raw(&mut self) -> R<()> {
        let p = self.parts()?;
        let top = self.top.clone();
        self.mount(&["-o", "subvolid=5", &p.fs_dev], &top)?;
        let dir = top.join("@snapshots/1");
        self.mkdir(&dir)?;
        self.sh(Cmd::new("btrfs").args([
            "subvolume",
            "snapshot",
            "-r",
            &top.join("@").display().to_string(),
            &dir.join("snapshot").display().to_string(),
        ]))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let xml = files::snapper_info(1, &files::utc_datetime(now), "factory");
        let info = dir.join("info.xml");
        self.log(self.cmd_level(), format!("write {}", info.display()));
        self.r.write_file(&info, xml.as_bytes(), 0o644)?;
        self.umount(&top)?;
        Ok(())
    }

    fn finish(&mut self) -> R<()> {
        if let Some(log) = self.log_file.clone()
            && (self.dry() || log.exists())
        {
            let dest = self.t("/var/log/kubaos/install.log");
            self.sh(Cmd::new("install").args([
                "-D",
                "-m",
                "0600",
                &log.display().to_string(),
                &dest.display().to_string(),
            ]))?;
        }
        self.progress(0.3, None);
        self.try_sh(Cmd::new("sync"));
        // Not `cleanup()`: here an unmount failure is reported.
        let mut failed = Vec::new();
        while let Some(mp) = self.mounts.last().cloned() {
            if self.umount(&mp).is_err() {
                failed.push(mp.display().to_string());
            }
        }
        self.progress(0.8, None);
        if self.luks_open {
            self.sh(Cmd::new("cryptsetup").args(["close", LUKS_NAME]))?;
            self.luks_open = false;
        }
        if !failed.is_empty() {
            self.warn(format!("had to detach busy mounts: {}", failed.join(", ")));
        }
        self.progress(1.0, None);
        Ok(())
    }
}

fn which(tool: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .chain(["/usr/sbin", "/sbin"].map(PathBuf::from))
        .map(|d| d.join(tool))
        .find(|p| p.is_file())
}

fn uuid_v4() -> Result<String, String> {
    let mut b = random16()?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    ))
}

fn machine_id() -> Result<String, String> {
    Ok(random16()?.iter().map(|x| format!("{x:02x}")).collect())
}

fn random16() -> Result<[u8; 16], String> {
    let mut b = [0u8; 16];
    std::io::Read::read_exact(
        &mut std::fs::File::open("/dev/urandom").map_err(|e| format!("/dev/urandom: {e}"))?,
        &mut b,
    )
    .map_err(|e| format!("/dev/urandom: {e}"))?;
    Ok(b)
}

/// `abs` as a path inside `root` with every symlink resolved the way it would
/// be after `chroot root`: absolute link targets start again at `root`, `..`
/// never climbs out of it. Gives up (returns the path so far) after 40 links.
fn resolve_in_root(
    root: &Path,
    abs: &str,
    mut read_link: impl FnMut(&Path) -> Option<PathBuf>,
) -> PathBuf {
    let mut todo: Vec<String> = abs
        .split('/')
        .filter(|c| !c.is_empty())
        .rev()
        .map(String::from)
        .collect();
    let mut cur = PathBuf::from("/");
    let mut hops = 0;
    while let Some(c) = todo.pop() {
        match c.as_str() {
            "." => continue,
            ".." => {
                cur.pop();
                continue;
            }
            _ => {}
        }
        let next = cur.join(&c);
        let host = root.join(next.strip_prefix("/").unwrap_or(&next));
        if let Some(link) = read_link(&host) {
            hops += 1;
            if hops > 40 {
                return next;
            }
            if link.is_absolute() {
                cur = PathBuf::from("/");
            }
            let l = link.to_string_lossy().into_owned();
            todo.extend(
                l.split('/')
                    .filter(|p| !p.is_empty())
                    .rev()
                    .map(String::from),
            );
            continue;
        }
        cur = next;
    }
    cur
}

#[cfg(test)]
mod resolve_tests {
    use super::resolve_in_root;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};

    #[test]
    fn follows_absolute_links_inside_the_root() {
        let root = std::env::temp_dir().join(format!("kubaos-resolve-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("nix/store/abc-env/bin")).unwrap();
        std::fs::write(root.join("nix/store/abc-env/bin/nix-daemon"), "").unwrap();
        std::fs::create_dir_all(root.join("nix/var/nix/profiles")).unwrap();
        symlink(
            "/nix/store/abc-env",
            root.join("nix/var/nix/profiles/default-1-link"),
        )
        .unwrap();
        symlink("default-1-link", root.join("nix/var/nix/profiles/default")).unwrap();
        symlink("../../../etc", root.join("nix/var/escape")).unwrap();

        let rl = |p: &Path| std::fs::read_link(p).ok();
        let got = resolve_in_root(&root, "/nix/var/nix/profiles/default/bin/nix-daemon", rl);
        assert_eq!(got, PathBuf::from("/nix/store/abc-env/bin/nix-daemon"));
        assert!(root.join(got.strip_prefix("/").unwrap()).exists());
        // `..` stops at the root.
        let got = resolve_in_root(&root, "/nix/var/escape/x", rl);
        assert_eq!(got, PathBuf::from("/etc/x"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
