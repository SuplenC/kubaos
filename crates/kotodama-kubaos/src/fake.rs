//! Stand-ins for the system bus and the programs, for the unit tests: they
//! keep what they were asked, and answer what the test set up.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::time::Duration;

use crate::bus::Bus;
use crate::bus::BusError;
use crate::bus::Property;
use crate::run::RunError;
use crate::run::Runner;

/// Properties by `service path interface.name`, and the children of objects
/// by `service path`.
#[derive(Debug, Default)]
pub struct FakeBus {
    pub booleans: BTreeMap<String, bool>,
    pub strings: BTreeMap<String, String>,
    pub children: BTreeMap<String, Vec<String>>,
    /// Every call fails, as without a system bus.
    pub down: bool,
}

pub fn key(property: &Property) -> String {
    format!(
        "{} {} {}.{}",
        property.service, property.path, property.interface, property.name
    )
}

impl FakeBus {
    fn check(&self, service: &str) -> Result<(), BusError> {
        if self.down {
            return Err(BusError::Answer {
                service: service.to_owned(),
                expected: "anything, as the bus is down",
            });
        }
        Ok(())
    }
}

impl Bus for FakeBus {
    fn get_bool(&mut self, property: &Property) -> Result<bool, BusError> {
        self.check(property.service)?;
        self.booleans.get(&key(property)).copied().ok_or(BusError::Answer {
            service: property.service.to_owned(),
            expected: "a boolean",
        })
    }

    fn set_bool(&mut self, property: &Property, value: bool) -> Result<(), BusError> {
        self.check(property.service)?;
        self.booleans.insert(key(property), value);
        Ok(())
    }

    fn set_string(&mut self, property: &Property, value: &str) -> Result<(), BusError> {
        self.check(property.service)?;
        self.strings.insert(key(property), value.to_owned());
        Ok(())
    }

    fn children(&mut self, service: &str, path: &str) -> Result<Vec<String>, BusError> {
        self.check(service)?;
        Ok(self
            .children
            .get(&format!("{service} {path}"))
            .cloned()
            .unwrap_or_default())
    }
}

/// Records each program it is asked to run, and answers with `outputs` by
/// the whole command line.
#[derive(Debug, Default)]
pub struct FakeRunner {
    pub ran: RefCell<Vec<Vec<String>>>,
    pub launched: RefCell<Vec<(Vec<String>, Duration)>>,
    pub outputs: BTreeMap<String, String>,
    /// Every program fails, saying this.
    pub failing: Option<String>,
}

impl FakeRunner {
    pub fn ran(&self) -> Vec<String> {
        self.ran.borrow().iter().map(|argv| argv.join(" ")).collect()
    }
}

impl Runner for FakeRunner {
    fn output(&self, argv: &[String]) -> Result<String, RunError> {
        self.ran.borrow_mut().push(argv.to_vec());
        if let Some(stderr) = &self.failing {
            return Err(RunError::Failed {
                program: argv[0].clone(),
                stderr: stderr.clone(),
            });
        }
        Ok(self.outputs.get(&argv.join(" ")).cloned().unwrap_or_default())
    }

    fn launch(&self, argv: &[String], delay: Duration) -> Result<(), RunError> {
        if self.failing.is_some() {
            return Err(RunError::Launch(io::Error::other(format!(
                "couldn't start {}",
                argv[0]
            ))));
        }
        self.launched.borrow_mut().push((argv.to_vec(), delay));
        Ok(())
    }
}
