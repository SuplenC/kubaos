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

use kubaos_user_layer::App;

use super::Cache;
use super::Extra;
use super::Scaffolder;
use super::Seed;
use super::Suite;
use super::Toolchain;

/// The Suites KubaOS ships, by name.
pub const SHIPPED: &[Suite] = &[NODE, PYTHON, PHP, PROTO];

/// Node and Bun. Its Nix module is `home/suites/node.nix`.
const NODE: Suite = Suite {
    name: "node",
    title: "Node",
    description: "Node and Bun, TypeScript's language server, eslint and the debugger, set up in Neovim.",
    versions: &["26", "24", "22"],
    extras: &[Extra {
        name: "bun",
        description: "Adds Bun, its package manager and its test runner.",
        default: true,
    }],
    toolchain: Toolchain::Nix,
    caches: &[
        Cache {
            path: ".npm",
            description: "npm's download cache and npx's packages.",
        },
        Cache {
            path: ".local/share/npm",
            description: "The packages `npm install --global` installed.",
        },
        Cache {
            path: ".cache/bun",
            description: "Bun's download cache.",
        },
        Cache {
            path: ".local/share/bun",
            description: "The packages `bun add --global` installed.",
        },
    ],
    // Node's first, as `kubaos suite new` takes the first by default: it needs no extra.
    scaffolders: &[
        Scaffolder {
            id: "node",
            label: "New Node + TypeScript app",
            argv: &["kubaos-node-new", "{path}"],
        },
        Scaffolder {
            id: "bun",
            label: "New Bun app",
            argv: &["bun", "init", "--yes", "{path}"],
        },
    ],
    seeds: &[Seed {
        app: App::Nvim,
        path: "plugins/node-suite.lua",
        contents: include_str!("seeds/node-suite.lua"),
    }],
    mentions: &[
        "nodejs",
        "npm",
        "npx",
        "bun",
        "javascript",
        "typescript",
        "vtsls",
        "eslint",
    ],
};

/// Python through uv. Its Nix module is `home/suites/python.nix`.
const PYTHON: Suite = Suite {
    name: "python",
    title: "Python",
    description: "uv for Pythons and venvs, basedpyright, ruff and debugpy, set up in Neovim.",
    versions: &[],
    extras: &[],
    toolchain: Toolchain::Upstream { manager: "uv" },
    caches: &[
        Cache {
            path: ".cache/uv",
            description: "uv's download and build cache.",
        },
        Cache {
            path: ".local/share/uv",
            description: "The Pythons and tools uv installed, and their commands.",
        },
    ],
    scaffolders: &[Scaffolder {
        id: "uv",
        label: "New Python project",
        argv: &["uv", "init", "{path}"],
    }],
    seeds: &[Seed {
        app: App::Nvim,
        path: "plugins/python-suite.lua",
        contents: include_str!("seeds/python-suite.lua"),
    }],
    mentions: &[
        "python",
        "python3",
        "pip",
        "venv",
        "pyproject",
        "basedpyright",
        "pyright",
        "ruff",
        "debugpy",
    ],
};

/// PHP and composer. Its Nix module is `home/suites/php.nix`.
const PHP: Suite = Suite {
    name: "php",
    title: "PHP",
    description: "PHP with Xdebug, composer, phpactor, php-cs-fixer and phpcs, set up in Neovim.",
    versions: &["8.5", "8.4", "8.3", "8.2"],
    extras: &[],
    toolchain: Toolchain::Nix,
    caches: &[Cache {
        path: ".cache/composer",
        description: "composer's download cache.",
    }],
    scaffolders: &[
        Scaffolder {
            id: "composer",
            label: "New PHP project",
            argv: &["kubaos-php-new", "{path}"],
        },
        Scaffolder {
            id: "laravel",
            label: "New Laravel app",
            argv: &["composer", "create-project", "laravel/laravel", "{path}"],
        },
    ],
    seeds: &[Seed {
        app: App::Nvim,
        path: "plugins/php-suite.lua",
        contents: include_str!("seeds/php-suite.lua"),
    }],
    mentions: &[
        "php",
        "composer",
        "laravel",
        "artisan",
        "phpactor",
        "intelephense",
        "xdebug",
        "php-cs-fixer",
        "phpcs",
        "phpunit",
    ],
};

/// proto and moon. Its Nix module is `home/suites/proto.nix`. proto is a version manager itself, so
/// the Suite has no versions and no Project pins: a project pins its tools, moon and proto among
/// them, in its `.prototools`.
const PROTO: Suite = Suite {
    name: "proto",
    title: "proto + moon",
    description: "proto, the toolchain manager that pins each project's tools in its .prototools, and moon, the monorepo build system, which proto installs on first use.",
    versions: &[],
    extras: &[],
    toolchain: Toolchain::Upstream { manager: "proto" },
    caches: &[Cache {
        path: ".local/share/proto",
        description: "The tools, shims and plugins proto installed, moon among them.",
    }],
    scaffolders: &[Scaffolder {
        id: "moon",
        label: "New moon workspace",
        argv: &["kubaos-moon-new", "{path}"],
    }],
    seeds: &[],
    mentions: &["proto", "moon", "moonrepo", ".prototools", "moon.yml"],
};

/// A Suite with one of everything, for the test harness and the unit tests. Its Nix module is
/// `home/suites/test.nix`. Version 3 is never installed by a check, so a Project pin of it shows
/// a toolchain fetched on first use.
#[cfg(any(test, feature = "test-suite"))]
pub const TEST: Suite = Suite {
    name: "test",
    title: "Test",
    description: "KubaOS's own test Suite: a script, an environment variable, a PATH entry and a Neovim setting.",
    versions: &["1", "2", "3"],
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
