//! T3 — the hermetic cell runner (ARCHITECTURE.md §3 step 4, §6).
//!
//! v1 hermeticity is **environment hygiene + the determinism meta-check**, not a full
//! sandbox: the child gets a cleared environment (only declared variables, a fixed
//! hygiene set, and `ARRAY_TEST_SEED`), its output is hash-committed, and
//! [`run_cell_checked`] runs every cell twice — any nondeterminism the environment
//! can't prevent (network, wall-clock reads, uninitialized memory) shows up as an
//! evidence-hash mismatch and quarantines the cell. Memory caps and network isolation
//! are deferred to T3b (s3 research report, R-g); until then the determinism claim per
//! cell is "meta-checked", not "sandbox-guaranteed".

use crate::hash::{domain, Hash};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Everything needed to execute one cell. `command[0]` is the program; the runner does
/// not involve a shell.
#[derive(Debug, Clone)]
pub struct CellSpec {
    pub cell_key: Hash,
    pub command: Vec<String>,
    pub cwd: PathBuf,
    /// Declared environment. Only these (plus the hygiene set and `ARRAY_TEST_SEED`)
    /// reach the child; the parent environment never leaks.
    pub env: BTreeMap<String, String>,
    pub seed: u64,
    /// Wall-clock envelope (D10). Breach is `TimedOut`, distinct from `Fail`.
    pub timeout: Duration,
    /// Opt-in memory cap in megabytes, enforced as `RLIMIT_AS` in the child (T3b).
    /// A breach surfaces as allocation failure inside the cell → `Fail`.
    pub mem_limit_mb: Option<u64>,
}

/// The isolation level actually applied to cells in this process (D16). Probed once:
/// if a network namespace can be created, every cell gets a fresh one (loopback only)
/// and pre_exec fails closed; otherwise cells run with env hygiene + the meta-check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    EnvOnly,
    NetIsolated,
}

#[cfg(unix)]
fn netns_flags() -> Option<libc::c_int> {
    static PROBE: std::sync::OnceLock<Option<libc::c_int>> = std::sync::OnceLock::new();
    *PROBE.get_or_init(|| {
        // Try root-style netns first, then unprivileged user+net namespaces. The probe
        // child attempts the unshare itself; spawn failure means "can't".
        for flags in [libc::CLONE_NEWNET, libc::CLONE_NEWUSER | libc::CLONE_NEWNET] {
            let mut cmd = Command::new("/bin/sh");
            cmd.args(["-c", "exit 0"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            unsafe {
                use std::os::unix::process::CommandExt;
                cmd.pre_exec(move || {
                    if libc::unshare(flags) == 0 {
                        Ok(())
                    } else {
                        Err(std::io::Error::last_os_error())
                    }
                });
            }
            if matches!(cmd.status(), Ok(status) if status.success()) {
                return Some(flags);
            }
        }
        None
    })
}

/// Recognized declared-env extension flag (T3c/D25): a cell declaring
/// `ARRAY_TEST_FS_READONLY = "1"` runs with the entire filesystem recursively
/// read-only (fresh mount namespace + `mount_setattr(AT_RECURSIVE, RDONLY)`).
/// Declared env is already inside `test_def_hash`, so the flag re-keys the cell —
/// the per-test extension channel that needs no frozen-layout change.
pub const FS_READONLY_ENV: &str = "ARRAY_TEST_FS_READONLY";

#[cfg(unix)]
mod fs_ro {
    // mount_setattr (Linux 5.12+): the one honest way to make every mount in the
    // namespace read-only recursively. Constants per uapi/linux/mount.h.
    #[repr(C)]
    pub struct MountAttr {
        pub attr_set: u64,
        pub attr_clr: u64,
        pub propagation: u64,
        pub userns_fd: u64,
    }
    pub const MOUNT_ATTR_RDONLY: u64 = 0x1;
    pub const AT_RECURSIVE: libc::c_int = 0x8000;

    /// Make the current mount namespace fully read-only. Caller must already be in a
    /// fresh, private namespace. Fail-closed.
    pub unsafe fn make_root_readonly() -> Result<(), std::io::Error> {
        // Disconnect propagation so the read-only flip cannot leak to the host.
        if libc::mount(
            std::ptr::null(),
            c"/".as_ptr(),
            std::ptr::null(),
            libc::MS_REC | libc::MS_PRIVATE,
            std::ptr::null(),
        ) != 0
        {
            return Err(std::io::Error::last_os_error());
        }
        let attr = MountAttr {
            attr_set: MOUNT_ATTR_RDONLY,
            attr_clr: 0,
            propagation: 0,
            userns_fd: 0,
        };
        let rc = libc::syscall(
            libc::SYS_mount_setattr,
            libc::AT_FDCWD,
            c"/".as_ptr(),
            AT_RECURSIVE,
            &attr as *const MountAttr,
            std::mem::size_of::<MountAttr>(),
        );
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

/// Probe (once) whether read-only cells are supported here: needs a mount namespace
/// (root or user-ns) plus mount_setattr.
#[cfg(unix)]
pub fn fs_readonly_supported() -> bool {
    static PROBE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PROBE.get_or_init(|| {
        for extra in [0, libc::CLONE_NEWUSER] {
            let mut cmd = Command::new("/bin/sh");
            cmd.args(["-c", "! touch /tmp/.array-test-ro-probe 2>/dev/null"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            unsafe {
                use std::os::unix::process::CommandExt;
                cmd.pre_exec(move || {
                    if libc::unshare(libc::CLONE_NEWNS | extra) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    fs_ro::make_root_readonly()
                });
            }
            if matches!(cmd.status(), Ok(status) if status.success()) {
                return true;
            }
        }
        false
    })
}

#[cfg(not(unix))]
pub fn fs_readonly_supported() -> bool {
    false
}

/// The isolation level cells in this process actually get (recorded per confirmation).
pub fn isolation_level() -> IsolationLevel {
    #[cfg(unix)]
    {
        if netns_flags().is_some() {
            return IsolationLevel::NetIsolated;
        }
    }
    IsolationLevel::EnvOnly
}

/// Fixed hygiene variables set for every cell, before declared `env` (which may
/// override them deliberately — a declared override is part of the test definition).
const HYGIENE_ENV: &[(&str, &str)] = &[("TZ", "UTC"), ("LC_ALL", "C"), ("SOURCE_DATE_EPOCH", "0")];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Pass,
    Fail { exit_code: Option<i32> },
    TimedOut,
}

/// Raw captured output. `evidence_hash` commits to all three fields with length
/// framing, so no boundary ambiguity between streams.
#[derive(Debug, Clone)]
pub struct Evidence {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: Option<i32>,
}

impl Evidence {
    /// The canonical framed encoding — the exact bytes `evidence_hash` covers, and the
    /// exact bytes the evidence store persists (s7 §2.5): anyone can re-hash the stored
    /// file and check it against the ledger.
    pub fn framed(&self) -> Vec<u8> {
        let mut framed = Vec::with_capacity(self.stdout.len() + self.stderr.len() + 25);
        framed.extend_from_slice(&(self.stdout.len() as u64).to_le_bytes());
        framed.extend_from_slice(&self.stdout);
        framed.extend_from_slice(&(self.stderr.len() as u64).to_le_bytes());
        framed.extend_from_slice(&self.stderr);
        // None (killed by signal / no exit code) encodes as i32::MIN — a value no real
        // process exit can produce, so the sentinel cannot collide with a status.
        framed.extend_from_slice(&(self.exit_code.unwrap_or(i32::MIN) as i64).to_le_bytes());
        framed
    }

    pub fn hash(&self) -> Hash {
        Hash::leaf(domain::EVIDENCE, &self.framed())
    }
}

#[derive(Debug)]
pub struct RunOutcome {
    pub status: RunStatus,
    pub evidence: Evidence,
    pub evidence_hash: Hash,
    pub duration: Duration,
}

/// Result of the determinism meta-check (§6): the cell ran twice; either both runs
/// agreed byte-for-byte, or the cell is quarantined with BOTH runs carried in full —
/// the whole meaning of quarantine is "these disagreed", so both transcripts are
/// evidence (F9).
#[derive(Debug)]
pub enum Verdict {
    Confirmed(RunOutcome),
    Quarantined {
        first: Box<RunOutcome>,
        second: Box<RunOutcome>,
    },
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("empty command for cell {cell_key}")]
    EmptyCommand { cell_key: Hash },
    #[error("failed to spawn {program:?}: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("io error while running cell: {0}")]
    Io(#[from] std::io::Error),
}

/// Execute one cell hermetically (v1 level — see module docs) and capture evidence.
pub fn run_cell(spec: &CellSpec) -> Result<RunOutcome, RunError> {
    let program = spec.command.first().ok_or(RunError::EmptyCommand {
        cell_key: spec.cell_key,
    })?;

    let mut cmd = Command::new(program);
    cmd.args(&spec.command[1..])
        .current_dir(&spec.cwd)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in HYGIENE_ENV {
        cmd.env(k, v);
    }
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }
    cmd.env("ARRAY_TEST_SEED", spec.seed.to_string());

    // On unix the cell runs as its own process group so an envelope breach kills the
    // whole tree — killing only the direct child leaves grandchildren holding the
    // output pipes open (and running), which both hangs evidence collection and lets
    // work escape the envelope.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);

        let mem_limit = spec.mem_limit_mb;
        let net_flags = netns_flags();
        // T3c/D25: declared-env opt-in for a recursively read-only filesystem.
        let fs_readonly = spec.env.get(FS_READONLY_ENV).map(String::as_str) == Some("1");
        unsafe {
            cmd.pre_exec(move || {
                if let Some(mb) = mem_limit {
                    let bytes = mb.saturating_mul(1024 * 1024);
                    let limit = libc::rlimit {
                        rlim_cur: bytes,
                        rlim_max: bytes,
                    };
                    if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                // Fail closed (D16): if the host can isolate, a cell that cannot be
                // isolated does not run. Same doctrine for the read-only flag: a cell
                // that declared RO and can't get it does not run.
                let mut flags = net_flags.unwrap_or(0);
                if fs_readonly {
                    flags |= libc::CLONE_NEWNS;
                }
                if flags != 0 && libc::unshare(flags) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if fs_readonly {
                    fs_ro::make_root_readonly()?;
                }
                Ok(())
            });
        }
    }

    let start = Instant::now();
    let mut child = cmd.spawn().map_err(|source| RunError::Spawn {
        program: program.clone(),
        source,
    })?;

    // Drain both pipes on threads so a full pipe buffer can never deadlock the child.
    let mut stdout_pipe = child.stdout.take().expect("stdout was piped");
    let mut stderr_pipe = child.stderr.take().expect("stderr was piped");
    let stdout_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        buf
    });
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        buf
    });

    let mut timed_out = false;
    let exit_status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if start.elapsed() >= spec.timeout {
            timed_out = true;
            #[cfg(unix)]
            unsafe {
                // Negative pid = the whole process group (see process_group(0) above).
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            child.wait()?;
            break None;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let duration = start.elapsed();

    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();
    let exit_code = exit_status.and_then(|s| s.code());

    let status = if timed_out {
        RunStatus::TimedOut
    } else if exit_code == Some(0) {
        RunStatus::Pass
    } else {
        RunStatus::Fail { exit_code }
    };

    let evidence = Evidence {
        stdout,
        stderr,
        exit_code,
    };
    let evidence_hash = evidence.hash();

    Ok(RunOutcome {
        status,
        evidence,
        evidence_hash,
        duration,
    })
}

/// The determinism meta-check: run the cell twice; identical evidence hashes confirm
/// it, a mismatch quarantines it (D10: quarantine is visible state, recorded with both
/// hashes so the divergence is auditable). The confirmed outcome returned is the first
/// run's.
pub fn run_cell_checked(spec: &CellSpec) -> Result<Verdict, RunError> {
    let first = run_cell(spec)?;
    let second = run_cell(spec)?;
    if first.evidence_hash == second.evidence_hash {
        Ok(Verdict::Confirmed(first))
    } else {
        Ok(Verdict::Quarantined {
            first: Box::new(first),
            second: Box::new(second),
        })
    }
}
