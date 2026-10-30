//! Date answers in the search: `days until christmas`, `next friday`, `15:00 tokyo`, worked out by
//! the clock popup's engine (`kubaos-clock`) with the popup's own settings, so the menu and the
//! popup count the same holidays and know the same zones.
//!
//! # Owns
//!
//! - Which Live queries get a date answer, the answer as a result, and what its Actions copy.
//!
//! # Not here
//!
//! - Parsing and working dates out: `kubaos_clock`.
//! - The clock's Items (Open calendar, Add countdown, Add time zone, settings): registry actions in
//!   `config/kotodama/kubaos.toml`.
//! - The protocol loop: `main.rs`.

use std::path::PathBuf;
use std::time::Duration;

use jiff::Zoned;
use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Fraction;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::LiveResult;
use kotodama_protocol::NamespacedKind;
use kubaos_clock::Answer;
use kubaos_clock::serve::Server;

use crate::run::Runner;

/// The Item kind of a date answer.
pub const KIND: &str = "kubaos.date";
/// Copies the answer, as shown.
pub const COPY: &str = "copy";
/// Copies the answer's ISO 8601 form: a date, a time with its offset, a duration or an interval.
pub const COPY_ISO: &str = "copy-iso";
/// Opens the clock popup on the query.
pub const SHOW: &str = "show";

/// Single words that are dates on their own. Any other one-word query, `mon` or `may` say, is
/// more likely the start of an app's name than a date, and an answer would push the app down.
const DATE_WORDS: [&str; 3] = ["today", "tomorrow", "yesterday"];

/// The date engine, and the last answer, which the Actions copy.
#[derive(Debug)]
pub struct Dates {
    server: Server,
    last: Option<(String, Answer)>,
}

impl Dates {
    /// Dates read with the clock settings in `dir` (`~/.config/kubaos/desktop/clock`).
    #[must_use]
    pub fn new(dir: Option<PathBuf>) -> Self {
        Dates {
            server: Server::new(dir),
            last: None,
        }
    }

    /// The answer to the Live query `query` as of `now`, if it is a date query.
    pub fn live(&mut self, query: &str, now: &Zoned) -> Option<LiveResult> {
        let query = query.trim();
        if !worth_answering(query) {
            return None;
        }
        self.server.refresh();
        let answer = self.server.query(query, now)?;
        let result = live_result(query, &answer);
        self.last = Some((query.to_owned(), answer));
        Some(result)
    }

    /// Runs `action` on the answer `id`, its query: what to copy, or `None` once the popup has
    /// been asked to show it.
    ///
    /// # Errors
    ///
    /// When the query no longer answers, the Action is unknown, or the popup can't be reached;
    /// the message is what the user reads.
    pub fn run(&mut self, action: &str, id: &str, now: &Zoned, runner: &impl Runner) -> Result<Option<String>, String> {
        if action == SHOW {
            let argv: Vec<String> = ["qs", "ipc", "call", "clock", "search", id].map(str::to_owned).into();
            runner
                .launch(&argv, Duration::ZERO)
                .map_err(|err| Chain(&err).to_string())?;
            return Ok(None);
        }
        // An answer's id is its query, so one older than the last is worked out again.
        let answer = match &self.last {
            Some((query, answer)) if query == id => answer.clone(),
            _ => {
                self.server.refresh();
                self.server
                    .query(id, now)
                    .ok_or_else(|| format!("{id:?} no longer has an answer"))?
            }
        };
        match action {
            COPY => Ok(Some(answer.title)),
            COPY_ISO => Ok(Some(answer.iso)),
            _ => Err(format!("a date answer has no Action `{action}`")),
        }
    }
}

/// Whether `query` may be a date query: more than one word, or one with a digit or a date word.
fn worth_answering(query: &str) -> bool {
    if query.contains(char::is_whitespace) {
        return true;
    }
    query.contains(|c: char| c.is_ascii_digit()) || DATE_WORDS.contains(&query.to_lowercase().as_str())
}

fn live_result(query: &str, answer: &Answer) -> LiveResult {
    LiveResult {
        item: Item {
            id: query.to_owned(),
            kind: ItemKind::Namespaced(kind()),
            identity: None,
            title: answer.title.clone(),
            subtitle: Some(answer.detail.clone()),
            icon: Some("x-office-calendar".into()),
            keyword: None,
            // Kotodama's calculator answers date queries too, its answer found by the question in
            // its subtitle, a match inside a word. The question as a keyword is a word start, a
            // Tier above, so of the two answers the clock's, which knows the countdowns and the
            // popup's holidays, takes the top row.
            keywords: vec![query.to_owned()],
            hidden: false,
            app_only: false,
            weak_keywords: Vec::new(),
            yields: false,
            actions: vec![
                Action::new(COPY, "Copy Answer"),
                Action::new(COPY_ISO, "Copy as ISO").shortcut("ctrl+enter"),
                Action::new(SHOW, "Show in Calendar").shortcut("ctrl+o"),
            ],
        },
        score: certain(),
        answer: true,
    }
}

fn kind() -> NamespacedKind {
    NamespacedKind::new(KIND).expect("the date kind is valid")
}

/// An answer's score: the highest there is.
fn certain() -> Fraction {
    Fraction::new(1.0).expect("1 is a fraction")
}

#[cfg(test)]
mod tests;
