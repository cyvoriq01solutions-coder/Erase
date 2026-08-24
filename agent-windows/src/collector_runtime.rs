//! Bounded execution for fixed, application-owned Windows collector commands.
//!
//! The desktop, server, and remote caller boundaries must never pass a command,
//! script, or shell fragment into this module. Collectors may use only reviewed
//! compile-time scripts together with explicit timeout and output limits.

use crate::{CollectorError, CollectorErrorKind, CollectorName, CollectorResult};
use std::{
    io::{self, Read},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
const DEFAULT_MAX_STDOUT_BYTES: usize = 256 * 1024;
const DEFAULT_MAX_STDERR_BYTES: usize = 64 * 1024;
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectorLimits {
    timeout: Duration,
    max_stdout_bytes: usize,
    max_stderr_bytes: usize,
    poll_interval: Duration,
}

impl CollectorLimits {
    #[must_use]
    pub const fn new(
        timeout: Duration,
        max_stdout_bytes: usize,
        max_stderr_bytes: usize,
        poll_interval: Duration,
    ) -> Self {
        Self {
            timeout,
            max_stdout_bytes,
            max_stderr_bytes,
            poll_interval,
        }
    }

    #[must_use]
    pub const fn timeout(self) -> Duration {
        self.timeout
    }

    #[must_use]
    pub const fn max_stdout_bytes(self) -> usize {
        self.max_stdout_bytes
    }

    #[must_use]
    pub const fn max_stderr_bytes(self) -> usize {
        self.max_stderr_bytes
    }

    fn is_valid(self) -> bool {
        !self.timeout.is_zero()
            && self.max_stdout_bytes > 0
            && self.max_stderr_bytes > 0
            && !self.poll_interval.is_zero()
            && self.poll_interval <= self.timeout
    }
}

impl Default for CollectorLimits {
    fn default() -> Self {
        Self::new(
            DEFAULT_TIMEOUT,
            DEFAULT_MAX_STDOUT_BYTES,
            DEFAULT_MAX_STDERR_BYTES,
            DEFAULT_POLL_INTERVAL,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// A compile-time PowerShell script owned and reviewed with the application.
///
/// Never construct this value from frontend, server, file, environment, or network
/// input. The `'static` boundary makes the intended fixed-script policy explicit.
#[derive(Clone, Copy)]
pub struct TrustedPowerShellScript(&'static str);

impl TrustedPowerShellScript {
    #[must_use]
    pub const fn application_owned(script: &'static str) -> Self {
        Self(script)
    }

    const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Successful command output. Stderr is deliberately not returned so routine callers
/// cannot accidentally log firmware strings, identifiers, or other diagnostic data.
pub struct CommandOutput {
    stdout: Vec<u8>,
}

impl CommandOutput {
    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }
}

/// Run a reviewed, fixed PowerShell collector with hard execution and output limits.
///
/// This function is unsupported off Windows and never accepts a runtime-built script.
pub fn run_fixed_powershell(
    collector: CollectorName,
    script: TrustedPowerShellScript,
    limits: CollectorLimits,
    cancellation: &CancellationToken,
) -> CollectorResult<CommandOutput> {
    if !cfg!(target_os = "windows") {
        return Err(collector_error(
            collector,
            CollectorErrorKind::Unsupported,
            "The Windows collector command is unavailable on this operating system.",
        ));
    }

    if script.as_str().trim().is_empty() {
        return Err(collector_error(
            collector,
            CollectorErrorKind::Internal,
            "The collector command is not configured.",
        ));
    }

    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        script.as_str(),
    ]);

    run_command(collector, &mut command, limits, cancellation)
}

fn run_command(
    collector: CollectorName,
    command: &mut Command,
    limits: CollectorLimits,
    cancellation: &CancellationToken,
) -> CollectorResult<CommandOutput> {
    if !limits.is_valid() {
        return Err(collector_error(
            collector,
            CollectorErrorKind::Internal,
            "The collector execution limits are invalid.",
        ));
    }

    if cancellation.is_cancelled() {
        return Err(collector_error(
            collector,
            CollectorErrorKind::Cancelled,
            "The collector was cancelled before execution.",
        ));
    }

    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| {
            collector_error(
                collector,
                CollectorErrorKind::CommandFailed,
                "The collector command could not be started.",
            )
        })?;

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_child(&mut child);
            return Err(collector_error(
                collector,
                CollectorErrorKind::Internal,
                "The collector stdout pipe was unavailable.",
            ));
        }
    };

    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            terminate_child(&mut child);
            return Err(collector_error(
                collector,
                CollectorErrorKind::Internal,
                "The collector stderr pipe was unavailable.",
            ));
        }
    };

    let output_exceeded = Arc::new(AtomicBool::new(false));
    let stdout_reader = spawn_bounded_reader(
        stdout,
        limits.max_stdout_bytes,
        Arc::clone(&output_exceeded),
    );
    let stderr_reader = spawn_bounded_reader(
        stderr,
        limits.max_stderr_bytes,
        Arc::clone(&output_exceeded),
    );

    let started_at = Instant::now();

    let status = loop {
        if cancellation.is_cancelled() {
            return terminate_with_error(
                child,
                stdout_reader,
                stderr_reader,
                collector_error(
                    collector,
                    CollectorErrorKind::Cancelled,
                    "The collector was cancelled.",
                ),
            );
        }

        if output_exceeded.load(Ordering::Acquire) {
            return terminate_with_error(
                child,
                stdout_reader,
                stderr_reader,
                collector_error(
                    collector,
                    CollectorErrorKind::OutputLimitExceeded,
                    "The collector exceeded its output limit.",
                ),
            );
        }

        if started_at.elapsed() >= limits.timeout {
            return terminate_with_error(
                child,
                stdout_reader,
                stderr_reader,
                collector_error(
                    collector,
                    CollectorErrorKind::TimedOut,
                    "The collector exceeded its execution timeout.",
                ),
            );
        }

        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(limits.poll_interval),
            Err(_) => {
                return terminate_with_error(
                    child,
                    stdout_reader,
                    stderr_reader,
                    collector_error(
                        collector,
                        CollectorErrorKind::Internal,
                        "The collector process status could not be read.",
                    ),
                );
            }
        }
    };

    let stdout_result = join_reader(stdout_reader);
    let stderr_result = join_reader(stderr_reader);

    let stdout = stdout_result.map_err(|_| {
        collector_error(
            collector,
            CollectorErrorKind::Internal,
            "The collector stdout could not be read.",
        )
    })?;

    stderr_result.map_err(|_| {
        collector_error(
            collector,
            CollectorErrorKind::Internal,
            "The collector stderr could not be read.",
        )
    })?;

    if output_exceeded.load(Ordering::Acquire) {
        return Err(collector_error(
            collector,
            CollectorErrorKind::OutputLimitExceeded,
            "The collector exceeded its output limit.",
        ));
    }

    if !status.success() {
        return Err(collector_error(
            collector,
            CollectorErrorKind::CommandFailed,
            "The collector command returned an unsuccessful status.",
        ));
    }

    Ok(CommandOutput { stdout })
}

fn spawn_bounded_reader<R>(
    mut reader: R,
    limit: usize,
    output_exceeded: Arc<AtomicBool>,
) -> JoinHandle<io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut output = Vec::with_capacity(limit.min(8 * 1024));
        let mut buffer = [0_u8; 8 * 1024];

        loop {
            let bytes_read = reader.read(&mut buffer)?;

            if bytes_read == 0 {
                return Ok(output);
            }

            let remaining = limit.saturating_sub(output.len());
            let bytes_to_keep = remaining.min(bytes_read);
            output.extend_from_slice(&buffer[..bytes_to_keep]);

            if bytes_to_keep < bytes_read {
                output_exceeded.store(true, Ordering::Release);
            }
        }
    })
}

fn terminate_with_error(
    mut child: Child,
    stdout_reader: JoinHandle<io::Result<Vec<u8>>>,
    stderr_reader: JoinHandle<io::Result<Vec<u8>>>,
    error: CollectorError,
) -> CollectorResult<CommandOutput> {
    terminate_child(&mut child);
    let _ = stdout_reader.join();
    let _ = stderr_reader.join();
    Err(error)
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn join_reader(reader: JoinHandle<io::Result<Vec<u8>>>) -> io::Result<Vec<u8>> {
    reader
        .join()
        .map_err(|_| io::Error::other("collector output reader panicked"))?
}

fn collector_error(
    collector: CollectorName,
    kind: CollectorErrorKind,
    safe_message: &'static str,
) -> CollectorError {
    CollectorError {
        collector,
        kind,
        safe_message: safe_message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{CancellationToken, CollectorLimits, run_command};
    use crate::{CollectorErrorKind, CollectorName};
    use std::{process::Command, time::Duration};

    #[cfg(target_os = "windows")]
    fn test_command(windows_script: &'static str, _unix_script: &'static str) -> Command {
        let mut command = Command::new("powershell.exe");
        command.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            windows_script,
        ]);
        command
    }

    #[cfg(not(target_os = "windows"))]
    fn test_command(_windows_script: &'static str, unix_script: &'static str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", unix_script]);
        command
    }

    fn test_limits(timeout: Duration, max_stdout_bytes: usize) -> CollectorLimits {
        CollectorLimits::new(
            timeout,
            max_stdout_bytes,
            4 * 1024,
            Duration::from_millis(5),
        )
    }

    #[test]
    fn successful_command_is_captured_within_limits() {
        let mut command = test_command("[Console]::Out.Write('ok')", "printf ok");
        let output = run_command(
            CollectorName::HardwareInventory,
            &mut command,
            test_limits(Duration::from_secs(5), 1024),
            &CancellationToken::new(),
        )
        .unwrap_or_else(|error| panic!("unexpected command failure: {:?}", error.kind));

        assert_eq!(output.stdout(), b"ok");
    }

    #[test]
    fn cancellation_is_honoured_before_process_spawn() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut command = test_command("[Console]::Out.Write('not-run')", "printf not-run");

        let error = match run_command(
            CollectorName::HardwareInventory,
            &mut command,
            test_limits(Duration::from_secs(5), 1024),
            &cancellation,
        ) {
            Ok(_) => panic!("cancelled command unexpectedly ran"),
            Err(error) => error,
        };

        assert_eq!(error.kind, CollectorErrorKind::Cancelled);
    }

    #[test]
    fn long_running_command_is_terminated_at_timeout() {
        let mut command = test_command("Start-Sleep -Seconds 2", "sleep 2");

        let error = match run_command(
            CollectorName::HardwareInventory,
            &mut command,
            test_limits(Duration::from_millis(50), 1024),
            &CancellationToken::new(),
        ) {
            Ok(_) => panic!("timed-out command unexpectedly succeeded"),
            Err(error) => error,
        };

        assert_eq!(error.kind, CollectorErrorKind::TimedOut);
    }

    #[test]
    fn excessive_output_is_terminated_and_not_returned() {
        let mut command = test_command("[Console]::Out.Write('x' * 4096)", "printf '%4096s' x");

        let error = match run_command(
            CollectorName::HardwareInventory,
            &mut command,
            test_limits(Duration::from_secs(5), 128),
            &CancellationToken::new(),
        ) {
            Ok(_) => panic!("over-limit command unexpectedly succeeded"),
            Err(error) => error,
        };

        assert_eq!(error.kind, CollectorErrorKind::OutputLimitExceeded);
    }
}
