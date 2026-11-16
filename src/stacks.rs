use std::path::Path;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Language stack presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stack {
    Rust,
    Typescript,
    Python,
    Go,
}

impl Stack {
    pub fn display_name(&self) -> &'static str {
        match self {
            Stack::Rust => "Rust",
            Stack::Typescript => "TypeScript",
            Stack::Python => "Python",
            Stack::Go => "Go",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Stack::Rust => "rust",
            Stack::Typescript => "typescript",
            Stack::Python => "python",
            Stack::Go => "go",
        }
    }

    pub fn parse(s: &str) -> Option<Stack> {
        match s {
            "rust" => Some(Stack::Rust),
            "typescript" => Some(Stack::Typescript),
            "python" => Some(Stack::Python),
            "go" => Some(Stack::Go),
            _ => None,
        }
    }

    /// Detect a stack from build files present in `dir`.
    pub fn detect(dir: &Path) -> Option<Stack> {
        if dir.join("Cargo.toml").is_file() {
            return Some(Stack::Rust);
        }
        if dir.join("package.json").is_file() {
            return Some(Stack::Typescript);
        }
        if dir.join("pyproject.toml").is_file() || dir.join("setup.py").is_file() {
            return Some(Stack::Python);
        }
        if dir.join("go.mod").is_file() {
            return Some(Stack::Go);
        }
        None
    }

    /// Markdown command list for the AGENTS.md "Commands" section.
    pub fn commands(&self) -> &'static str {
        match self {
            Stack::Rust => {
                "\
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Run: `cargo run`"
            }
            Stack::Typescript => {
                "\
- Install: `npm install`
- Build: `npm run build`
- Test: `npm test`
- Lint: `npm run lint`
- Run: `npm start`"
            }
            Stack::Python => {
                "\
- Install: `pip install -e .`
- Test: `pytest`
- Lint: `ruff check .`
- Run: `python -m {{name}}`"
            }
            Stack::Go => {
                "\
- Build: `go build ./...`
- Test: `go test ./...`
- Lint: `golangci-lint run`
- Run: `go run .`"
            }
        }
    }

    /// Recipe bodies for the justfile (tab-indented).
    pub fn justfile_commands(&self) -> &'static str {
        match self {
            Stack::Rust => {
                "\
build:
\tcargo build

test:
\tcargo test

lint:
\tcargo clippy -- -D warnings

run:
\tcargo run"
            }
            Stack::Typescript => {
                "\
install:
\tnpm install

build:
\tnpm run build

test:
\tnpm test

lint:
\tnpm run lint

run:
\tnpm start"
            }
            Stack::Python => {
                "\
install:
\tpip install -e .

test:
\tpytest

lint:
\truff check .

run:
\tpython -m {{name}}"
            }
            Stack::Go => {
                "\
build:
\tgo build ./...

test:
\tgo test ./...

lint:
\tgolangci-lint run

run:
\tgo run ."
            }
        }
    }

    /// Stack-specific .gitignore content.
    pub fn gitignore(&self) -> &'static str {
        match self {
            Stack::Rust => "/target\n",
            Stack::Typescript => "node_modules/\ndist/\n*.log\n.env\n",
            Stack::Python => {
                "__pycache__/\n*.py[cod]\n.venv/\nbuild/\ndist/\n*.egg-info/\n.pytest_cache/\n"
            }
            Stack::Go => "{{name}}\n*.test\n/out/\n",
        }
    }

    /// Extra build-config stub files: (path, content template).
    pub fn build_stubs(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Stack::Rust => vec![(
                "Cargo.toml",
                "[package]\nname = \"{{name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
            )],
            Stack::Typescript => vec![
                (
                    "package.json",
                    "{\n  \"name\": \"{{name}}\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"scripts\": {\n    \"build\": \"tsc\",\n    \"test\": \"echo \\\"Error: no test specified\\\" && exit 1\",\n    \"lint\": \"eslint .\",\n    \"start\": \"node dist/index.js\"\n  }\n}\n",
                ),
                (
                    "tsconfig.json",
                    "{\n  \"compilerOptions\": {\n    \"target\": \"ES2022\",\n    \"module\": \"commonjs\",\n    \"rootDir\": \"src\",\n    \"outDir\": \"dist\",\n    \"strict\": true,\n    \"esModuleInterop\": true,\n    \"skipLibCheck\": true\n  },\n  \"include\": [\"src\"]\n}\n",
                ),
            ],
            Stack::Python => vec![(
                "pyproject.toml",
                "[project]\nname = \"{{name}}\"\nversion = \"0.1.0\"\ndescription = \"{{description}}\"\nrequires-python = \">=3.10\"\n\n[build-system]\nrequires = [\"setuptools>=68\"]\nbuild-backend = \"setuptools.build_meta\"\n",
            )],
            Stack::Go => vec![(
                "go.mod",
                "module {{name}}\n\ngo 1.22\n",
            )],
        }
    }
}
