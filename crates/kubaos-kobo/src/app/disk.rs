//! The Disk screen's install mode: picking one, when the picked disk allows it.

use super::App;
use crate::model::DiskExt;
use crate::model::DiskMode;

impl App {
    pub(super) fn pick_mode(&mut self, m: DiskMode) {
        let ok = match m {
            DiskMode::Whole => true,
            DiskMode::FreeSpace => self
                .ready_probe()
                .and_then(|p| self.form.selected_disk(p).map(|d| d.fits_dual_boot(p)))
                .unwrap_or(false),
        };
        if ok {
            self.form.disk_mode = Some(m);
        }
    }

    pub(super) fn toggle_mode(&mut self) {
        let m = match self.form.disk_mode {
            Some(DiskMode::Whole) => DiskMode::FreeSpace,
            _ => DiskMode::Whole,
        };
        self.pick_mode(m);
    }
}
