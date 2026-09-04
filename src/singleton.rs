//! Singleton / double-launch detection for dynamic-mcp (v1.8.0).
//!
//! ## Why this exists
//!
//! A `http` or `both` instance binds a TCP port. If two instances are started
//! on the **same HTTP endpoint** (host:port/path), one of them cannot use
//! HTTP. Rather than failing silently, we detect the situation at startup and
//! either step aside, take over, or warn the user — depending on which transport
//! is the superset.
//!
//! ## Mechanism
//!
//! Each `http`/`both` instance writes a lock file under
//! `<data_dir>/locks/<sha256(endpoint)[..16]>.lock` —— `data_dir` 优先取
//! `dmcp.exe` 同目录，不可写时降级 `~/.dynamic-mcp/`（见 [`dmcp_data_dir`]）。
//! 文件名是端点的哈希，所以**不同端点永不碰撞**（它们确实是独立的实例）。
//! 启动时原子地 *创建* 该文件（`O_EXCL`）；若已存在且属于一个*活着的*
//! dynamic-mcp 进程，就比较传输模式再决定如何处理。
//!
//! ## Decisions (see [`decide`])
//!
//! The decision is expressed as the rule set (R0–R4) below rather than as an
//! ad-hoc lookup table, so that every branch follows from the same two facts:
//! the **mode priority** and whether the new instance still has a usable
//! channel after losing HTTP.
//!
//! * **R0 域划分**：只有会争端口的 HTTP 域（http/both）参与仲裁；纯 stdio
//!   不绑端口，豁免仲裁（允许 N 个并存）。
//! * **R1 复用性**：http/both 支持多程序复用；stdio 受操作系统管道约束不可复用。
//! * **R2 优先级**：`both`(3) > `http`(2) > `stdio`(1)，以模式向下兼容为准，
//!   不以启动顺序为准。
//! * **R3 仲裁动作**：新优先级 > 旧 -> 驱逐旧、新接管；新优先级 <= 旧 ->
//!   新让位（自带 stdio 则降级为 stdio-only，否则自终止复用旧实例）。
//! * **R4 唯一例外**：`--no-evict` 仅豁免"新 both 驱逐旧 http"这一格（唯一含
//!   杀伤性动作者）。
//!
//! Concretely:
//!
//! * new `http` vs existing `both` / `http` -> **self-terminate** after
//!   [`POPUP_TIMEOUT_SECS`] (a pure-http instance with no HTTP is useless; the
//!   existing instance is reused).
//! * new `both` vs existing `http` (`allow_dual=false`) -> **kill old http**,
//!   stdio starts immediately, HTTP starts 8s later (port is free by then).
//! * new `both` vs existing `http` (`allow_dual=true`) -> **keep stdio only**,
//!   HTTP intentionally off, no kill. **This is intended coexistence, not
//!   waste** — see [`keep_stdio_msg`].
//! * new `both` vs existing `both` -> **keep stdio only**, HTTP off, no kill.
//!   A `both` that loses HTTP is no longer a `both`: it has become a stdio
//!   instance and therefore lawfully coexists in the stdio domain
//!   (**降级即转域**).
//!
//! A pure `stdio` instance never touches the port, so detection is skipped for
//! it. It nevertheless **registers** a file under `~/.dynamic-mcp/instances/`
//! so that `dmcp status` can list it — registration never participates in
//! arbitration.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::TransportMode;

/// 弹窗最长展示时长（秒）。超过该时长后自动关闭；用户可随时手动提前关闭。
/// 同时用于 SelfTerminate 实例的退出等待，确保弹窗能被完整读完。
pub const POPUP_TIMEOUT_SECS: u64 = 15;

/// On-disk record describing a running dynamic-mcp instance that owns an endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceLock {
    /// OS process id of the owner.
    pub pid: u32,
    /// `"http"` or `"both"` — the transport the owner exposes.
    pub transport: String,
    /// Whether the owner declared `--no-evict` (only meaningful for `http`).
    /// A later `both` reads this to decide whether to kill the old http.
    pub allow_dual: bool,
    /// The full HTTP endpoint (host:port/path) the owner bound to.
    pub endpoint: String,
    /// Absolute path of the owner's executable (used to rule out pid reuse).
    pub exe_path: String,
    /// RFC3339 timestamp when the lock was written (diagnostics only).
    pub started_at: String,
    /// 实例加载的配置文件路径（来自 `--config` 参数或 `DYNAMIC_MCP_CONFIG`）。
    ///
    /// 仅用于**提示**（对比新旧实例是否用同一份配置），**绝不参与锁键计算**：
    /// 一旦把配置身份并入锁键，同端口不同配置的两个实例就会跳过仲裁、同时去
    /// bind 同一端口，后者必然失败，服务反而起不来。
    ///
    /// **必须 `serde(default)`**：1.9.0 之前写入的锁文件没有此字段。若声明为必
    /// 填，旧锁会被 `try_acquire_lock` 当成"损坏锁"删除，新实例误判端口无主并
    /// 直冲 bind，最终因端口被旧实例占着而启动失败。
    #[serde(default)]
    pub config_path: Option<String>,
}

/// Pure decision produced by [`decide`] from the new transport + the existing lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionKind {
    /// No relevant conflict.
    None,
    /// New instance is a redundant `http` -> self-terminate after
    /// [`POPUP_TIMEOUT_SECS`] (it has no HTTP, so it cannot serve anything).
    SelfTerminate,
    /// New `both` may evict the old `http` -> kill old, start HTTP after 8s.
    KillOldThenStartHttp,
    /// New `both` keeps its own stdio, leaves HTTP off, does not kill old.
    KeepStdioHttpOff,
}

/// Result of trying to claim the lock file.
#[derive(Debug)]
pub enum AcquireResult {
    /// The file did not exist (or was stale) and we wrote our own lock.
    Acquired,
    /// A live, valid instance already owns the endpoint.
    Conflict(InstanceLock),
}

/// Tells `run_server` how to proceed after the early singleton check.
pub enum StartMode {
    /// No conflict (or we became the primary). Start stdio+http normally.
    Normal,
    /// Visitor: keep stdio only, HTTP intentionally disabled (B2/B3).
    StdioOnly,
    /// B1: stdio starts now, HTTP is delayed 8s (after evicting old http).
    DelayHttpThenNormal,
    /// A1/A3: redundant http — caller shows popup, waits [`POPUP_TIMEOUT_SECS`]
    /// so the popup can be read in full, then exits.
    SelfTerminate,
}

/// What [`check_singleton`] returns to `run_server`.
pub struct SingletonResult {
    pub mode: StartMode,
    /// Guard that releases whatever this instance registered: a lock file for
    /// http/both, a registration file for stdio.
    pub guard: Option<InstanceGuard>,
    /// Warnings/info gathered for the user (layer1 = double-open, layer2 = port).
    pub popup: PopupCollector,
}

/// 本实例留在磁盘上的那份"存在证明"，进程退出时由 Drop 自动清理。
///
/// 两类实例各留一种痕迹，但**只有锁参与仲裁**：
/// * HTTP 域（http/both）写锁文件 —— 争端口，必须仲裁（R0）；
/// * STDIO 域写登记文件 —— 不争端口，仅用于 `dmcp status` 查询，从不参与仲裁。
#[allow(dead_code)]
pub enum InstanceGuard {
    /// HTTP 域：持有端点锁，退出时删除属于自己的锁文件。
    Lock(LockGuard),
    /// STDIO 域：登记自身信息供查询，退出时删除登记文件。
    Registry(RegistryGuard),
}

// ---------------------------------------------------------------------------
// Lock file paths
// ---------------------------------------------------------------------------

/// 解析 dynamic-mcp 的数据目录：**优先 `dmcp.exe` 同目录**，不可写时降级为
/// `~/.dynamic-mcp/`。
///
/// 锁文件、登记文件、日志、工具产物都落在这个目录下。优先同目录是为了让每个
/// 分发目录（便携版 / 自解压包 / 各平台二进制）自带一套完整状态，互不干扰；
/// 降级到 `~/.dynamic-mcp/` 是为了在 Program Files 等受限目录下仍能跑。
///
/// 可写性用临时文件探针判定（`exists()` 不等于可写）。
pub fn dmcp_data_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.exists() {
                let probe = dir.join(format!(".dynamic-mcp-writable-{}.tmp", std::process::id()));
                if std::fs::File::create(&probe).is_ok() {
                    let _ = std::fs::remove_file(&probe);
                    return dir.to_path_buf();
                }
            }
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".dynamic-mcp")
}

/// Directory holding all instance lock files: `<data_dir>/locks/`.
pub fn lock_dir() -> PathBuf {
    dmcp_data_dir().join("locks")
}

/// Lock file path for an endpoint: `<lock_dir>/<sha256(endpoint)[..16]>.lock`.
pub fn lock_file_path(endpoint: &str) -> PathBuf {
    lock_dir().join(format!("{}.lock", endpoint_hash(endpoint)))
}

/// Stable, collision-resistant key for an endpoint: first 16 hex chars of SHA-256.
///
/// `DefaultHasher` is NOT used because its seed is randomized per process, so the
/// same endpoint would hash to different file names across runs. SHA-256 is stable.
fn endpoint_hash(endpoint: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(endpoint.as_bytes());
    let out = hasher.finalize();
    let mut s = String::with_capacity(16);
    for b in &out[..8] {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ---------------------------------------------------------------------------
// Pure decision logic
// ---------------------------------------------------------------------------

fn transport_str(t: TransportMode) -> &'static str {
    match t {
        TransportMode::Stdio => "stdio",
        TransportMode::Http => "http",
        TransportMode::Both => "both",
    }
}

/// 反解锁文件里记录的传输模式字符串；无法识别时返回 `None`（视为无关冲突）。
fn parse_transport(s: &str) -> Option<TransportMode> {
    match s {
        "stdio" => Some(TransportMode::Stdio),
        "http" => Some(TransportMode::Http),
        "both" => Some(TransportMode::Both),
        _ => None,
    }
}

/// **R2 优先级**：`both`(3) > `http`(2) > `stdio`(1)。
///
/// 按模式的"向下兼容能力"排序：数值越大表示能提供的通道越全。**与启动顺序
/// 无关** —— 后来者不一定赢，通道更全的一方才赢。
fn transport_priority(t: TransportMode) -> u8 {
    match t {
        TransportMode::Stdio => 1,
        TransportMode::Http => 2,
        TransportMode::Both => 3,
    }
}

/// 该模式是否自带 stdio 通道 —— 决定了它"丢掉 HTTP 之后还有没有存在价值"
/// （R1）：`both` 丢掉 HTTP 还剩 stdio，可以降级并存；`http` 丢掉 HTTP 就一
/// 无所有，只能自终止。
fn has_stdio(t: TransportMode) -> bool {
    matches!(t, TransportMode::Stdio | TransportMode::Both)
}

/// Decide what a new instance should do given its transport and the existing lock.
///
/// Pure: no I/O, no time. Easy to unit test every branch.
///
/// 决策由 R0–R4 规则集推导，而非逐格硬编码匹配：
/// * R0 任一侧属于 STDIO 域即不参与仲裁（不争端口）；
/// * R2/R3 新优先级更高则驱逐旧实例，否则新实例让位；
/// * R1 让位方式取决于新模式是否自带 stdio；
/// * R4 仅"驱逐"这一格受 `--no-evict` 豁免。
pub fn decide(new_transport: TransportMode, old: &InstanceLock) -> DecisionKind {
    let old_transport = match parse_transport(old.transport.as_str()) {
        Some(t) => t,
        // 锁里是无法识别的模式 -> 不做任何猜测，按无关冲突处理。
        None => return DecisionKind::None,
    };
    // R0：任一侧不争端口，就不存在需要仲裁的冲突。
    if matches!(new_transport, TransportMode::Stdio)
        || matches!(old_transport, TransportMode::Stdio)
    {
        return DecisionKind::None;
    }

    // R3 上半：新优先级更高 -> 驱逐旧实例、由新实例接管端口。
    if transport_priority(new_transport) > transport_priority(old_transport) {
        // R4：这是唯一含杀伤性动作的一格（both 驱逐 http），也是 `--no-evict`
        // 唯一的作用域；旧实例上锁时改为和平共存。
        return if old.allow_dual {
            DecisionKind::KeepStdioHttpOff
        } else {
            DecisionKind::KillOldThenStartHttp
        };
    }

    // R3 下半：新优先级不高于旧实例 -> 新实例让位。
    if has_stdio(new_transport) {
        // R1：`both` 让出 HTTP 后仍有 stdio，降级为 stdio 实例即可合法并存
        // （降级即转域）。这是预期行为，不是双开浪费。
        DecisionKind::KeepStdioHttpOff
    } else {
        // `http` 让出 HTTP 后一无所有，只能自终止，让调用方去复用旧实例。
        DecisionKind::SelfTerminate
    }
}

// ---------------------------------------------------------------------------
// Lock acquisition / persistence
// ---------------------------------------------------------------------------

/// Atomically claim the lock, or report a conflict with a live instance.
///
/// Uses `create_new` (the `O_EXCL` equivalent) so two instances starting at the
/// same instant cannot both believe they are primary. If the file already exists
/// we read it; a dead-or-reused pid makes it stale and we overwrite.
pub fn try_acquire_lock(endpoint: &str, lock: &InstanceLock) -> std::io::Result<AcquireResult> {
    let dir = lock_dir();
    let _ = fs::create_dir_all(&dir);
    let path = lock_file_path(endpoint);
    loop {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut f) => {
                let content = serde_json::to_string_pretty(lock)?;
                f.write_all(content.as_bytes())?;
                return Ok(AcquireResult::Acquired);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let existing: InstanceLock = match fs::read_to_string(&path)
                    .ok()
                    .and_then(|c| serde_json::from_str(&c).ok())
                {
                    Some(l) => l,
                    None => {
                        // Unreadable or corrupt lock -> treat as stale.
                        let _ = fs::remove_file(&path);
                        continue;
                    }
                };
                if !is_pid_alive(existing.pid) || !is_same_binary(existing.pid, &existing.exe_path)
                {
                    // Stale lock (owner dead or pid reused by another program).
                    let _ = fs::remove_file(&path);
                    continue;
                }
                return Ok(AcquireResult::Conflict(existing));
            }
            Err(e) => return Err(e),
        }
    }
}

/// Overwrite the lock file (used by B1 to become the new primary after eviction).
pub fn write_lock(endpoint: &str, lock: &InstanceLock) -> std::io::Result<()> {
    let dir = lock_dir();
    let _ = fs::create_dir_all(&dir);
    let path = lock_file_path(endpoint);
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    f.write_all(serde_json::to_string_pretty(lock)?.as_bytes())?;
    Ok(())
}

/// RAII guard: deletes *our own* lock file on drop, but only if it still records
/// our pid (so we never delete a lock belonging to a newer primary, e.g. after a
/// B1 takeover the old http must not remove the new both's lock).
pub struct LockGuard {
    path: PathBuf,
    pid: u32,
}

impl LockGuard {
    pub fn new(path: PathBuf, pid: u32) -> Self {
        Self { path, pid }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.path) {
            if let Ok(lock) = serde_json::from_str::<InstanceLock>(&content) {
                if lock.pid == self.pid {
                    let _ = fs::remove_file(&self.path);
                }
            }
        }
    }
}

/// 登记文件目录：`<data_dir>/instances/`。
///
/// 用于存放 stdio 实例的登记文件。stdio 实例不绑端口、不写锁，原本在磁盘上
/// 不留任何痕迹，因而无法被查询和治理（"行为碰巧正确，但机制缺失"）。登记
/// 文件**只供 `dmcp status` 读取，从不参与仲裁**（R0：STDIO 域豁免仲裁）。
pub fn instances_dir() -> PathBuf {
    dmcp_data_dir().join("instances")
}

/// 登记文件路径：`~/.dynamic-mcp/instances/stdio-<pid>.json`。
fn stdio_registry_path(pid: u32) -> PathBuf {
    instances_dir().join(format!("stdio-{}.json", pid))
}

/// RAII guard：进程退出时删除本实例的 stdio 登记文件。
///
/// 与 [`LockGuard`] 的区别：登记文件不参与仲裁，因此无需再校验内容是否仍属
/// 于自己 —— 文件名自带 pid，只有本进程会写它。
pub struct RegistryGuard {
    path: PathBuf,
}

impl Drop for RegistryGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// 为 stdio 实例写一份登记文件（best-effort：写失败也照常启动）。
///
/// stdio 实例不争端口、本就无需仲裁，写登记文件的唯一目的是让它出现在
/// `dmcp status` 里，从"看不见"变成"可查询"。
fn register_stdio_instance(config_path: Option<&str>) -> Option<RegistryGuard> {
    let dir = instances_dir();
    fs::create_dir_all(&dir).ok()?;
    let pid = std::process::id();
    let path = stdio_registry_path(pid);
    let record = InstanceLock {
        pid,
        transport: transport_str(TransportMode::Stdio).to_string(),
        allow_dual: false,
        // stdio 没有 HTTP 端点；空串表示"不适用"。
        endpoint: String::new(),
        exe_path: std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
        started_at: chrono::Utc::now().to_rfc3339(),
        config_path: config_path.map(|s| s.to_string()),
    };
    let content = serde_json::to_string_pretty(&record).ok()?;
    fs::write(&path, content).ok()?;
    Some(RegistryGuard { path })
}

// ---------------------------------------------------------------------------
// Process detection (cross-platform)
// ---------------------------------------------------------------------------

/// True if `pid` is alive (best-effort; false on any error).
pub fn is_pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // Signal 0: error-check only, does not actually send a signal.
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(windows)]
    {
        unsafe {
            use windows_sys::Win32::System::Threading::{
                GetExitCodeProcess, OpenProcess, PROCESS_QUERY_INFORMATION,
            };
            const STILL_ACTIVE: u32 = 259;
            let h = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if h.is_null() {
                return false;
            }
            let mut code: u32 = 0;
            GetExitCodeProcess(h, &mut code);
            windows_sys::Win32::Foundation::CloseHandle(h);
            code == STILL_ACTIVE
        }
    }
}

/// Absolute executable path of `pid`, if we can read it.
fn process_exe_path(pid: u32) -> Option<String> {
    #[cfg(unix)]
    {
        let link = fs::read_link(format!("/proc/{}/exe", pid)).ok()?;
        let canon = fs::canonicalize(&link).unwrap_or(link);
        Some(canon.to_string_lossy().into_owned())
    }
    #[cfg(windows)]
    {
        unsafe {
            use windows_sys::Win32::Foundation::CloseHandle;
            use windows_sys::Win32::System::Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
            };
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return None;
            }
            let mut buf = [0u16; 260];
            let mut size: u32 = buf.len() as u32;
            // BOOL return is ignored; we trust `size` instead.
            QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut size);
            CloseHandle(h);
            if size == 0 {
                return None;
            }
            let s = String::from_utf16_lossy(&buf[..size as usize]);
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
    }
}

/// True only if `pid` is alive AND is the same dynamic-mcp executable we expect.
/// This blocks the pid-reuse trap: a dead pid later handed to an unrelated
/// program (e.g. a calculator) would otherwise look "alive".
pub fn is_same_binary(pid: u32, expected_exe: &str) -> bool {
    if !is_pid_alive(pid) {
        return false;
    }
    match process_exe_path(pid) {
        Some(path) => {
            let p = Path::new(&path);
            let e = Path::new(expected_exe);
            p == e || p.file_name() == e.file_name()
        }
        None => false,
    }
}

/// Request termination of `pid`. Unix sends SIGTERM (graceful); Windows uses the
/// forceful `TerminateProcess` (there is no SIGTERM concept on Windows).
pub fn terminate_process(pid: u32) {
    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, TerminateProcess, PROCESS_TERMINATE,
        };
        let h = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !h.is_null() {
            TerminateProcess(h, 1);
            CloseHandle(h);
        }
    }
}

/// Unix-only: SIGKILL fallback after SIGTERM did not take effect quickly.
#[cfg(unix)]
pub fn force_kill(pid: u32) {
    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }
}

// ---------------------------------------------------------------------------
// User notifications
// ---------------------------------------------------------------------------

/// Collects layered messages, then emits ONE combined popup (Windows MessageBox /
/// unix stderr banner). Each layer also writes its own `tracing` line so it is
/// captured in logs even when no GUI is present.
pub struct PopupCollector {
    info: Option<String>,
    layer1: Option<String>, // double-open / redundant warning (warn! level)
    layer2: Option<String>, // port conflict (warn! level)
    layer3: Option<String>, // config mismatch (warn! level)
}

impl PopupCollector {
    pub fn new() -> Self {
        Self {
            info: None,
            layer1: None,
            layer2: None,
            layer3: None,
        }
    }

    /// Informational note (e.g. "both can take over http"). Logged at info level.
    pub fn add_info(&mut self, msg: impl Into<String>) {
        self.info = Some(msg.into());
        tracing::info!("{}", self.info.as_ref().unwrap());
    }

    /// Layer 1: a double-launch / redundant-instance warning.
    pub fn add_double_open(&mut self, msg: impl Into<String>) {
        self.layer1 = Some(msg.into());
        tracing::warn!("{}", self.layer1.as_ref().unwrap());
    }

    /// Layer 2: the HTTP port is (or will be) occupied by another instance.
    pub fn add_port_conflict(&mut self, msg: impl Into<String>) {
        self.layer2 = Some(msg.into());
        tracing::warn!("{}", self.layer2.as_ref().unwrap());
    }

    /// Layer 3: 新旧实例用的不是同一份配置文件（复用旧实例 = 用它那份配置）。
    pub fn add_config_mismatch(&mut self, msg: impl Into<String>) {
        self.layer3 = Some(msg.into());
        tracing::warn!("{}", self.layer3.as_ref().unwrap());
    }

    /// Emit a single combined popup (if anything was collected).
    pub fn emit(&self) {
        let mut lines: Vec<String> = Vec::new();
        if let Some(i) = &self.info {
            lines.push(i.clone());
        }
        if let Some(l) = &self.layer1 {
            lines.push(l.clone());
        }
        if let Some(l) = &self.layer2 {
            lines.push(l.clone());
        }
        if let Some(l) = &self.layer3 {
            lines.push(l.clone());
        }
        if lines.is_empty() {
            return;
        }
        let combined = lines.join("\n\n");
        let warned = self.layer1.is_some() || self.layer2.is_some() || self.layer3.is_some();
        let level = if warned { "告警" } else { "提示" };
        show_popup(&format!("dynamic-mcp {}", level), &combined);
    }
}

impl Default for PopupCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Show a user-facing popup that **closes itself after [`POPUP_TIMEOUT_SECS`]
/// seconds and can also be dismissed by the user at any time**.
///
/// * Windows: a real `MessageBoxW` on a background thread (so it never blocks
///   the server). A second thread locates the box by its exact title and posts
///   `WM_CLOSE` once the timeout elapses; if the user clicks OK first the window
///   is already gone and the closer thread simply exits.
/// * macOS: `osascript display dialog ... giving up after N` (native timeout).
/// * Linux: `notify-send -t <ms>` (best-effort — some desktop implementations
///   ignore the timeout, but the notification stays manually dismissible).
/// * All platforms also print to stderr (logs / headless fallback), which is the
///   only channel available on a server without a GUI.
pub fn show_popup(title: &str, message: &str) {
    eprintln!("[{}] {}", title, message);

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowW, IsWindow, MessageBoxW, PostMessageW, MB_ICONWARNING, WM_CLOSE,
        };
        let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let msg_w: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();

        // 线程 A：弹出消息框，一直阻塞到用户点击或收到 WM_CLOSE 为止。
        // `move` 让 UTF-16 缓冲区的生命周期长于本次调用。
        // 注：windows-sys 里 `HWND` 是类型别名而非元组结构体，故直接传空句柄。
        let title_for_box = title_w.clone();
        std::thread::spawn(move || unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                msg_w.as_ptr(),
                title_for_box.as_ptr(),
                MB_ICONWARNING,
            );
        });

        // 线程 B：超时后自动关闭。用户提前点掉时 `IsWindow` 变为 0，本线程随即
        // 退出，不会误伤任何窗口。
        std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(POPUP_TIMEOUT_SECS);
            let poll = Duration::from_millis(200);
            // 阶段一：等窗口被创建出来（它由线程 A 创建，需要一点点时间）。
            // 标题为精确匹配且由本程序独有，误匹配他人窗口的概率可忽略。
            let mut target = None;
            while std::time::Instant::now() < deadline {
                let hwnd = unsafe { FindWindowW(std::ptr::null(), title_w.as_ptr()) };
                if unsafe { IsWindow(hwnd) } != 0 {
                    target = Some(hwnd);
                    break;
                }
                std::thread::sleep(poll);
            }
            let hwnd = match target {
                Some(h) => h,
                // 窗口始终没出现（例如无桌面会话）：放弃自动关闭，什么都不做。
                None => return,
            };
            // 阶段二：守到截止时间；期间用户手动关闭则立即收工。
            loop {
                if unsafe { IsWindow(hwnd) } == 0 {
                    return;
                }
                if std::time::Instant::now() >= deadline {
                    let _ = unsafe { PostMessageW(hwnd, WM_CLOSE, 0, 0) };
                    return;
                }
                std::thread::sleep(poll);
            }
        });
    }
    #[cfg(target_os = "macos")]
    {
        // 仅转义 AppleScript 字符串内的 " 与 \（走 exec 不经 shell(命令行解释器)，无需处理 $/反引号）
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        // 裸换行在 AppleScript 字符串字面量里是语法错误 → 用 " & return & " 拼接（修复 mac(苹果系统) 弹窗因含 \n 静默失败）
        let joined = esc(&message)
            .split('\n')
            .collect::<Vec<_>>()
            .join("\" & return & \"");
        let title_esc = esc(&title);
        // `giving up after N` = N 秒后自动关闭；用户点按钮可随时提前关闭。
        let script = format!(
            "display dialog \"{}\" with title \"{}\" buttons {{\"好\"}} default button \"好\" giving up after {}",
            joined, title_esc, POPUP_TIMEOUT_SECS
        );
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        // `-t` 单位是毫秒。部分桌面实现会忽略该超时，但通知本身仍可手动点掉。
        let timeout_ms = POPUP_TIMEOUT_SECS.saturating_mul(1000);
        let _ = std::process::Command::new("notify-send")
            .args([
                "-u",
                "critical",
                "-t",
                &timeout_ms.to_string(),
                title,
                message,
            ])
            .output();
    }
}

// ---------------------------------------------------------------------------
// Orchestration
// ---------------------------------------------------------------------------

/// Run the early singleton check. Returns how `run_server` should start.
///
/// This is called *before* any config loading or upstream connections, so a
/// redundant `http` instance wastes no work before it self-terminates. The
/// endpoint is identified by `host:port/path` (the single `--http-endpoint` CLI arg,
/// form `host:port/path`); two instances on different endpoints are independent.
///
/// `config_path` is used **only** for diagnostics: the layer-3 config-mismatch
/// warning and the registration record listed by `dmcp status`. It is
/// deliberately **not** part of the lock key.
pub async fn check_singleton(
    transport: TransportMode,
    endpoint: &str,
    display_addr: &str,
    allow_dual: bool,
    config_path: Option<&str>,
) -> SingletonResult {
    // STDIO 域：不绑端口 -> 不需要仲裁，也不写锁。但仍写一份登记文件，让这个
    // 实例在 `dmcp status` 里可见（此前 stdio 实例在磁盘上不留任何痕迹）。
    if matches!(transport, TransportMode::Stdio) {
        return SingletonResult {
            mode: StartMode::Normal,
            guard: register_stdio_instance(config_path).map(InstanceGuard::Registry),
            popup: PopupCollector::new(),
        };
    }

    // Full endpoint key: host + port + path. Used as the lock-file name hash so
    // that distinct endpoints never collide (they are genuinely separate instances).
    let my_pid = std::process::id();
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let my_lock = InstanceLock {
        pid: my_pid,
        transport: transport_str(transport).to_string(),
        allow_dual,
        endpoint: endpoint.to_string(),
        exe_path,
        started_at: chrono::Utc::now().to_rfc3339(),
        config_path: config_path.map(|s| s.to_string()),
    };

    match try_acquire_lock(endpoint, &my_lock) {
        Ok(AcquireResult::Acquired) => SingletonResult {
            mode: StartMode::Normal,
            guard: Some(InstanceGuard::Lock(LockGuard::new(
                lock_file_path(endpoint),
                my_pid,
            ))),
            popup: PopupCollector::new(),
        },
        Ok(AcquireResult::Conflict(old)) => {
            let decision = decide(transport, &old);
            let mut popup = PopupCollector::new();
            match decision {
                DecisionKind::None => SingletonResult {
                    mode: StartMode::Normal,
                    guard: None,
                    popup,
                },
                DecisionKind::SelfTerminate => {
                    popup.add_double_open(double_open_msg(transport, &old));
                    popup.add_port_conflict(port_conflict_msg(&old, display_addr));
                    // 自终止 = 复用旧实例，也就意味着沿用**旧实例那份配置**。若两
                    // 份配置不同，调用方会静默拿到别的项目/别份配置下的工具集 ——
                    // 这是最危险的一种静默串味，必须明确告知，不能默默了事。
                    if let Some(msg) = config_mismatch_msg(config_path, &old) {
                        popup.add_config_mismatch(msg);
                    }
                    SingletonResult {
                        mode: StartMode::SelfTerminate,
                        guard: None,
                        popup,
                    }
                }
                DecisionKind::KillOldThenStartHttp => {
                    // B1: evict the old http now; stdio starts immediately; HTTP
                    // starts 8s later when the port is guaranteed free.
                    terminate_process(old.pid);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    #[cfg(unix)]
                    if is_pid_alive(old.pid) {
                        force_kill(old.pid);
                    }
                    let _ = write_lock(endpoint, &my_lock); // become the new primary
                    popup.add_info(
                        "当前启动的 both 模式可接替已运行的 http 实例：stdio 已立即启动，HTTP 将在 8 秒后接管端口。"
                            .to_string(),
                    );
                    SingletonResult {
                        mode: StartMode::DelayHttpThenNormal,
                        guard: Some(InstanceGuard::Lock(LockGuard::new(
                            lock_file_path(endpoint),
                            my_pid,
                        ))),
                        popup,
                    }
                }
                DecisionKind::KeepStdioHttpOff => {
                    // 降级为 stdio-only 属于**预期行为**：本实例让出 HTTP 后已不再
                    // 持有 HTTP 通道，按「降级即转域」它就是一个普通的 stdio 实例，
                    // 与旧实例合法并存（R0/R1）。此前复用"双开浪费"的兜底文案纯属
                    // 误导 —— 这里并没有任何资源被浪费，必须讲清楚。
                    popup.add_double_open(keep_stdio_msg(transport, &old));
                    // 写一份登记文件，让这个 stdio 实例在 `dmcp status` 里可见
                    // （否则它就从"有记录的 stdio"退化为"看不见的幽灵"）。
                    let guard = register_stdio_instance(config_path).map(InstanceGuard::Registry);
                    SingletonResult {
                        mode: StartMode::StdioOnly,
                        guard,
                        popup,
                    }
                }
            }
        }
        Err(e) => {
            // Lock dir/IO trouble: never block startup; proceed without detection.
            tracing::warn!("单例检测失败（{}），跳过检测正常启动", e);
            SingletonResult {
                mode: StartMode::Normal,
                guard: None,
                popup: PopupCollector::new(),
            }
        }
    }
}

/// Layer-1 message body (why this launch is redundant).
fn double_open_msg(new: TransportMode, old: &InstanceLock) -> String {
    match (new, old.transport.as_str()) {
        (TransportMode::Http, "both") => {
            "当前已有同端口的 both 模式 dynamic-mcp 在运行，其已包含 http 功能，可直接复用，无需再启动 http 模式。"
                .to_string()
        }
        (TransportMode::Http, "http") => {
            "当前已有同端口的 http 模式 dynamic-mcp 在运行，再开一个 http 属于双开浪费。"
                .to_string()
        }
        _ => "检测到同端口已有 dynamic-mcp 实例，存在双开浪费。".to_string(),
    }
}

/// Layer-2 message body (port occupied + what the user should do).
fn port_conflict_msg(old: &InstanceLock, addr: &str) -> String {
    match old.transport.as_str() {
        "both" => format!(
            "端口 {} 已被已运行的 both 模式 dynamic-mcp 占用。\n建议：① 你不必新开 http —— 已有的 both 模式已包含 http 功能，可直接复用；② 若确实需要单独再跑 http，请在启动命令的 --http-endpoint 与 LLM 的 MCP 配置文件中都改成其他端点（两处必须一致，例如 --http-endpoint 127.0.0.1:9000/dynamic-mcp）。",
            addr
        ),
        "http" => format!(
            "端口 {} 已被已运行的 http 模式 dynamic-mcp 占用。\n建议：若确实需要再开一个 http 实例，请在启动命令的 --http-endpoint 与 LLM 的 MCP 配置文件中都改成其他端点（两处必须一致，例如 --http-endpoint 127.0.0.1:9000/dynamic-mcp）。",
            addr
        ),
        _ => format!("端口 {} 已被另一个 dynamic-mcp 实例占用。", addr),
    }
}

/// 把锁/登记记录里的实例身份拼成一行人话，供告警文案直接引用。
/// 形如：`PID 4312 · both 模式 · 启动于 2026-09-02T11:20:33Z · 配置 /x/dynamic-mcp.json`
fn describe_instance(old: &InstanceLock) -> String {
    let cfg = match old.config_path.as_deref() {
        Some(p) if !p.is_empty() => format!(" · 配置 {}", p),
        _ => String::new(),
    };
    format!(
        "PID {} · {} 模式 · 启动于 {}{}",
        old.pid, old.transport, old.started_at, cfg
    )
}

/// Layer-1 文案：新实例让出 HTTP、降级为 stdio-only 的情形（#12 格）。
///
/// 进入这一格的路径有两条（both vs both、both vs 带 `--no-evict` 的 http），
/// 二者都属于**设计好的和平共存**，既不是错误也不是浪费。文案要做的三件事：
/// ① 报出旧实例身份，让用户能判断要不要去关它；② 说明本次少了什么（只有
/// HTTP 没起，stdio 照常可用）；③ 明确告知这是预期行为，消除"是不是坏了"的疑虑。
fn keep_stdio_msg(new: TransportMode, old: &InstanceLock) -> String {
    let who = describe_instance(old);
    match (new, old.transport.as_str()) {
        (TransportMode::Both, "both") => format!(
            "同端口已有一个 both 模式实例在运行（{}）。\n\
             本次不重复占用该端口：仅启用 stdio，HTTP 不启用 —— 这是设计好的共存方式\
             （端口归先来的进程，后来者走 stdio），属于正常现象，无需处理。\n\
             若确实希望本次接管 HTTP 端口，请先关闭上面那个实例。",
            who
        ),
        (TransportMode::Both, "http") => format!(
            "同端口已有一个带 --no-evict 的 http 实例在运行（{}）。\n\
             按该参数的约定不驱逐它：本次仅启用 stdio，HTTP 不启用，二者和平共存 ——\
             这正是 --no-evict 预期的结果，不是异常。\n\
             若希望本次改为接管端口，请去掉那个实例的 --no-evict 后重启它。",
            who
        ),
        _ => format!(
            "同端口已有一个 {} 模式实例在运行（{}）。\n本次仅启用 stdio，HTTP 不启用，二者共存。",
            old.transport, who
        ),
    }
}

/// Layer-3 文案：新旧实例加载的不是同一份配置文件（#2）。
///
/// 只在**双方配置都已知且确实不同**时返回 `Some`，信息不全就闭嘴，避免瞎告警。
/// 比较前先 `canonicalize` 归一化（消除 `./`、`..`、符号链接等差异）；归一化
/// 失败（例如文件不存在）时退回原始字符串比较。
///
/// **注意**：配置身份只用于提示，**绝不能并入锁键** —— 否则同端口不同配置的
/// 两个实例会跳过仲裁、同时去 bind 同一端口，后者必然失败，服务反而起不来。
fn config_mismatch_msg(our: Option<&str>, old: &InstanceLock) -> Option<String> {
    let ours = our.unwrap_or_default().trim();
    let theirs = old.config_path.as_deref().unwrap_or_default().trim();
    if ours.is_empty() || theirs.is_empty() {
        return None;
    }
    let norm = |p: &str| {
        std::fs::canonicalize(p)
            .map(|c| c.to_string_lossy().into_owned())
            .unwrap_or_else(|_| p.to_string())
    };
    let a = norm(ours);
    let b = norm(theirs);
    if a == b {
        return None;
    }
    Some(format!(
        "⚠️ 配置不一致：本次要加载的是\n  {}\n但同端口已在运行的实例（PID {}）加载的是\n  {}\n\
         本次将复用那个实例，因此你实际拿到的工具集来自**它的配置**，而非上面第一份。\n\
         若这属于两个不同项目，请给它们配置不同的 --http-endpoint（端口或路径不同即可）。",
        a, old.pid, b
    ))
}

/// 一条实例的来源类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceKind {
    /// HTTP 域：持有端点锁，参与端口仲裁。
    Lock,
    /// STDIO 域：仅登记，从不参与仲裁（R0）。
    Registry,
}

/// `dmcp status` 的一行输出：实例记录 + 来源类型 + 当前是否存活。
pub struct InstanceEntry {
    pub record: InstanceLock,
    pub kind: InstanceKind,
    /// pid 存活且确实是同一个 dynamic-mcp 可执行程序（防止 pid 复用误报）。
    pub alive: bool,
}

/// 启动时清理所有"所属进程已失效"的残留记录文件（登记文件 + 端点锁）。
///
/// 进程被强杀（Task Manager、`taskkill /F`）时，负责清理的
/// [`RegistryGuard::drop`] / [`LockGuard::drop`] 都不会执行，磁盘上便留下指向
/// 已死进程的记录，让 `dmcp status` 把尸体当成活实例报出来。本函数在任何
/// 实例启动时做一次兜底清扫。
///
/// **必须同步完成、且必须在 [`check_singleton`] 之前调用**：若与单例检测并发，
/// 本函数有可能删掉本实例刚写下的锁，令仲裁机制彻底失效；故这里刻意不走
/// `tokio::spawn`，与相邻的 [`cleanup_old_logs`]（纯年龄清理、无序要求）不同。
///
/// 判定残留的统一口径是 [`is_same_binary`]——它同时检查"pid 存活"与"确实是我
/// 们自己的可执行程序"，与 [`try_acquire_lock`]、[`list_instances`] 保持一致。
/// 只用 `is_pid_alive` 是不够的：Windows 的 pid 会回绕复用，一旦残留记录里的
/// pid 被系统分给了别的程序，单看存活就会把尸体永远留在盘上。
pub fn cleanup_orphans() {
    cleanup_dead_registry_files(&instances_dir());
    cleanup_dead_lock_files(&lock_dir());
}

/// 本进程可执行程序的绝对路径，用作"某个 pid 是不是另一个 dynamic-mcp"的基准。
fn current_exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// 从 `stdio-<pid>.json` 这样的登记文件名里解析出 pid。
///
/// 登记文件名自带 pid（见 [`stdio_registry_path`]），因此可以**不打开文件内容**
/// 就判断归属进程是否还活着——这是规避 TOCTOU 误删的关键：若去读文件内容，
/// 恰好撞上别人写了一半（半截 JSON 解析失败）或在 Windows 上因文件被占用而
/// 读取失败，都会被误判成"损坏文件"从而删掉一个活实例的登记。
fn pid_from_registry_filename(filename: &str) -> Option<u32> {
    filename
        .strip_prefix("stdio-")?
        .strip_suffix(".json")?
        .parse::<u32>()
        .ok()
}

/// 清理登记目录里所有"归属进程已失效"的 `stdio-*.json`。
fn cleanup_dead_registry_files(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let my_exe = current_exe_path();
    for entry in entries.flatten() {
        let path = entry.path();
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !filename.starts_with("stdio-") || !filename.ends_with(".json") {
            continue;
        }

        // 文件名合规：只用 pid 判定，绝不读取文件内容。
        if let Some(pid) = pid_from_registry_filename(filename) {
            if is_same_binary(pid, &my_exe) {
                // 存活的同胞实例 —— 动它不得。
                continue;
            }
            let _ = fs::remove_file(&path);
            tracing::info!("清理残留登记文件: {:?} (pid {} 已失效)", path, pid);
            continue;
        }

        // 文件名不合规范（不是我们写出来的）：退回读内容取 pid。
        // 读不到或解析不了就当损坏文件删掉——它连命名都不合规，不可能有效。
        let Ok(content) = fs::read_to_string(&path) else {
            let _ = fs::remove_file(&path);
            continue;
        };
        let Ok(record) = serde_json::from_str::<InstanceLock>(&content) else {
            let _ = fs::remove_file(&path);
            continue;
        };
        if !is_same_binary(record.pid, &record.exe_path) {
            let _ = fs::remove_file(&path);
            tracing::info!("清理残留登记文件: {:?} (pid {} 已失效)", path, record.pid);
        }
    }
}

/// 清理锁目录里所有"归属进程已失效"的 `*.lock`。
///
/// 锁文件名是端点的哈希、**不含 pid**（见 [`lock_file_path`]），只能读内容取
/// pid，因此这里比登记文件更保守：只有能明确判定为残留（pid 已死，或 pid 被
/// 复用给了别的程序）才删除；**读失败或解析失败一律跳过**，交给
/// [`try_acquire_lock`] 在真正要抢这个端点时处理——那样至少是在它自己的
/// 加锁路径上，不会误伤其它实例正在写入的锁。
fn cleanup_dead_lock_files(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("lock") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(record) = serde_json::from_str::<InstanceLock>(&content) else {
            continue;
        };
        if !is_same_binary(record.pid, &record.exe_path) {
            let _ = fs::remove_file(&path);
            tracing::info!("清理残留锁文件: {:?} (pid {} 已失效)", path, record.pid);
        }
    }
}

/// 扫描 `locks/` 与 `instances/`，列出当前所有已知实例（供 `dmcp status` 使用）。
///
/// 纯读取，不修改任何文件：过期记录的清理由各自持有者在退出时完成。这里只是
/// 把"看不见的运行中实例"变成一张可查询的清单。
pub fn list_instances() -> Vec<InstanceEntry> {
    let mut out = Vec::new();
    for (dir, kind) in [
        (lock_dir(), InstanceKind::Lock),
        (instances_dir(), InstanceKind::Registry),
    ] {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // 两类记录的扩展名不同：端点锁是 `.lock`，stdio 登记文件是 `.json`。
            let expected = match kind {
                InstanceKind::Lock => "lock",
                InstanceKind::Registry => "json",
            };
            if path.extension().and_then(|s| s.to_str()) != Some(expected) {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(record) = serde_json::from_str::<InstanceLock>(&content) else {
                continue;
            };
            // 登记文件只在持有者正常退出时删除；进程被强杀会残留，所以这里用
            // "pid 存活 + 同路径可执行程序"来判定，避免把残留文件当成活实例。
            let alive = is_pid_alive(record.pid) && is_same_binary(record.pid, &record.exe_path);
            out.push(InstanceEntry {
                record,
                kind,
                alive,
            });
        }
    }
    // 端口实例（HTTP 域）排在前，其次按端点、pid 排序，保证输出稳定可比对。
    // 用三次稳定排序代替复合 key，避免超宽行。
    out.sort_by_key(|e| e.record.pid);
    out.sort_by_key(|e| e.record.endpoint.clone());
    out.sort_by_key(|e| e.kind != InstanceKind::Lock);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock(transport: &str, allow_dual: bool) -> InstanceLock {
        InstanceLock {
            pid: 12345,
            transport: transport.to_string(),
            allow_dual,
            endpoint: "127.0.0.1:8082/dynamic-mcp".to_string(),
            exe_path: "/usr/bin/dmcp".to_string(),
            started_at: "2026-07-14T00:00:00Z".to_string(),
            config_path: None,
        }
    }

    #[test]
    fn decide_http_vs_both_self_terminates_regardless_of_switch() {
        assert_eq!(
            decide(TransportMode::Http, &lock("both", false)),
            DecisionKind::SelfTerminate
        );
        assert_eq!(
            decide(TransportMode::Http, &lock("both", true)),
            DecisionKind::SelfTerminate
        );
    }

    #[test]
    fn decide_http_vs_http_self_terminates() {
        assert_eq!(
            decide(TransportMode::Http, &lock("http", false)),
            DecisionKind::SelfTerminate
        );
        assert_eq!(
            decide(TransportMode::Http, &lock("http", true)),
            DecisionKind::SelfTerminate
        );
    }

    #[test]
    fn decide_both_vs_http_depends_on_allow_dual() {
        assert_eq!(
            decide(TransportMode::Both, &lock("http", false)),
            DecisionKind::KillOldThenStartHttp
        );
        assert_eq!(
            decide(TransportMode::Both, &lock("http", true)),
            DecisionKind::KeepStdioHttpOff
        );
    }

    #[test]
    fn decide_both_vs_both_keeps_stdio() {
        assert_eq!(
            decide(TransportMode::Both, &lock("both", false)),
            DecisionKind::KeepStdioHttpOff
        );
    }

    #[test]
    fn endpoint_hash_is_deterministic_and_distinct() {
        let a = endpoint_hash("127.0.0.1:8082/dynamic-mcp");
        let b = endpoint_hash("127.0.0.1:8082/dynamic-mcp");
        let c = endpoint_hash("127.0.0.1:8083/dynamic-mcp");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 16);
    }

    // ---- R0：STDIO 域豁免仲裁 ----

    #[test]
    fn decide_stdio_never_arbitrates() {
        // 任一侧属于 STDIO 域（不争端口）即不产生仲裁动作。
        for old in ["stdio", "http", "both"] {
            assert_eq!(
                decide(TransportMode::Stdio, &lock(old, false)),
                DecisionKind::None
            );
        }
        assert_eq!(
            decide(TransportMode::Http, &lock("stdio", false)),
            DecisionKind::None
        );
        assert_eq!(
            decide(TransportMode::Both, &lock("stdio", false)),
            DecisionKind::None
        );
    }

    // ---- R1/R2：优先级与"让位后能否存活" ----

    #[test]
    fn decide_priority_is_mode_compatibility_not_launch_order() {
        assert_eq!(transport_priority(TransportMode::Both), 3);
        assert_eq!(transport_priority(TransportMode::Http), 2);
        assert_eq!(transport_priority(TransportMode::Stdio), 1);
        // 自带 stdio 者让出 HTTP 后仍有价值，可降级并存；http 则一无所有。
        assert!(has_stdio(TransportMode::Both));
        assert!(has_stdio(TransportMode::Stdio));
        assert!(!has_stdio(TransportMode::Http));
    }

    #[test]
    fn decide_unknown_old_transport_is_no_conflict() {
        // 锁里是无法识别的模式：不做猜测，按无关冲突处理。
        let weird = lock("weird", false);
        assert_eq!(decide(TransportMode::Both, &weird), DecisionKind::None);
        assert_eq!(decide(TransportMode::Http, &weird), DecisionKind::None);
    }

    // ---- #2：配置一致性告警 ----

    #[test]
    fn config_mismatch_only_warns_when_both_known_and_different() {
        // 信息不全（任一侧未知）时不告警 —— 不瞎报。
        let unknown_old = config_mismatch_msg(Some("/a/x.json"), &lock("both", false));
        assert!(unknown_old.is_none());
        assert!(config_mismatch_msg(None, &lock("both", false)).is_none());
        // 同一份配置 -> 不告警。
        let mut same = lock("both", false);
        same.config_path = Some("/a/x.json".to_string());
        assert!(config_mismatch_msg(Some("/a/x.json"), &same).is_none());
        // 确实是两份配置 -> 告警，且两份路径都要点出来。
        let mut other = lock("both", false);
        other.config_path = Some("/b/y.json".to_string());
        let mismatch = config_mismatch_msg(Some("/a/x.json"), &other);
        let msg = mismatch.expect("配置不同应告警");
        assert!(msg.contains("/a/x.json"), "应点出本次配置：{msg}");
        assert!(msg.contains("/b/y.json"), "应点出旧实例配置：{msg}");
        // 空串视为"未知"，同样不参与比较。
        let mut blank = lock("both", false);
        blank.config_path = Some(String::new());
        assert!(config_mismatch_msg(Some("/a/x.json"), &blank).is_none());
    }

    #[test]
    fn legacy_lock_without_config_path_still_deserializes() {
        // 1.9.0 之前写入的锁没有 `config_path` 字段。若该字段不是 serde(default)，
        // 这里就会解析失败 -> 旧锁被 try_acquire_lock 当成损坏锁删除 -> 新实例
        // 误判端口无主并直冲 bind -> 因端口被旧实例占着而启动失败。
        let legacy = r#"{
            "pid": 4321,
            "transport": "http",
            "allow_dual": false,
            "endpoint": "127.0.0.1:8082/dynamic-mcp",
            "exe_path": "/usr/bin/dmcp",
            "started_at": "2026-07-14T00:00:00Z"
        }"#;
        let parsed: InstanceLock = serde_json::from_str(legacy).expect("旧锁必须能解析");
        assert_eq!(parsed.pid, 4321);
        assert!(parsed.config_path.is_none());
    }

    // ---- #1：降级文案不得再误导为"双开浪费" ----

    #[test]
    fn keep_stdio_msg_explains_coexistence_not_waste() {
        let mut old = lock("both", false);
        old.config_path = Some("/a/x.json".to_string());
        let msg = keep_stdio_msg(TransportMode::Both, &old);
        assert!(msg.contains("PID 12345"), "应带出旧实例身份：{msg}");
        assert!(msg.contains("/a/x.json"), "应带出旧实例配置：{msg}");
        assert!(!msg.contains("双开浪费"), "降级是预期行为，不是浪费：{msg}");
        assert!(msg.contains("正常现象"), "应明确说明这是预期行为：{msg}");
    }

    #[test]
    fn keep_stdio_msg_mentions_no_evict_opt_out() {
        let msg = keep_stdio_msg(TransportMode::Both, &lock("http", true));
        assert!(
            msg.contains("--no-evict"),
            "应说明是 --no-evict 生效：{msg}"
        );
        assert!(!msg.contains("双开浪费"), "和平共存不是浪费：{msg}");
    }

    // ---- M1 修复：降级为 stdio-only 时必须写登记文件 ----

    #[test]
    fn register_stdio_instance_writes_visible_file() {
        // 一个 both 实例因端口冲突降级为 stdio-only 后，必须写登记文件，
        // 否则它在 `dmcp status` 里就完全不可见（M1）。
        let guard = register_stdio_instance(Some("D:/test/config.json"));
        assert!(guard.is_some(), "registry 必须成功返回 guard");

        // 清理：让 RegistryGuard Drop 删除它
        drop(guard);

        // 验证登记目录干净（无本次 pid 的残留）
        let pid = std::process::id();
        let path = stdio_registry_path(pid);
        assert!(!path.exists(), "登记文件应在 guard drop 后自动删除");
    }

    // ---- cleanup_orphans：残留清理的判定路径（B1/B4/B5 回归） ----

    /// 建一个干净的临时目录；按测试名 + pid 双重隔离，避免并行测试互相踩。
    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("dmcp-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("创建临时目录失败");
        d
    }

    /// 一个几乎不可能存在的 pid：`is_same_binary` 对它必返回 false（pid 已死）。
    const DEAD_PID: u32 = u32::MAX;

    #[test]
    fn cleanup_registry_removes_file_of_dead_pid() {
        let dir = temp_dir("reg-dead");
        let path = dir.join(format!("stdio-{DEAD_PID}.json"));
        fs::write(&path, "{}").unwrap();
        assert!(path.exists());

        cleanup_dead_registry_files(&dir);

        assert!(!path.exists(), "pid 已死的登记文件应被清理");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_registry_keeps_file_of_live_pid() {
        // 用本测试进程自己的 pid 冒充"活着的同胞实例"：pid 存活且可执行文件路径
        // 与本进程一致，is_same_binary 为真 → 必须保留。
        let dir = temp_dir("reg-live");
        let path = dir.join(format!("stdio-{}.json", std::process::id()));
        fs::write(&path, "{}").unwrap();

        cleanup_dead_registry_files(&dir);

        assert!(path.exists(), "存活实例的登记文件绝不能被删");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_registry_never_deletes_live_pid_file_with_corrupt_content() {
        // B4 回归：文件内容是损坏的，但 pid 存活。旧实现先读内容、解析失败即删，
        // 于是误删了一个活实例的登记文件。修复后据文件名 pid 直接跳过，连内容
        // 都不读。
        let dir = temp_dir("reg-live-corrupt");
        let path = dir.join(format!("stdio-{}.json", std::process::id()));
        fs::write(&path, "this-is-not-json").unwrap();

        cleanup_dead_registry_files(&dir);

        assert!(
            path.exists(),
            "pid 存活时不得因内容损坏而删除（B4 TOCTOU 回归）"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_registry_reuses_dead_pid_is_still_removed() {
        // B5 回归：pid 虽"存活"却不是我们自己的程序（模拟 Windows pid 复用）。
        // 只判 is_pid_alive 会把它当成活实例而永久漏删，必须靠 exe 校验识别。
        //
        // 文件名刻意不合规范（stdio-abc.json 解析不出 pid），才会走"读内容取
        // pid + exe_path"那条兜底分支 —— 合规文件名走的是"只用 pid、不读内容"
        // 的路径，那里改文件内容是无效的。
        let dir = temp_dir("reg-reuse");
        let path = dir.join("stdio-abc.json");
        let mut record = lock("stdio", false);
        record.pid = std::process::id();
        // 故意填一个绝非本进程的可执行路径，模拟 pid 被别的程序占用。
        record.exe_path = "/definitely/not/this/binary".to_string();
        fs::write(&path, serde_json::to_string_pretty(&record).unwrap()).unwrap();

        cleanup_dead_registry_files(&dir);

        assert!(
            !path.exists(),
            "pid 存活但 exe 不匹配 = pid 被复用，应判为残留并清理（B5）"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_lock_removes_lock_of_dead_pid() {
        // B1：锁文件此前完全无人清理，本用例锁住这条新覆盖。
        let dir = temp_dir("lock-dead");
        let path = dir.join("deadbeef12345678.lock");
        let mut dead = lock("http", false);
        dead.pid = DEAD_PID;
        fs::write(&path, serde_json::to_string_pretty(&dead).unwrap()).unwrap();

        cleanup_dead_lock_files(&dir);

        assert!(!path.exists(), "pid 已死的锁文件应被清理（B1 覆盖缺口）");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_lock_keeps_lock_of_live_pid() {
        let dir = temp_dir("lock-live");
        let path = dir.join("cafebabe12345678.lock");
        let mut live = lock("http", false);
        live.pid = std::process::id();
        live.exe_path = current_exe_path();
        fs::write(&path, serde_json::to_string_pretty(&live).unwrap()).unwrap();

        cleanup_dead_lock_files(&dir);

        assert!(path.exists(), "存活实例持有的锁绝不能被删");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_lock_skips_unreadable_lock_conservatively() {
        // 保守策略：锁文件名不含 pid、只能读内容；内容解析不了时一律跳过，
        // 交给 try_acquire_lock 在真正抢端点时处理，避免误删他人正在写的锁。
        let dir = temp_dir("lock-corrupt");
        let path = dir.join("cafebabe12345678.lock");
        fs::write(&path, "not-json-at-all").unwrap();

        cleanup_dead_lock_files(&dir);

        assert!(path.exists(), "无法判定的锁文件应保守跳过、不删除");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn pid_from_registry_filename_parses_pid() {
        assert_eq!(pid_from_registry_filename("stdio-4321.json"), Some(4321));
        // 不合规范的文件名返回 None，交由读内容兜底。
        assert_eq!(pid_from_registry_filename("stdio-abc.json"), None);
        assert_eq!(pid_from_registry_filename("other-4321.json"), None);
        assert_eq!(pid_from_registry_filename("stdio-4321.txt"), None);
    }
}
