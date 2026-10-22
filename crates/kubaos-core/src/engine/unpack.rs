//! Unpacking the payload's rootfs. Two formats, behind one small trait, so
//! the image build can pick either: squashfs (`unsquashfs -percentage`) or a
//! zstd tar (`tar --zstd` with checkpoints).

use std::path::Path;

use super::runner::Cmd;

pub trait Unpacker {
    fn name(&self) -> &'static str;
    /// The command that unpacks `image` into the existing directory `dest`.
    fn command(&self, image: &Path, dest: &Path) -> Cmd;
    /// Fraction done (0..=1) from one output line, if the line carries it.
    fn progress(&mut self, line: &str) -> Option<f32>;
}

pub struct Squashfs;

impl Unpacker for Squashfs {
    fn name(&self) -> &'static str {
        "squashfs"
    }

    fn command(&self, image: &Path, dest: &Path) -> Cmd {
        // -f: dest and the mount points under it already exist.
        // xattrs (file capabilities) are kept by default.
        Cmd::new("unsquashfs").args([
            "-percentage".to_string(),
            "-f".into(),
            "-d".into(),
            dest.display().to_string(),
            image.display().to_string(),
        ])
    }

    fn progress(&mut self, line: &str) -> Option<f32> {
        let n: u32 = line.trim().parse().ok()?;
        (n <= 100).then(|| n as f32 / 100.0)
    }
}

pub struct TarZstd {
    /// Uncompressed size from the manifest; tar reports records read.
    pub installed_bytes: u64,
}

/// Records between checkpoints; one record is 20 blocks of 512 bytes.
const CHECKPOINT: u64 = 1000;
const RECORD: u64 = 10240;
const CKPT_TAG: &str = "kubaos-ckpt ";

impl Unpacker for TarZstd {
    fn name(&self) -> &'static str {
        "tar.zst"
    }

    fn command(&self, image: &Path, dest: &Path) -> Cmd {
        Cmd::new("tar").args([
            "--zstd".to_string(),
            "-x".into(),
            "-p".into(),
            "--numeric-owner".into(),
            "--xattrs".into(),
            "--xattrs-include=*".into(),
            "--acls".into(),
            format!("--checkpoint={CHECKPOINT}"),
            format!("--checkpoint-action=echo={CKPT_TAG}%u"),
            "-f".into(),
            image.display().to_string(),
            "-C".into(),
            dest.display().to_string(),
        ])
    }

    fn progress(&mut self, line: &str) -> Option<f32> {
        let i = line.find(CKPT_TAG)?;
        let records: u64 = line[i + CKPT_TAG.len()..].trim().parse().ok()?;
        let bytes = records * RECORD;
        // The tar stream is a bit bigger than the files; never claim 100%
        // before tar exits.
        Some((bytes as f64 / self.installed_bytes.max(1) as f64).min(0.99) as f32)
    }
}

/// Picks the unpacker from the file's first bytes, or its name when the
/// file can't be read (dry run on another machine).
pub fn detect(path: &Path, installed_bytes: u64) -> Result<Box<dyn Unpacker>, String> {
    let mut head = [0u8; 4];
    let magic = std::fs::File::open(path)
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut head))
        .ok()
        .map(|_| head);
    match magic {
        Some(m) if &m == b"hsqs" => Ok(Box::new(Squashfs)),
        Some([0x28, 0xb5, 0x2f, 0xfd]) => Ok(Box::new(TarZstd { installed_bytes })),
        Some(_) => Err(format!("{}: neither squashfs nor zstd", path.display())),
        None => {
            let name = path.to_string_lossy();
            if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
                Ok(Box::new(TarZstd { installed_bytes }))
            } else {
                Ok(Box::new(Squashfs))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squashfs_progress() {
        let mut u = Squashfs;
        assert_eq!(u.progress("Parallel unsquashfs: Using 8 processors"), None);
        assert_eq!(u.progress("1225 inodes (1392 blocks) to write"), None);
        assert_eq!(u.progress("42"), Some(0.42));
        assert_eq!(u.progress("100"), Some(1.0));
        assert_eq!(u.progress("101"), None);
        let c = u.command(Path::new("/p/rootfs.sqfs"), Path::new("/mnt/t"));
        assert_eq!(
            c.display(),
            "unsquashfs -percentage -f -d /mnt/t /p/rootfs.sqfs"
        );
    }

    #[test]
    fn tar_progress() {
        let mut u = TarZstd {
            installed_bytes: 10240 * 4000,
        };
        assert_eq!(u.progress("tar: kubaos-ckpt 1000"), Some(0.25));
        assert_eq!(u.progress("tar: kubaos-ckpt 9000"), Some(0.99));
        assert_eq!(u.progress("tar: something else"), None);
        let c = u.command(Path::new("/p/rootfs.tar.zst"), Path::new("/mnt/t"));
        assert!(
            c.display()
                .contains("'--checkpoint-action=echo=kubaos-ckpt %u'")
        );
    }

    #[test]
    fn detect_by_magic_and_name() {
        let dir = std::env::temp_dir().join(format!("kubaos-unpack-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sq = dir.join("a.img");
        std::fs::write(&sq, b"hsqs....").unwrap();
        assert_eq!(detect(&sq, 1).unwrap().name(), "squashfs");
        let zs = dir.join("b.img");
        std::fs::write(&zs, [0x28, 0xb5, 0x2f, 0xfd, 0]).unwrap();
        assert_eq!(detect(&zs, 1).unwrap().name(), "tar.zst");
        let bad = dir.join("c.img");
        std::fs::write(&bad, b"nope").unwrap();
        assert!(detect(&bad, 1).is_err());
        assert_eq!(
            detect(&dir.join("missing.tar.zst"), 1).unwrap().name(),
            "tar.zst"
        );
        assert_eq!(
            detect(&dir.join("missing.sqfs"), 1).unwrap().name(),
            "squashfs"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
