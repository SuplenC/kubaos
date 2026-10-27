//! The machine for real: programs through `kubaos_core`'s runner, names through the system's
//! resolver, and the tunnel through
//! NetworkManager's D-Bus API on the system bus.
//!
//! # Owns
//!
//! - [`Real`], and turning a [`Connection`] into NetworkManager's settings dictionary.
//!
//! # Not here
//!
//! - What the connection holds: [`crate::tunnel`].

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::ToSocketAddrs as _;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use kubaos_core::bitwarden::Host as _;
use zbus::blocking::Connection as Bus;
use zbus::blocking::Proxy;
use zbus::zvariant::ObjectPath;
use zbus::zvariant::OwnedObjectPath;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Value;

use crate::machine::Cmd;
use crate::machine::Machine;
use crate::machine::Ran;
use crate::machine::ResolveError;
use crate::machine::Said;
use crate::machine::StartError;
use crate::machine::TunnelError;
use crate::tunnel::Connection;
use crate::tunnel::ID;
use crate::tunnel::Prop;

const NM: &str = "org.freedesktop.NetworkManager";
const NM_PATH: &str = "/org/freedesktop/NetworkManager";
const ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
/// `NM_ACTIVE_CONNECTION_STATE_ACTIVATED`; above it, the connection is going or gone.
const ACTIVATED: u32 = 2;
/// How long NetworkManager gets to bring the tunnel up. A WireGuard link has no handshake to
/// wait for, so it is up in well under a second; a name to resolve for the endpoint is the most
/// it waits on.
const UP_TIMEOUT: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(100);
/// How long NetworkManager gets to take the tunnel down: a deactivation is near instant.
const DOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// The programs and NetworkManager, for real.
#[derive(Debug, Default)]
pub struct Real {
    host: kubaos_core::bitwarden::Real,
    bus: Option<Bus>,
}

impl Real {
    #[must_use]
    pub fn new() -> Real {
        Real::default()
    }

    fn bus(&mut self) -> Result<Bus, TunnelError> {
        if let Some(bus) = &self.bus {
            return Ok(bus.clone());
        }
        let bus = Bus::system().map_err(unreachable)?;
        self.bus = Some(bus.clone());
        Ok(bus)
    }

    /// The active connections KubaOS's tunnel has, with their state.
    fn ours(&mut self) -> Result<Vec<(OwnedObjectPath, u32)>, TunnelError> {
        let bus = self.bus()?;
        let nm = Proxy::new(&bus, NM, NM_PATH, NM).map_err(unreachable)?;
        let active: Vec<OwnedObjectPath> = nm.get_property("ActiveConnections").map_err(unreachable)?;
        let mut ours = Vec::new();
        for path in active {
            let Ok(connection) = Proxy::new(&bus, NM, path.clone(), ACTIVE) else {
                continue;
            };
            if connection.get_property::<String>("Id").is_ok_and(|id| id == ID) {
                let state = connection.get_property::<u32>("State").unwrap_or(0);
                ours.push((path, state));
            }
        }
        Ok(ours)
    }
}

fn unreachable(err: impl std::fmt::Display) -> TunnelError {
    TunnelError::Unreachable(Said(err.to_string()))
}

fn refused(err: zbus::Error) -> TunnelError {
    TunnelError::Refused(Said(match err {
        zbus::Error::MethodError(_, Some(message), _) => message,
        other => other.to_string(),
    }))
}

/// A [`Prop`] as a D-Bus value.
fn value(prop: &Prop) -> Value<'static> {
    match prop {
        Prop::Str(s) | Prop::Secret(s) => Value::from(s.clone()),
        Prop::U32(n) => Value::U32(*n),
        Prop::I32(n) => Value::I32(*n),
        Prop::Bool(b) => Value::Bool(*b),
        Prop::Strs(list) => Value::from(list.clone()),
        Prop::Dicts(dicts) => Value::from(dicts.iter().map(dict).collect::<Vec<_>>()),
    }
}

fn dict(props: &BTreeMap<String, Prop>) -> HashMap<String, Value<'static>> {
    props.iter().map(|(k, v)| (k.clone(), value(v))).collect()
}

fn settings(connection: &Connection) -> HashMap<String, HashMap<String, Value<'static>>> {
    connection
        .iter()
        .map(|(name, props)| (name.clone(), dict(props)))
        .collect()
}

impl Machine for Real {
    fn run(&mut self, cmd: &Cmd) -> Result<Ran, StartError> {
        self.host.run(cmd).map_err(|said| StartError {
            program: cmd.program.clone(),
            said: Said(said),
        })
    }

    fn tunnel_up(&mut self, connection: &Connection) -> Result<(), TunnelError> {
        self.tunnel_down()?;
        let bus = self.bus()?;
        let nm = Proxy::new(&bus, NM, NM_PATH, NM).map_err(unreachable)?;
        let root = ObjectPath::from_static_str_unchecked("/");
        let options: HashMap<&str, Value<'_>> = HashMap::from([("persist", Value::from("volatile"))]);
        let (_, active, _): (OwnedObjectPath, OwnedObjectPath, HashMap<String, OwnedValue>) = nm
            .call(
                "AddAndActivateConnection2",
                &(settings(connection), &root, &root, options),
            )
            .map_err(refused)?;
        let start = Instant::now();
        loop {
            let state = Proxy::new(&bus, NM, active.clone(), ACTIVE)
                .and_then(|connection| connection.get_property::<u32>("State"))
                .ok();
            match state {
                Some(ACTIVATED) => return Ok(()),
                Some(state) if state > ACTIVATED => return Err(TunnelError::Dropped),
                None => return Err(TunnelError::Dropped),
                Some(_) if start.elapsed() > UP_TIMEOUT => {
                    let _ = self.tunnel_down();
                    return Err(TunnelError::NotUp(UP_TIMEOUT.as_secs()));
                }
                Some(_) => thread::sleep(POLL),
            }
        }
    }

    fn tunnel_down(&mut self) -> Result<(), TunnelError> {
        let ours = self.ours()?;
        if ours.is_empty() {
            return Ok(());
        }
        let bus = self.bus()?;
        let nm = Proxy::new(&bus, NM, NM_PATH, NM).map_err(unreachable)?;
        for (path, _) in ours {
            let done: zbus::Result<()> = nm.call("DeactivateConnection", &(&path,));
            if let Err(err) = done
                && !matches!(&err, zbus::Error::MethodError(name, _, _) if name.as_str().ends_with("ConnectionNotActive"))
            {
                return Err(refused(err));
            }
        }
        // A new tunnel takes the same interface, which NetworkManager frees only once the old
        // connection is gone, a moment after it said yes.
        let start = Instant::now();
        while !self.ours()?.is_empty() {
            if start.elapsed() > DOWN_TIMEOUT {
                return Err(TunnelError::StillUp(DOWN_TIMEOUT.as_secs()));
            }
            thread::sleep(POLL);
        }
        Ok(())
    }

    fn tunnel_active(&mut self) -> Result<bool, TunnelError> {
        Ok(self.ours()?.iter().any(|(_, state)| *state == ACTIVATED))
    }

    fn resolve(&mut self, host: &str, port: u16) -> Result<Vec<IpAddr>, ResolveError> {
        let addrs = (host, port).to_socket_addrs().map_err(|err| ResolveError {
            host: host.to_owned(),
            said: Said(err.to_string()),
        })?;
        Ok(addrs.map(|addr| addr.ip()).collect())
    }
}
