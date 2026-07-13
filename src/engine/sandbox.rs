//! Filesystem/syscall sandbox applied by the PDF worker subprocesses (see
//! `engine::pdf_worker`) before they touch any untrusted PDF bytes. All
//! poppler parsing/rendering happens in one of these workers - never in the
//! main GTK process - so a memory-corruption bug in poppler can't do
//! anything with the app's own privileges.
//!
//! Best-effort: an older kernel without Landlock/seccomp support just runs
//! the worker unsandboxed rather than refusing to work at all (this is a
//! defense-in-depth layer, not the only thing standing between untrusted
//! input and the rest of the system).

use std::path::Path;

use landlock::{
    Access, AccessFs, CompatLevel, Compatible, PathBeneath, PathFd, Ruleset, RulesetAttr,
    RulesetCreatedAttr, ABI,
};

/// Applies both layers. `write_dirs` are the only directories the process
/// may create/write files in afterwards (typically just an export
/// destination's parent directory); everything else stays read-only.
pub fn apply(write_dirs: &[&Path]) {
    apply_landlock(write_dirs);
    apply_seccomp();
}

/// Read-only filesystem access everywhere (fonts, shared libs, the config
/// dir, etc. all still need to be readable), no execute, plus write access
/// to each of `write_dirs`. Landlock rules can only ever get *more*
/// restrictive within a process, never lifted - exactly right for a worker
/// that does one job and exits, wrong for the long-lived main process,
/// which is why this only ever runs in a worker.
fn apply_landlock(write_dirs: &[&Path]) {
    let abi = ABI::V5;
    let build = || -> anyhow::Result<()> {
        let mut ruleset = Ruleset::default()
            .set_compatibility(CompatLevel::BestEffort)
            .handle_access(AccessFs::from_all(abi))?
            .create()?
            .add_rule(PathBeneath::new(
                PathFd::new("/")?,
                AccessFs::ReadFile | AccessFs::ReadDir,
            ))?;
        for dir in write_dirs {
            ruleset = ruleset.add_rule(PathBeneath::new(PathFd::new(*dir)?, AccessFs::from_write(abi)))?;
        }
        ruleset.restrict_self()?;
        Ok(())
    };
    if let Err(e) = build() {
        eprintln!("[pdf worker] landlock sandbox not applied, continuing unsandboxed: {e}");
    }
}

/// Blocks a small, deliberately conservative set of syscalls that a PDF
/// renderer has zero legitimate use for and that GTK/GLib/Cairo internals
/// never call either (unlike e.g. `clone`/`socket`, which threading and
/// Wayland/D-Bus plumbing may need - those are left alone so a mistake here
/// can't silently break rendering). This specifically closes off the most
/// common post-exploitation moves: spawning a process, ptrace-based
/// injection, and kernel/module tampering.
fn apply_seccomp() {
    use seccompiler::{BpfProgram, SeccompAction, SeccompFilter};
    use std::collections::BTreeMap;
    use std::convert::TryInto;

    let blocked: &[i64] = &[
        libc::SYS_execve,
        libc::SYS_execveat,
        libc::SYS_ptrace,
        libc::SYS_process_vm_readv,
        libc::SYS_process_vm_writev,
        libc::SYS_kexec_load,
        libc::SYS_reboot,
        libc::SYS_mount,
        libc::SYS_umount2,
        libc::SYS_pivot_root,
        libc::SYS_swapon,
        libc::SYS_swapoff,
        libc::SYS_init_module,
        libc::SYS_finit_module,
        libc::SYS_delete_module,
    ];
    let rules: BTreeMap<i64, Vec<seccompiler::SeccompRule>> = blocked.iter().map(|&nr| (nr, vec![])).collect();

    let build = || -> anyhow::Result<BpfProgram> {
        let arch = std::env::consts::ARCH.try_into().map_err(|e| anyhow::anyhow!("{e}"))?;
        let filter = SeccompFilter::new(rules, SeccompAction::Allow, SeccompAction::Errno(libc::EPERM as u32), arch)?;
        filter.try_into().map_err(|e: seccompiler::BackendError| anyhow::anyhow!("{e}"))
    };
    match build() {
        Ok(program) => {
            if let Err(e) = seccompiler::apply_filter(&program) {
                eprintln!("[pdf worker] seccomp filter not applied, continuing without it: {e}");
            }
        }
        Err(e) => eprintln!("[pdf worker] could not build seccomp filter, continuing without it: {e:#}"),
    }
}

/// Proves the sandbox actually blocks something, rather than just not
/// warning on startup: applies it with no write grants, then tries a
/// forbidden write, a forbidden exec, and (as a control showing the sandbox
/// isn't *overly* restrictive) a plain read - one line per check on stdout,
/// so a test in the unsandboxed main process can assert on the outcome.
pub fn run_selftest() -> ! {
    apply(&[]);

    let temp_path = std::env::temp_dir().join(format!("inkpdf-sandbox-selftest-{}", std::process::id()));
    let write_result = std::fs::write(&temp_path, b"x");
    if write_result.is_ok() {
        let _ = std::fs::remove_file(&temp_path);
    }
    println!("write: {}", if write_result.is_ok() { "allowed" } else { "blocked" });

    let exec_result = std::process::Command::new("/bin/true").status();
    println!("exec: {}", if exec_result.is_ok() { "allowed" } else { "blocked" });

    let read_result = std::fs::read("/etc/hostname").or_else(|_| std::fs::read("/proc/version"));
    println!("read: {}", if read_result.is_ok() { "allowed" } else { "blocked" });

    std::process::exit(0);
}
