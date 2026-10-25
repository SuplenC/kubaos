//! Every Suite KubaOS ships, as its metadata. A new Suite is one entry in [`SHIPPED`] plus its Nix
//! module, `home/suites/<name>.nix`.
//!
//! # Owns
//!
//! - The list of Suites, and finding one by name.
//! - The test Suite, built in only for tests and with the `test-suite` feature.
//!
//! # Not here
//!
//! - What a Suite's fields mean, and everything done with them: `suites.rs`.

#[cfg(any(test, feature = "test-suite"))]
use kubaos_user_layer::App;

#[cfg(any(test, feature = "test-suite"))]
use super::Cache;
#[cfg(any(test, feature = "test-suite"))]
use super::Extra;
#[cfg(any(test, feature = "test-suite"))]
use super::Scaffolder;
#[cfg(any(test, feature = "test-suite"))]
use super::Seed;
use super::Suite;
#[cfg(any(test, feature = "test-suite"))]
use super::Toolchain;

/// The Suites KubaOS ships, by name.
pub const SHIPPED: &[Suite] = &[];

/// A Suite with one of everything, for the test harness and the unit tests. Its Nix module is
/// `home/suites/test.nix`.
#[cfg(any(test, feature = "test-suite"))]
pub const TEST: Suite = Suite {
    name: "test",
    title: "Test",
    description: "KubaOS's own test Suite: a script, an environment variable, a PATH entry and a Neovim setting.",
    versions: &["1", "2"],
    extras: &[Extra {
        name: "greeting",
        description: "Adds kubaos-test-suite-greet.",
        default: false,
    }],
    toolchain: Toolchain::Nix,
    caches: &[Cache {
        path: ".cache/kubaos-test-suite",
        description: "What `kubaos-test-suite run` writes.",
    }],
    scaffolders: &[Scaffolder {
        id: "new",
        label: "New Test project",
        argv: &["kubaos-test-suite", "new", "{path}"],
    }],
    seeds: &[Seed {
        app: App::Nvim,
        path: "plugins/test-suite.lua",
        contents: include_str!("seeds/test-suite.lua"),
    }],
    mentions: &["kubaos-test-suite", "kubaos_test_suite"],
};

#[cfg(any(test, feature = "test-suite"))]
const TESTING: &[Suite] = &[TEST];
#[cfg(not(any(test, feature = "test-suite")))]
const TESTING: &[Suite] = &[];

/// Every Suite this build knows, shipped ones first.
pub fn all() -> impl Iterator<Item = &'static Suite> {
    SHIPPED.iter().chain(TESTING)
}

/// The Suite called `name`.
pub fn find(name: &str) -> Option<&'static Suite> {
    all().find(|suite| suite.name == name)
}
