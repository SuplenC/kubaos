//! A machine made of strings, for the tests.

use std::collections::BTreeMap;

use super::{Cmd, Output, Stream, Sys};

#[derive(Default)]
pub struct FakeSys {
    pub files: BTreeMap<String, String>,
    /// (prefix of the command line, result). First match wins; anything
    /// unmatched fails like a missing program.
    pub responses: Vec<(String, Result<String, String>)>,
    /// Command lines run, in order (env and stdin never included).
    pub ran: Vec<String>,
    /// The stdin each command got, by position in `ran`.
    pub stdin: Vec<Option<String>>,
    pub env: Vec<Vec<(String, String)>>,
    pub online_fallback: bool,
}

impl FakeSys {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file(mut self, path: &str, contents: &str) -> Self {
        self.files.insert(path.into(), contents.into());
        self
    }

    pub fn ok(mut self, prefix: &str, stdout: &str) -> Self {
        self.responses.push((prefix.into(), Ok(stdout.into())));
        self
    }

    pub fn fail(mut self, prefix: &str, err: &str) -> Self {
        self.responses.push((prefix.into(), Err(err.into())));
        self
    }

    fn line(cmd: &Cmd) -> String {
        std::iter::once(cmd.program.clone())
            .chain(cmd.args.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn respond(&mut self, cmd: &Cmd) -> Result<Output, String> {
        let line = Self::line(cmd);
        self.ran.push(line.clone());
        self.stdin.push(cmd.stdin.as_ref().map(|s| s.0.clone()));
        self.env.push(cmd.env.clone());
        match self
            .responses
            .iter()
            .find(|(p, _)| line.starts_with(p.as_str()))
        {
            Some((_, Ok(out))) => Ok(Output {
                stdout: out.clone(),
                stderr: String::new(),
            }),
            Some((_, Err(e))) => Err(e.clone()),
            None => Err(format!("{}: not found", cmd.program)),
        }
    }

    pub fn ran_any(&self, prefix: &str) -> bool {
        self.ran.iter().any(|l| l.starts_with(prefix))
    }
}

impl Sys for FakeSys {
    fn read(&mut self, path: &str) -> Option<String> {
        self.files.get(path).cloned()
    }

    fn exists(&mut self, path: &str) -> bool {
        let dir = format!("{}/", path.trim_end_matches('/'));
        self.files.contains_key(path) || self.files.keys().any(|k| k.starts_with(&dir))
    }

    fn list(&mut self, dir: &str) -> Vec<String> {
        let dir = format!("{}/", dir.trim_end_matches('/'));
        let mut v: Vec<String> = self
            .files
            .keys()
            .filter_map(|k| k.strip_prefix(&dir))
            .map(|rest| rest.split('/').next().unwrap_or(rest).to_string())
            .collect();
        v.dedup();
        v
    }

    fn run(&mut self, cmd: &Cmd) -> Result<Output, String> {
        self.respond(cmd)
    }

    fn stream(&mut self, cmd: &Cmd, f: &mut dyn FnMut(Stream)) -> Result<(), String> {
        let out = self.respond(cmd)?;
        for l in out.stdout.lines() {
            f(Stream::Line(l.to_string()));
        }
        Ok(())
    }

    fn write(&mut self, path: &str, contents: &str) -> Result<(), String> {
        self.files.insert(path.into(), contents.into());
        Ok(())
    }

    fn reachable(&mut self) -> bool {
        self.online_fallback
    }
}
