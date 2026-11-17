//! The power menu's state changes: opening and closing it, the keys while it is open, and taking
//! the machine down, after stopping a running install first.

use std::sync::Arc;
use std::time::Instant;

use iced::Task;

use super::App;
use super::F;
use super::Key;
use super::Message;
use super::PowerAction;
use super::PowerMenu;

impl App {
    pub(super) fn open_power(&mut self) {
        if self.power.is_some() || self.shutting_down.is_some() {
            return;
        }
        self.hold = None;
        self.power = Some(PowerMenu {
            return_focus: self.focus,
            stopping: None,
        });
        // The safe answer first: Enter twice in a row never takes the machine down.
        self.set_focus(Some(F::PowerCancel));
    }

    /// Closes the power menu, unless it's already taking the machine down.
    pub(super) fn close_power(&mut self) {
        let Some(menu) = self.power else {
            return;
        };
        if menu.stopping.is_some() || self.shutting_down.is_some() {
            return;
        }
        self.power = None;
        self.set_focus(menu.return_focus);
    }

    /// The menu's Power off or Restart: at once, or once the running install has stopped.
    pub(super) fn confirm_power(&mut self, action: PowerAction) -> Task<Message> {
        if self.shutting_down.is_some() {
            return Task::none();
        }
        let running = self.install_running();
        let Some(menu) = &mut self.power else {
            return Task::none();
        };
        if menu.stopping.is_some() {
            return Task::none();
        }
        if !running {
            return self.shut_down(action);
        }
        self.now = Instant::now();
        menu.stopping = Some((action, self.now));
        self.backend.stop();
        Task::none()
    }

    /// Asks systemd to power off or restart, on a thread of its own. systemctl returns once
    /// systemd has the job and the machine takes a few seconds to go down, so the screen says so
    /// meanwhile.
    pub(super) fn shut_down(&mut self, action: PowerAction) -> Task<Message> {
        if self.shutting_down.is_some() {
            return Task::none();
        }
        self.shutting_down = Some(action);
        self.now = Instant::now();
        let backend = Arc::clone(&self.backend);
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let r = match action {
                PowerAction::PowerOff => backend.poweroff(),
                PowerAction::Restart => backend.reboot(),
            };
            let _ = tx.send(r);
        });
        Task::perform(
            async move { rx.await.unwrap_or_else(|_| Err("it crashed".into())) },
            move |r| Message::ShutDown(action, r),
        )
    }

    /// Keys while the power menu is open: it keeps them all, so nothing behind it moves.
    pub(super) fn power_key(&mut self, k: &Key) -> Task<Message> {
        let busy = self.shutting_down.is_some() || self.power.is_some_and(|m| m.stopping.is_some());
        if busy {
            return Task::none();
        }
        match *k {
            Key::Esc => {
                self.close_power();
                Task::none()
            }
            Key::Tab { shift } => self.cycle_focus(shift),
            Key::Left { .. } => self.cycle_focus(true),
            Key::Right { .. } => self.cycle_focus(false),
            Key::Enter | Key::Space => match self.focus {
                Some(f) if f.in_power_menu() => self.activate(f),
                _ => Task::none(),
            },
            _ => Task::none(),
        }
    }
}
