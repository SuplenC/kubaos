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
use super::Request;
use super::Scaffolder;
use super::Seed;
use super::Suite;
use super::Tool;
use super::Toolchain;

/// The Suites KubaOS ships, by name.
pub const SHIPPED: &[Suite] = &[RUST, GO, NODE, PYTHON, PHP, PROTO, WALLPAPER_DEPTH];

/// Node from mise, with Bun (the `bun` extra) and vtsls, each its own mise tool. Its Nix module is
/// `home/suites/node.nix`.
///
/// Node and Bun are their projects' release binaries; vtsls ships only on npm, so mise's `npm:`
/// backend installs it, running the Node it installs beside it.
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
    mise: &[
        Tool {
            name: "node",
            request: Request::Suite,
            extra: None,
        },
        Tool {
            name: "npm:@vtsls/language-server",
            request: Request::Fixed("0.3"),
            extra: None,
        },
        Tool {
            name: "bun",
            request: Request::Fixed("1.4"),
            extra: Some("bun"),
        },
    ],
    toolchain: Toolchain::Mise,
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
            path: ".local/share/bun",
            description: "The packages `bun add --global` installed, and Bun's download cache.",
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

/// Python through uv, with basedpyright, ruff and debugpy, each its own mise tool. Its Nix module
/// is `home/suites/python.nix`.
///
/// The Suite has no versions: uv installs each project's Python, from its `.python-version` and
/// `requires-python`. uv and ruff are release binaries from aqua. basedpyright and debugpy publish
/// none, so mise's `pypi:` backend installs each into a venv of its own with the Suite's uv, on the
/// Python uv finds (the system's, or a Python uv installed); basedpyright's wheel brings its own
/// Node. mise finds that uv only in the same install or in the fragment, so uv stays in the
/// Suite's tools and a `pypi:` tool is never installed without it.
const PYTHON: Suite = Suite {
    name: "python",
    title: "Python",
    description: "uv for Pythons and venvs, basedpyright, ruff and debugpy, set up in Neovim.",
    versions: &[],
    extras: &[],
    mise: &[
        Tool {
            name: "aqua:astral-sh/uv",
            request: Request::Fixed("0.13"),
            extra: None,
        },
        Tool {
            name: "pypi:basedpyright",
            request: Request::Fixed("1.40"),
            extra: None,
        },
        Tool {
            name: "aqua:astral-sh/ruff",
            request: Request::Fixed("0.17"),
            extra: None,
        },
        Tool {
            name: "pypi:debugpy",
            request: Request::Fixed("1.8"),
            extra: None,
        },
    ],
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
    mise: &[],
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

/// proto from mise, and moon through proto. Its Nix module is `home/suites/proto.nix`.
///
/// proto is its project's release binary, from aqua. It is a version manager itself, so the Suite
/// has no versions and no Project pins: a project pins its tools, moon and proto among them, in
/// its `.prototools`, and proto installs them into its own home.
const PROTO: Suite = Suite {
    name: "proto",
    title: "proto + moon",
    description: "proto, the toolchain manager that pins each project's tools in its .prototools, and moon, the monorepo build system, which proto installs for each workspace.",
    versions: &[],
    extras: &[],
    mise: &[Tool {
        name: "aqua:moonrepo/proto",
        request: Request::Fixed("0.63"),
        extra: None,
    }],
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

/// Go from mise, with gopls, gofumpt, goimports, golangci-lint and delve, each its own mise tool.
/// Its Nix module is `home/suites/go.nix`.
///
/// gofumpt, golangci-lint and delve are release binaries from aqua. gopls and goimports publish
/// none, so mise's `go:` backend builds them with the Go it installs beside them. One that needs a
/// newer Go than the chosen line to build (gopls 0.24 needs 1.27) has Go fetch that toolchain for
/// the build, which `GOTOOLCHAIN=local` refuses: KubaOS leaves `GOTOOLCHAIN` unset.
const GO: Suite = Suite {
    name: "go",
    title: "Go",
    description: "Go with gopls, gofumpt, goimports, golangci-lint and delve, and Neovim's Go support.",
    versions: &["1.27", "1.26", "1.25"],
    extras: &[],
    mise: &[
        Tool {
            name: "go",
            request: Request::Suite,
            extra: None,
        },
        Tool {
            name: "go:golang.org/x/tools/gopls",
            request: Request::Fixed("0.24"),
            extra: None,
        },
        Tool {
            name: "go:golang.org/x/tools/cmd/goimports",
            request: Request::Fixed("0.52"),
            extra: None,
        },
        Tool {
            name: "aqua:mvdan/gofumpt",
            request: Request::Fixed("0.12"),
            extra: None,
        },
        Tool {
            name: "aqua:golangci/golangci-lint",
            request: Request::Fixed("2.15"),
            extra: None,
        },
        Tool {
            name: "aqua:go-delve/delve",
            request: Request::Fixed("1.27"),
            extra: None,
        },
    ],
    toolchain: Toolchain::Mise,
    caches: &[
        Cache {
            path: "go",
            description: "Go's modules and what `go install` built (GOPATH).",
        },
        Cache {
            path: ".cache/go-build",
            description: "Go's build cache.",
        },
        Cache {
            path: ".cache/gopls",
            description: "gopls's index of your modules.",
        },
        Cache {
            path: ".cache/golangci-lint",
            description: "golangci-lint's cache.",
        },
    ],
    scaffolders: &[Scaffolder {
        id: "module",
        label: "New Go project",
        argv: &["kubaos-go-new", "{path}", "{name}"],
    }],
    seeds: &[Seed {
        app: App::Nvim,
        path: "plugins/go.lua",
        contents: include_str!("seeds/go.lua"),
    }],
    mentions: &[
        "golang",
        "gopls",
        "gofumpt",
        "goimports",
        "golangci-lint",
        "delve",
        "lang.go",
        "gopath",
        "gotoolchain",
    ],
};

/// Rust through rustup, with rust-analyzer, clippy, bacon, cargo-nextest and sccache, linked by
/// clang with mold. Its Nix module is `home/suites/rust.nix`.
///
/// It has no versions: rustup installs the toolchain a project's `rust-toolchain.toml` names on
/// first use, and stable everywhere else, so that file is the per-project pin. mise installs
/// rustup itself, not mise's own `rust`, which sets `RUSTUP_TOOLCHAIN` and so outranks that file.
/// Every tool is a release binary: aqua's where its registry has one, else the project's GitHub
/// release. codelldb is its VS Code extension, which carries the adapter and its own LLDB.
const RUST: Suite = Suite {
    name: "rust",
    title: "Rust",
    description: "Rust through rustup, with rust-analyzer, clippy, bacon, cargo-nextest and sccache, and Neovim's Rust \
                  support. A project's rust-toolchain.toml picks its toolchain.",
    versions: &[],
    extras: &[Extra {
        name: "wild",
        description: "Links with wild instead of mold: faster relinks, before its 1.0.",
        default: false,
    }],
    mise: &[
        Tool {
            name: "aqua:rust-lang/rustup",
            request: Request::Fixed("1.29"),
            extra: None,
        },
        Tool {
            name: "aqua:nextest-rs/nextest/cargo-nextest",
            request: Request::Fixed("0.9"),
            extra: None,
        },
        Tool {
            name: "aqua:mozilla/sccache",
            request: Request::Fixed("0.19"),
            extra: None,
        },
        Tool {
            name: "github:vadimcn/codelldb",
            request: Request::Fixed("1.12"),
            extra: None,
        },
        Tool {
            name: "github:davidlattimore/wild",
            request: Request::Fixed("0.11"),
            extra: Some("wild"),
        },
    ],
    toolchain: Toolchain::Upstream { manager: "rustup" },
    caches: &[
        Cache {
            path: ".rustup",
            description: "The toolchains rustup installed, and its settings.",
        },
        Cache {
            path: ".cargo/registry",
            description: "The crates cargo downloaded.",
        },
        Cache {
            path: ".cargo/git",
            description: "The git dependencies cargo checked out.",
        },
        Cache {
            path: ".cache/sccache",
            description: "sccache's compiled crates.",
        },
    ],
    scaffolders: &[
        Scaffolder {
            id: "bin",
            label: "New Rust project",
            argv: &["cargo", "new", "--quiet", "{path}"],
        },
        Scaffolder {
            id: "lib",
            label: "New Rust library",
            argv: &["cargo", "new", "--quiet", "--lib", "{path}"],
        },
        Scaffolder {
            id: "workspace",
            label: "New Rust workspace",
            argv: &["kubaos-rust-workspace", "{path}", "{name}"],
        },
    ],
    seeds: &[Seed {
        app: App::Nvim,
        path: "plugins/rust.lua",
        contents: include_str!("seeds/rust.lua"),
    }],
    mentions: &[
        "rust",
        "rustup",
        "rustc",
        "cargo",
        "rust-analyzer",
        "rust_analyzer",
        "rustaceanvim",
        "sccache",
    ],
};

/// The cut-out the lock draws over its clock, for the user's own wallpapers. Its Nix module is
/// `home/suites/wallpaper-depth.nix`.
const WALLPAPER_DEPTH: Suite = Suite {
    name: "wallpaper-depth",
    title: "Wallpaper depth",
    description: "Cuts the foreground out of each wallpaper of your own as you set it, so the lock's clock stands behind it; with its models, about 700 MB.",
    versions: &[],
    extras: &[],
    mise: &[],
    toolchain: Toolchain::Nix,
    caches: &[Cache {
        path: ".cache/kubaos/wallpapers",
        description: "The cut-outs and clock places made for your own wallpapers.",
    }],
    scaffolders: &[],
    seeds: &[],
    mentions: &["wallpaper-depth", "kubaos-depth", "cutout"],
};

/// A Suite with one of everything, for the test harness and the unit tests. Its Nix module is
/// `home/suites/test.nix`. Version 3 is never installed by a check, so a Project pin of it shows
/// a toolchain fetched on first use. Its mise tool, `kubaos-test-tool`, comes from the asdf plugin
/// in `test/mise/`, which the harness links into mise first.
#[cfg(any(test, feature = "test-suite"))]
pub const TEST: Suite = Suite {
    name: "test",
    title: "Test",
    description: "KubaOS's own test Suite: a script, a mise tool, an environment variable, a PATH entry and a Neovim setting.",
    versions: &["1", "2", "3"],
    extras: &[Extra {
        name: "greeting",
        description: "Adds kubaos-test-suite-greet.",
        default: false,
    }],
    mise: &[Tool {
        name: "kubaos-test-tool",
        request: Request::Suite,
        extra: None,
    }],
    toolchain: Toolchain::Mise,
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
