use crate::ptrace;
use crate::utils;

use super::Replacer;

use std::path::{Path, PathBuf};
use std::{collections::HashMap, fmt::Debug};
use std::fs::read_link;

use anyhow::{anyhow, Result};

use tracing::{info, trace, error};

use procfs::process::all_processes;

#[derive(Debug)]
pub struct CwdReplacer {
    processes: Vec<ptrace::TracedProcess>,
    new_path: PathBuf,
}

impl CwdReplacer {
    #[tracing::instrument(skip(detect_path, new_path))]
    pub fn prepare<P1: AsRef<Path>, P2: AsRef<Path>>(
        detect_path: P1,
        new_path: P2,
    ) -> Result<CwdReplacer> {
        info!("preparing cmdreplacer");

        let processes = all_processes()?.into_iter().filter_map(|process| -> Option<_> {
            let pid = process.pid;
            trace!("itering proc: {}", pid);

            match process.cwd() {
                Ok(cwd) => Some((pid, cwd)),
                Err(err) => {
                    trace!("filter out pid({}) because of error: {:?}", pid, err);
                    None
                }
            }
        }).filter(|(_, path)| {
            path.starts_with(detect_path.as_ref())
        }).filter_map(|(pid, _)| {
            match ptrace::TracedProcess::trace(pid) {
                Ok(process) => Some(process),
                Err(err) => {
                    error!("fail to ptrace process: pid({}) with error: {:?}", pid, err);
                    None
                }
            }
        }).collect();

        Ok(CwdReplacer {
            processes,
            new_path: new_path.as_ref().to_owned()
        })
    }
}

impl Replacer for CwdReplacer {
    #[tracing::instrument]
    fn run(&mut self) -> Result<()> {
        info!("running cwd replacer");
        for process in self.processes.iter() {
            process.chdir(&self.new_path)?;
        }

        Ok(())
    }
}