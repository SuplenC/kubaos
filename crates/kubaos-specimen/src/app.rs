//! The live specimen, in a window: the pages at the display's own scale, the motion samples moving.
//!
//! # Owns
//!
//! - The window, the keys (Tab and the arrows for pages, M for the mode, Esc to close; on the
//!   templates page, the template's, and Ctrl+Tab to leave it) and the motion clock.
//!
//! # Not here
//!
//! - What is drawn: [`crate::specimen`].

use std::time::Instant;

use iced::Element;
use iced::Size;
use iced::Subscription;
use iced::Task;
use iced::Theme;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::window;
use kubaos_ui::Mode;
use kubaos_ui::Scale;

use crate::pages::templates::Demo;
use crate::shot;
use crate::specimen;
use crate::specimen::Message;
use crate::specimen::Page;
use crate::specimen::Specimen;

/// The window's size when it opens: wide enough for the pages' four columns.
// design-allow(size): the window's opening size; tiling sets the real one.
const WINDOW: Size = Size::new(1280.0, 900.0);

struct App {
    specimen: Specimen,
    started: Instant,
    /// The templates page's live sidebar page.
    demo: Demo,
}

impl App {
    fn new() -> (App, Task<Message>) {
        let app = App {
            specimen: Specimen {
                page: Page::Type,
                mode: Mode::Dark,
                scale: Scale::default(),
                seconds: 0.0,
                live: true,
            },
            started: Instant::now(),
            demo: Demo::default(),
        };
        let scale = window::oldest().and_then(window::scale_factor).map(Message::Scaled);
        (app, scale)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Page(page) => {
                self.specimen.page = page;
                if page == Page::Templates {
                    return self.demo.page.measure().map(Message::Template);
                }
            }
            Message::Mode(mode) => self.specimen.mode = mode,
            Message::Scaled(factor) => self.specimen.scale = Scale::new(factor),
            Message::Frame(now) => {
                self.specimen.seconds = now.saturating_duration_since(self.started).as_secs_f32();
            }
            Message::Key(event) => return self.key(&event),
            Message::Template(message) => return self.demo.update(message),
            Message::Sample => {}
        }
        Task::none()
    }

    fn key(&mut self, event: &keyboard::Event) -> Task<Message> {
        let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
            return Task::none();
        };
        // On the templates page the keys are the template's, all but Ctrl+Tab, which leaves it.
        if self.specimen.page == Page::Templates {
            if matches!(key.as_ref(), keyboard::Key::Named(Named::Tab)) && modifiers.control() {
                self.specimen.page = self.specimen.page.step(modifiers.shift());
            }
            return Task::none();
        }
        match key.as_ref() {
            keyboard::Key::Named(Named::Tab) => self.specimen.page = self.specimen.page.step(modifiers.shift()),
            keyboard::Key::Named(Named::ArrowRight) => self.specimen.page = self.specimen.page.step(false),
            keyboard::Key::Named(Named::ArrowLeft) => self.specimen.page = self.specimen.page.step(true),
            keyboard::Key::Named(Named::Escape) => return iced::exit(),
            keyboard::Key::Character("m" | "M") => {
                self.specimen.mode = match self.specimen.mode {
                    Mode::Dark => Mode::Light,
                    Mode::Light => Mode::Dark,
                };
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        specimen::view(&self.specimen, Some(&self.demo))
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            keyboard::listen().map(Message::Key),
            window::events().filter_map(|(_, event)| match event {
                window::Event::Rescaled(factor) => Some(Message::Scaled(factor)),
                _ => None,
            }),
        ];
        // On the templates page the keys are the template's, as they are an app's.
        if self.specimen.page == Page::Templates {
            subscriptions.push(self.demo.page.subscription().map(Message::Template));
        }
        if self.specimen.page == Page::Motion {
            subscriptions.push(window::frames().map(Message::Frame));
        }
        Subscription::batch(subscriptions)
    }
}

/// Opens the specimen's window and runs it until it closes.
///
/// # Errors
///
/// When the window or its renderer can't start.
pub fn run() -> iced::Result {
    let mut settings = window::Settings {
        size: WINDOW,
        ..window::Settings::default()
    };
    #[cfg(target_os = "linux")]
    {
        settings.platform_specific.application_id = "kubaos-specimen".into();
    }
    let mut app = iced::application(App::new, App::update, App::view)
        .title("KubaOS design specimen")
        .subscription(App::subscription)
        .theme(|_: &App| Theme::Dark)
        .window(settings)
        .default_font(kubaos_ui::text::default_font())
        .antialiasing(true);
    for font in shot::fonts() {
        app = app.font(font);
    }
    app.run()
}
