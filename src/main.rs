use anyhow::{anyhow, Context, Result};
use clap::Parser;
use sha2::{
    digest::{generic_array::GenericArray, typenum::U32},
    Digest,
};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::{Path, PathBuf},
};

mod spawn_exec;

use spawn_exec::SpawnExec;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, rename_all="snake_case")]
struct Args {
    #[arg(short, long)]
    execution_log_json_file: Vec<PathBuf>,
}

fn spawn_exec_iter(path: &Path) -> Result<impl Iterator<Item = Result<SpawnExec>>> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    Ok(SpawnExecIter {
        lines: buf_reader.lines(),
        buf: vec![],
        reached_end: false,
    })
}

struct SpawnExecIter<B> {
    lines: Lines<B>,
    buf: Vec<u8>,
    reached_end: bool,
}

impl<B: BufRead> Iterator for SpawnExecIter<B> {
    type Item = Result<SpawnExec>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reached_end {
            return None;
        }

        let mut push_open_bracket = false;
        while let Some(line) = self.lines.next() {
            match line {
                Ok(line) => {
                    if line == "}{" {
                        self.buf.push('}' as u8);
                        push_open_bracket = true;
                        break;
                    } else if line == "}" {
                        self.reached_end = true;
                        self.buf.push('}' as u8);
                        break;
                    } else {
                        self.buf.extend_from_slice(line.as_bytes());
                        self.buf.push('\n' as u8);
                    }
                }
                Err(e) => return Some(Err(e.into())),
            }
        }

        let spawn_exec = match serde_json::from_slice(&self.buf) {
            Ok(spawn_exec) => spawn_exec,
            Err(e) => {
                return Some(Err(anyhow!(e)).with_context(|| {
                    format!(
                        "Failed to parse json {}",
                        std::str::from_utf8(&self.buf).unwrap_or("<malformed>")
                    )
                }));
            }
        };

        self.buf.clear();

        if push_open_bracket {
            self.buf.push('{' as u8);
        }

        Some(Ok(spawn_exec))
    }
}

fn outputs_are_same(a: &[spawn_exec::File], b: &[spawn_exec::File]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for (a, b) in a.iter().zip(b.iter()) {
        if a.path != b.path {
            return false;
        }

        if a.digest.hash != b.digest.hash {
            return false;
        }
    }

    true
}

type Sha256 = GenericArray<u8, U32>;

impl SpawnExec {
    pub fn digest(&self) -> Sha256 {
        if let Some(ref digest) = self.digest {
            let digest = hex::decode(digest.hash.as_bytes()).expect("invalid digest");
            GenericArray::clone_from_slice(&digest)
        } else {
            let mut hasher = sha2::Sha256::new();
            for arg in self.command_args.iter() {
                hasher.update(arg.as_bytes());
            }
            for env in self.environment_variables.iter() {
                hasher.update(env.name.as_bytes());
                hasher.update(env.value.as_bytes());
            }
            if let Some(ref platform) = self.platform {
                for prop in platform.properties.iter() {
                    hasher.update(prop.name.as_bytes());
                    hasher.update(prop.value.as_bytes());
                }
            }
            for input in self.inputs.iter() {
                hasher.update(input.path.as_bytes());
                let digest = hex::decode(input.digest.hash.as_bytes()).expect("invalid digest");
                hasher.update(digest);
            }
            hasher.finalize()
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut digest_to_spawn: HashMap<Sha256, SpawnExec> = HashMap::new();
    let mut non_deterministics_spawns: HashMap<Sha256, [SpawnExec; 2]> = HashMap::new();

    for execution_log in args.execution_log_json_file.iter() {
        for spawn_exec in spawn_exec_iter(&execution_log)? {
            let spawn_exec = spawn_exec?;

            let digest = spawn_exec.digest();

            if let Some(old_spawn) = digest_to_spawn.get(&digest) {
                if !outputs_are_same(&old_spawn.actual_outputs, &spawn_exec.actual_outputs) {
                    if !non_deterministics_spawns.contains_key(&digest) {
                        non_deterministics_spawns.insert(digest, [old_spawn.clone(), spawn_exec]);
                    }
                }
            } else {
                digest_to_spawn.insert(digest, spawn_exec);
            }
        }
    }

    if non_deterministics_spawns.is_empty() {
        println!("No non-deterministic actions found!")
    } else {
        for [a, b] in non_deterministics_spawns.values() {
            let a_json = serde_json::to_string_pretty(a).unwrap();
            let b_json = serde_json::to_string_pretty(b).unwrap();
            let diff = diffy::create_patch(&a_json, &b_json);
            println!(
                "Outputs of the same action `{}` are different:",
                &a.progress_message
            );
            println!("{}", diff);
        }
    }

    Ok(())
}
