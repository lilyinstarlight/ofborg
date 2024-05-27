use crate::files::file_to_str;
use crate::nix;

use std::path::PathBuf;

use tracing::{info, warn};

enum StdenvFrom {
    Before,
    After,
}

#[derive(Debug)]
pub enum System {
    X8664Darwin,
    AArch64Darwin,
    X8664Linux,
    AArch64Linux,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Stdenvs {
    nix: nix::Nix,
    co: PathBuf,

    x86_64_linux_stdenv_before: Option<String>,
    x86_64_linux_stdenv_after: Option<String>,
    aarch64_linux_stdenv_before: Option<String>,
    aarch64_linux_stdenv_after: Option<String>,

    x86_64_darwin_stdenv_before: Option<String>,
    x86_64_darwin_stdenv_after: Option<String>,
    aarch64_darwin_stdenv_before: Option<String>,
    aarch64_darwin_stdenv_after: Option<String>,
}

impl Stdenvs {
    pub fn new(nix: nix::Nix, co: PathBuf) -> Stdenvs {
        Stdenvs {
            nix,
            co,

            x86_64_linux_stdenv_before: None,
            x86_64_linux_stdenv_after: None,

            aarch64_linux_stdenv_before: None,
            aarch64_linux_stdenv_after: None,

            x86_64_darwin_stdenv_before: None,
            x86_64_darwin_stdenv_after: None,

            aarch64_darwin_stdenv_before: None,
            aarch64_darwin_stdenv_after: None,
        }
    }

    pub fn identify_before(&mut self) {
        self.identify(System::X8664Linux, StdenvFrom::Before);
        self.identify(System::AArch64Linux, StdenvFrom::Before);
        self.identify(System::X8664Darwin, StdenvFrom::Before);
        self.identify(System::AArch64Darwin, StdenvFrom::Before);
    }

    pub fn identify_after(&mut self) {
        self.identify(System::X8664Linux, StdenvFrom::After);
        self.identify(System::AArch64Linux, StdenvFrom::After);
        self.identify(System::X8664Darwin, StdenvFrom::After);
        self.identify(System::AArch64Darwin, StdenvFrom::After);
    }

    pub fn are_same(&self) -> bool {
        self.changed().is_empty()
    }

    pub fn changed(&self) -> Vec<System> {
        let mut changed: Vec<System> = vec![];

        if self.x86_64_linux_stdenv_before != self.x86_64_linux_stdenv_after {
            changed.push(System::X8664Linux);
        }

        if self.aarch64_linux_stdenv_before != self.aarch64_linux_stdenv_after {
            changed.push(System::X8664Linux);
        }

        if self.x86_64_darwin_stdenv_before != self.x86_64_darwin_stdenv_after {
            changed.push(System::X8664Darwin);
        }

        if self.aarch64_darwin_stdenv_before != self.aarch64_darwin_stdenv_after {
            changed.push(System::X8664Darwin);
        }

        changed
    }

    fn identify(&mut self, system: System, from: StdenvFrom) {
        match (system, from) {
            (System::X8664Linux, StdenvFrom::Before) => {
                self.x86_64_linux_stdenv_before = self.evalstdenv("x86_64-linux");
            }
            (System::X8664Linux, StdenvFrom::After) => {
                self.x86_64_linux_stdenv_after = self.evalstdenv("x86_64-linux");
            }

            (System::AArch64Linux, StdenvFrom::Before) => {
                self.aarch64_linux_stdenv_before = self.evalstdenv("aarch64-linux");
            }
            (System::AArch64Linux, StdenvFrom::After) => {
                self.aarch64_linux_stdenv_after = self.evalstdenv("aarch64-linux");
            }

            (System::X8664Darwin, StdenvFrom::Before) => {
                self.x86_64_darwin_stdenv_before = self.evalstdenv("x86_64-darwin");
            }
            (System::X8664Darwin, StdenvFrom::After) => {
                self.x86_64_darwin_stdenv_after = self.evalstdenv("x86_64-darwin");
            }

            (System::AArch64Darwin, StdenvFrom::Before) => {
                self.aarch64_darwin_stdenv_before = self.evalstdenv("aarch64-darwin");
            }
            (System::AArch64Darwin, StdenvFrom::After) => {
                self.aarch64_darwin_stdenv_after = self.evalstdenv("aarch64-darwin");
            }
        }
    }

    /// This is used to find out what the output path of the stdenv for the
    /// given system.
    fn evalstdenv(&self, system: &str) -> Option<String> {
        info!(?system, "query stdenv output");
        let result = self.nix.with_system(system.to_owned()).safely(
            &nix::Operation::QueryPackagesOutputs,
            &self.co,
            vec![
                String::from("-f"),
                String::from("."),
                String::from("-A"),
                String::from("stdenv"),
            ],
            true,
        );

        match result {
            Ok(mut out) => Some(file_to_str(&mut out)),
            Err(mut out) => {
                warn!("{:?}", file_to_str(&mut out));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::env;
    use std::process::Command;

    #[test]
    fn stdenv_checking() {
        let output = Command::new("nix-instantiate")
            .args(&["--eval", "-E", "<nixpkgs>"])
            .output()
            .expect("nix-instantiate required");

        let nixpkgs = String::from_utf8(output.stdout).expect("nixpkgs required");

        let remote = env::var("NIX_REMOTE").unwrap_or("".to_owned());
        let nix = nix::Nix::new(String::from("x86_64-linux"), remote, 1200, None);
        let mut stdenv = Stdenvs::new(nix, PathBuf::from(nixpkgs.trim_end()));
        stdenv.identify(System::X8664Linux, StdenvFrom::Before);
        stdenv.identify(System::AArch64Linux, StdenvFrom::Before);
        stdenv.identify(System::X8664Darwin, StdenvFrom::Before);
        stdenv.identify(System::AArch64Darwin, StdenvFrom::Before);

        stdenv.identify(System::X8664Linux, StdenvFrom::After);
        stdenv.identify(System::AArch64Linux, StdenvFrom::After);
        stdenv.identify(System::X8664Darwin, StdenvFrom::After);
        stdenv.identify(System::AArch64Darwin, StdenvFrom::After);

        assert!(stdenv.are_same());
    }
}
