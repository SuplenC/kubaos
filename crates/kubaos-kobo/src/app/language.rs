//! The Language screen's keyboard layouts: the row Space acts on, and ticking a layout.

use super::App;
use super::F;

impl App {
    /// The keymap row Space acts on: the cursor's, else the default layout's.
    pub fn keymap_row(&self) -> String {
        self.keymap_cursor
            .clone()
            .or_else(|| self.form.keymaps.first().cloned())
            .unwrap_or_default()
    }

    /// Space and Shift+Space act on the keymap list while its search box has the focus with
    /// nothing typed: a search never starts with a space.
    pub(super) fn keymap_list_keys(&self) -> bool {
        self.focus == Some(F::KeymapQ) && self.keymap_q.is_empty()
    }

    pub(super) fn toggle_keymap(&mut self, keymap: &str) {
        let r = self.form.toggle_keymap(keymap);
        self.refused(r);
    }
}
