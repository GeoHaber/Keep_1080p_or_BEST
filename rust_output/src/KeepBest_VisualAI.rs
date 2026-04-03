use anyhow::{Result, Context};
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use tokio;

pub const REV: &str = "\\nKeep 1080p or Best - Visual AI v6.1 (Simplify to Amplify)\\n\\nCHANGELOG v6.1:\\n  - REFACTOR: \"Simplify to Amplify\" - reduced ~300 lines while keeping all features\\n  - REFACTOR: DependencyManager consolidated (4 methods -> 1 check_import)\\n  - REFACTOR: Constants.CODEC_RANK and FONTS centralized\\n  - REFACTOR: MediaFile.to_meta_dict() for cleaner cache operations\\n  - REFACTOR: VLCPlayerApp simplified - single-pass best value computation\\n  - REFACTOR: ZoomCompareWindow - cached images, combined handlers, faster rendering\\n  - REFACTOR: BackgroundFingerprinter - uses DuplicateDetector.hamming_distance\\n  - PERF: Zoom uses BILINEAR during drag, LANCZOS on release\\n  - PERF: Single-pass comparison stats in VLC player\\n\\nAll features from v6.0 preserved:\\n  - Equal window sizes, color-coded info panel\\n  - Background fingerprinting with progress bar\\n  - Zoom/magnify comparison (Press Z)\\n  - Folder path display\\n";

pub static LOGGER: std::sync::LazyLock<String /* logging::getLogger */> = std::sync::LazyLock::new(|| Default::default());

pub static PRINT_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

pub const HAS_NUMPY: &str = "DependencyManager.check_import('numpy') or DependencyManager.try_install('numpy')";

pub const HAS_VLC: &str = "DependencyManager.check_import('vlc') or DependencyManager.try_install('python-vlc', 'vlc')";

pub const HAS_TQDM: &str = "DependencyManager.check_import('tqdm') or DependencyManager.try_install('tqdm')";

pub const HAS_PILLOW: &str = "DependencyManager.check_import('PIL') or DependencyManager.try_install('Pillow', 'PIL')";

/// Manages optional dependencies with single unified check method.
#[derive(Debug, Clone)]
pub struct DependencyManager {
}

impl DependencyManager {
    /// Single method for all dependency checks.
    pub fn check_import(name: String) -> Result<bool> {
        // Single method for all dependency checks.
        // try:
        {
            __import__(name);
            true
        }
        // except ImportError as _e:
    }
    /// Attempt to pip-install a package if not already available.
    pub fn try_install(package_name: String, import_name: Option<String>) -> Result<bool> {
        // Attempt to pip-install a package if not already available.
        if import_name.is_none() {
            let mut import_name = package_name;
        }
        if DependencyManager.check_import(import_name) {
            true
        }
        safe_print(format!("\n{} not found. Installing...", package_name));
        // try:
        {
            subprocess::check_call(vec![sys::executable, "-m".to_string(), "pip".to_string(), "install".to_string(), package_name, "--quiet".to_string()], /* stdout= */ subprocess::DEVNULL, /* stderr= */ subprocess::DEVNULL);
            safe_print(format!("OK {} installed successfully", package_name));
            true
        }
        // except (OSError, subprocess::SubprocessError) as e:
    }
}

#[derive(Debug, Clone)]
pub struct Constants {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub source_dirs: Vec<PathBuf>,
    pub except_dir: PathBuf,
    pub user_recycle_dir: PathBuf,
    pub exclude_keywords: Vec<String>,
    pub video_extensions: HashSet<String>,
    pub max_workers: i64,
    pub dry_run: bool,
    pub log_file: Option<PathBuf>,
    pub verbose: bool,
    pub cache_db_path: PathBuf,
}

/// SQLite-based cache for file metadata and fingerprints.
#[derive(Debug, Clone)]
pub struct MetadataCache {
    pub db_path: String,
    pub logger: String,
    pub conn: Option<serde_json::Value>,
    pub _lock: std::sync::Mutex<()>,
}

impl MetadataCache {
    /// Initialize the cache with a SQLite database at *db_path*.
    pub fn new(db_path: PathBuf, logger: logging::Logger) -> Self {
        Self {
            db_path,
            logger,
            conn: None,
            _lock: std::sync::Mutex::new(()),
        }
    }
    /// Open or create the SQLite database and verify schema version.
    pub fn _init_db(&mut self) -> Result<()> {
        // Open or create the SQLite database and verify schema version.
        // try:
        {
            self.conn = /* sqlite3 */ self.db_path, /* check_same_thread= */ false;
            self.conn.row_factory = sqlite3::Row;
            let mut current_version = self._get_meta("schema_version".to_string());
            if current_version.is_none() {
                self._create_schema();
            } else if current_version.to_string().parse::<i64>().unwrap_or(0) != self.SCHEMA_VERSION {
                self.logger.warning("Cache schema mismatch. Rebuilding...".to_string());
                self._reset_db();
            }
            self.conn.execute("PRAGMA journal_mode=WAL;".to_string());
            self.conn.commit();
        }
        // except (OSError, sqlite3::Error) as e:
    }
    /// Create the initial database tables and metadata entries.
    pub fn _create_schema(&self) -> () {
        // Create the initial database tables and metadata entries.
        if !self.conn {
            return;
        }
        self.conn.execute("\n            CREATE TABLE IF NOT EXISTS file_cache (\n                path TEXT PRIMARY KEY, size INTEGER, mtime REAL,\n                metadata_json TEXT, fingerprint_json TEXT, last_seen REAL\n            )\n        ".to_string());
        self.conn.execute("\n            CREATE TABLE IF NOT EXISTS db_info (key TEXT PRIMARY KEY, value TEXT)\n        ".to_string());
        self._set_meta("schema_version".to_string(), self.SCHEMA_VERSION.to_string());
        self._set_meta("app_version".to_string(), self.APP_VERSION);
        self.conn.commit();
    }
    /// Drop and recreate the database from scratch.
    pub fn _reset_db(&mut self) -> Result<()> {
        // Drop and recreate the database from scratch.
        if self.conn {
            self.conn.close();
        }
        if self.db_path.exists() {
            // try:
            {
                self.db_path.remove_file().ok();
            }
            // except OSError as _e:
        }
        self.conn = /* sqlite3 */ self.db_path, /* check_same_thread= */ false;
        self.conn.row_factory = sqlite3::Row;
        Ok(self._create_schema())
    }
    /// Retrieve a value from the db_info metadata table.
    pub fn _get_meta(&mut self, key: String) -> Result<Option<String>> {
        // Retrieve a value from the db_info metadata table.
        // try:
        {
            let mut cur = self.conn.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='db_info'".to_string());
            if !cur.fetchone() {
                None
            }
            let mut cur = self.conn.execute("SELECT value FROM db_info WHERE key = ?".to_string(), (key));
            let mut row = cur.fetchone();
            if row { row["value".to_string()] } else { None }
        }
        // except (sqlite3::Error, KeyError) as _e:
    }
    /// Insert or update a key-value pair in the db_info table.
    pub fn _set_meta(&self, key: String, value: String) -> () {
        // Insert or update a key-value pair in the db_info table.
        self.conn.execute("INSERT OR REPLACE INTO db_info (key, value) VALUES (?, ?)".to_string(), (key, value));
    }
    /// Return cached metadata and fingerprint for *path*, or None.
    pub fn get(&mut self, path: PathBuf) -> Result<Option<HashMap>> {
        // Return cached metadata and fingerprint for *path*, or None.
        if !self.conn {
            None
        }
        let _ctx = self._lock;
        {
            // try:
            {
                let mut stat = path.stat();
                let mut cur = self.conn.execute("SELECT metadata_json, fingerprint_json FROM file_cache WHERE path = ? AND size = ? AND mtime = ?".to_string(), (path.to_string(), stat.st_size, stat.st_mtime));
                let mut row = cur.fetchone();
                if row {
                    let mut meta = serde_json::from_str(&row["metadata_json".to_string()]).unwrap();
                    let mut fp = if row["fingerprint_json".to_string()] { serde_json::from_str(&row["fingerprint_json".to_string()]).unwrap() } else { None };
                    self.conn.execute("UPDATE file_cache SET last_seen = ? WHERE path = ?".to_string(), (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64(), path.to_string()));
                    self.conn.commit();
                    HashMap::from([("meta".to_string(), meta), ("fingerprint".to_string(), fp)])
                }
            }
            // except (OSError, sqlite3::Error, json::JSONDecodeError, KeyError) as _e:
        }
        Ok(None)
    }
    /// Start an explicit database transaction.
    pub fn begin_transaction(&self) -> () {
        // Start an explicit database transaction.
        if self.conn {
            let _ctx = self._lock;
            {
                self.conn.execute("BEGIN TRANSACTION".to_string());
            }
        }
    }
    /// Commit the current database transaction.
    pub fn commit_transaction(&self) -> () {
        // Commit the current database transaction.
        if self.conn {
            let _ctx = self._lock;
            {
                self.conn.commit();
            }
        }
    }
    /// Store metadata and optional fingerprint for *path* in the cache.
    pub fn set(&mut self, path: PathBuf, meta: HashMap<String, serde_json::Value>, fingerprint: Option<Vec<i64>>, commit: bool) -> Result<()> {
        // Store metadata and optional fingerprint for *path* in the cache.
        if !self.conn {
            return;
        }
        let _ctx = self._lock;
        {
            // try:
            {
                let mut stat = path.stat();
                self.conn.execute("INSERT OR REPLACE INTO file_cache (path, size, mtime, metadata_json, fingerprint_json, last_seen)\n                       VALUES (?, ?, ?, ?, ?, ?)".to_string(), (path.to_string(), stat.st_size, stat.st_mtime, serde_json::to_string(&meta).unwrap(), if fingerprint { serde_json::to_string(&fingerprint).unwrap() } else { None }, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64()));
                if commit {
                    self.conn.commit();
                }
            }
            // except (OSError, sqlite3::Error) as e:
        }
    }
    /// Close the database connection.
    pub fn close(&self) -> () {
        // Close the database connection.
        if self.conn {
            self.conn.close();
        }
    }
}

/// Manages subprocess lifecycle and cleanup.
#[derive(Debug, Clone)]
pub struct ProcessManager {
    pub _procs: HashMap<i64, subprocess::Popen>,
    pub _lock: std::sync::Mutex<()>,
    pub logger: String,
}

impl ProcessManager {
    /// Initialize the process manager with cleanup on program exit.
    pub fn new(logger: logging::Logger) -> Self {
        Self {
            _procs: HashMap::new(),
            _lock: std::sync::Mutex::new(()),
            logger,
        }
    }
    /// Track a subprocess for lifecycle management.
    pub fn register(&mut self, proc: subprocess::Popen) -> () {
        // Track a subprocess for lifecycle management.
        if proc.pid {
            let _ctx = self._lock;
            {
                self._procs[proc.pid] = proc;
            }
        }
    }
    /// Remove a subprocess from tracking.
    pub fn unregister(&self, proc: subprocess::Popen) -> () {
        // Remove a subprocess from tracking.
        if proc.pid {
            let _ctx = self._lock;
            {
                self._procs.remove(&proc.pid).unwrap_or(None);
            }
        }
    }
    /// Kill all tracked subprocesses that are still running.
    pub fn terminate_all(&mut self) -> Result<()> {
        // Kill all tracked subprocesses that are still running.
        let _ctx = self._lock;
        {
            let mut procs = self._procs.values().into_iter().collect::<Vec<_>>();
        }
        for proc in procs.iter() {
            if proc.returncode.is_none() {
                // try:
                {
                    proc.kill();
                }
                // except OSError as _e:
            }
        }
    }
    /// Run a command asynchronously with timeout, returning (returncode, stdout, stderr).
    pub async fn run_command(&mut self, cmd: Vec<String>, timeout: f64, binary_output: bool) -> Result<(i64, Box<dyn std::any::Any>, Box<dyn std::any::Any>)> {
        // Run a command asynchronously with timeout, returning (returncode, stdout, stderr).
        let mut proc = None;
        // try:
        {
            let mut proc = asyncio.create_subprocess_exec(/* *cmd */, /* stdout= */ asyncio.subprocess::PIPE, /* stderr= */ asyncio.subprocess::PIPE, /* creationflags= */ if platform.system() == "Windows".to_string() { subprocess::CREATE_NEW_PROCESS_GROUP } else { 0 }).await;
            self.register(proc);
            let (mut stdout, mut stderr) = asyncio.wait_for(proc.communicate(), /* timeout= */ timeout).await;
            if binary_output {
                (proc.returncode, stdout, stderr)
            }
            (proc.returncode, stdout.decode(/* errors= */ "ignore".to_string()), stderr.decode(/* errors= */ "ignore".to_string()))
        }
        // except asyncio.TimeoutError as _e:
        // except OSError as e:
        // finally:
            if proc {
                self.unregister(proc);
            }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub path: PathBuf,
    pub name: String,
    pub size: i64,
    pub content_type: String,
    pub title: String,
    pub width: i64,
    pub height: i64,
    pub duration: f64,
    pub bitrate: f64,
    pub video_codec: String,
    pub bit_depth: i64,
    pub audio_channels: i64,
    pub audio_codec: String,
    pub audio_streams_count: i64,
    pub bpp: f64,
    pub subs_count: i64,
    pub visual_hashes: Option<Vec<i64>>,
    pub sort_score: tuple,
    pub created: f64,
    pub modified: f64,
    pub is_hdr: bool,
    pub is_dolby_vision: bool,
}

impl MediaFile {
    /// Human-readable creation date.
    pub fn created_str(&self) -> &String {
        // Human-readable creation date.
        datetime::fromtimestamp(self.created).strftime("%Y-%m-%d".to_string())
    }
    /// Human-readable modification date.
    pub fn modified_str(&self) -> &String {
        // Human-readable modification date.
        datetime::fromtimestamp(self.modified).strftime("%Y-%m-%d".to_string())
    }
    /// Resolution as 'WIDTHxHEIGHT' or 'Unknown'.
    pub fn resolution_str(&self) -> &String {
        // Resolution as 'WIDTHxHEIGHT' or 'Unknown'.
        if self.width > 0 { format!("{}x{}", self.width, self.height) } else { "Unknown".to_string() }
    }
    /// Total pixel count (width * height).
    pub fn pixels(&self) -> i64 {
        // Total pixel count (width * height).
        (self.width * self.height)
    }
    /// File size formatted for display.
    pub fn nice_size(&self) -> &String {
        // File size formatted for display.
        format_file_size(self.size)
    }
    /// HDR format label: 'Dolby Vision', 'HDR10', or 'SDR'.
    pub fn hdr_str(&self) -> &String {
        // HDR format label: 'Dolby Vision', 'HDR10', or 'SDR'.
        if self.is_dolby_vision {
            "Dolby Vision".to_string()
        } else if self.is_hdr {
            "HDR10".to_string()
        }
        "SDR".to_string()
    }
    /// Serialize media attributes to a dict for cache storage.
    pub fn to_meta_dict(&self) -> HashMap {
        // Serialize media attributes to a dict for cache storage.
        HashMap::from([("width".to_string(), self.width), ("height".to_string(), self.height), ("duration".to_string(), self.duration), ("bitrate".to_string(), self.bitrate), ("video_codec".to_string(), self.video_codec), ("bit_depth".to_string(), self.bit_depth), ("audio_channels".to_string(), self.audio_channels), ("audio_codec".to_string(), self.audio_codec), ("audio_streams_count".to_string(), self.audio_streams_count), ("bpp".to_string(), self.bpp), ("subs_count".to_string(), self.subs_count), ("is_hdr".to_string(), self.is_hdr), ("is_dolby_vision".to_string(), self.is_dolby_vision)])
    }
}

/// Handles visual fingerprinting and pHash computation.
#[derive(Debug, Clone)]
pub struct FingerprintGenerator {
    pub pm: String,
    pub ffmpeg_bin: String,
    pub logger: String,
    pub executor: String /* concurrent.futures.ThreadPoolExecutor */,
}

impl FingerprintGenerator {
    /// Initialize the fingerprint generator with FFmpeg and thread pool.
    pub fn new(process_mgr: ProcessManager, ffmpeg_bin: String, logger: logging::Logger, max_workers: i64) -> Self {
        Self {
            pm: process_mgr,
            ffmpeg_bin,
            logger,
            executor: concurrent.futures.ThreadPoolExecutor(/* max_workers= */ (os::cpu_count() || 4).min(max_workers)),
        }
    }
    /// Compute a 2-D DCT on an 8x8 block for perceptual hashing.
    pub fn _dct_2d(&self, matrix_8x8: Box<dyn std::any::Any>) -> Box<dyn std::any::Any> {
        // Compute a 2-D DCT on an 8x8 block for perceptual hashing.
        if HAS_NUMPY {
            let mut N = Constants.PHASH_DCT_SIZE;
            let mut dct = np.zeros((N, N), /* dtype= */ np.float64);
            let mut x = np.arange(N);
            let mut u = np.arange(N);
            let mut cos_table = np.cos(((np.outer(((2 * x) + 1), u) * np.pi) / (2 * N)));
            let mut c = np.ones(N);
            c[0] = (1.0_f64 / np.sqrt(2));
            for u_idx in 0..N.iter() {
                for v_idx in 0..N.iter() {
                    let mut sum_val = np.sum((matrix_8x8 * np.outer(cos_table[(.., u_idx)], cos_table[(.., v_idx)])));
                    dct[(u_idx, v_idx)] = (((0.25_f64 * c[&u_idx]) * c[&v_idx]) * sum_val);
                }
            }
            dct
        }
        np.zeros((8, 8))
    }
    /// Compute a single perceptual hash from raw grayscale image bytes.
    pub fn _compute_single_hash(&mut self, image_bytes: Vec<u8>) -> i64 {
        // Compute a single perceptual hash from raw grayscale image bytes.
        let mut size = Constants.FINGERPRINT_SIZE;
        if (!image_bytes || image_bytes.len() != (size * size)) {
            0
        }
        if HAS_NUMPY {
            let mut img = np.frombuffer(image_bytes, /* dtype= */ np.uint8).reshape(size, size);
            let mut matrix_8x8 = np.zeros((8, 8), /* dtype= */ np.float64);
            let mut block_size = (size / 8);
            for y in 0..8.iter() {
                for x in 0..8.iter() {
                    let mut block = img[((y * block_size)..((y + 1) * block_size), (x * block_size)..((x + 1) * block_size))];
                    matrix_8x8[(y, x)] = np.mean(block);
                }
            }
            let mut dct_vals = self._dct_2d(matrix_8x8);
            let mut vals = 0..8.iter().map(|y| dct_vals[(y, x)]).collect::<Vec<_>>();
            let mut median = np.median(vals);
            let mut hash_val = 0;
            for y in 0..8.iter() {
                for x in 0..8.iter() {
                    let mut hash_val = ((hash_val << 1) | if dct_vals[(y, x)] > median { 1 } else { 0 });
                }
            }
            hash_val
        }
        0
    }
    /// Split a raw buffer into three frames and hash each one.
    pub fn _compute_hashes_from_buffer(&mut self, buffer: Vec<u8>) -> Vec<i64> {
        // Split a raw buffer into three frames and hash each one.
        let mut chunk_size = (Constants.FINGERPRINT_SIZE * Constants.FINGERPRINT_SIZE);
        if buffer.len() != (3 * chunk_size) {
            vec![]
        }
        0..3.iter().map(|i| self._compute_single_hash(buffer[(i * chunk_size)..((i + 1) * chunk_size)])).collect::<Vec<_>>()
    }
    /// Generate visual fingerprint hashes for a video file.
    pub async fn generate(&mut self, path: PathBuf, duration: f64) -> Option<Vec<i64>> {
        // Generate visual fingerprint hashes for a video file.
        if duration < Constants.MIN_DURATION_FOR_FINGERPRINT {
            None
        }
        let mut ts = Constants.FINGERPRINT_TIMESTAMPS.iter().map(|p| (duration * p)).collect::<Vec<_>>();
        let mut cmd = vec![self.ffmpeg_bin, "-y".to_string(), "-hide_banner".to_string(), "-loglevel".to_string(), "error".to_string(), "-ss".to_string(), format!("{:.2}", ts[0]), "-i".to_string(), path.to_string(), "-ss".to_string(), format!("{:.2}", ts[1]), "-i".to_string(), path.to_string(), "-ss".to_string(), format!("{:.2}", ts[2]), "-i".to_string(), path.to_string(), "-filter_complex".to_string(), format!("[0:v]scale={}:{},format=gray[v0];[1:v]scale={}:{},format=gray[v1];[2:v]scale={}:{},format=gray[v2];[v0][v1][v2]hstack=inputs=3", Constants.FINGERPRINT_SIZE, Constants.FINGERPRINT_SIZE, Constants.FINGERPRINT_SIZE, Constants.FINGERPRINT_SIZE, Constants.FINGERPRINT_SIZE, Constants.FINGERPRINT_SIZE), "-vframes".to_string(), "1".to_string(), "-f".to_string(), "rawvideo".to_string(), "-".to_string()];
        let (mut rc, mut out, _) = self.pm.run_command(cmd, Constants.FINGERPRINT_TIMEOUT, /* binary_output= */ true).await;
        if (rc == 0 && out.len() == Constants.FINGERPRINT_BUFFER_SIZE) {
            let mut r#loop = asyncio.get_running_loop();
            r#loop.run_in_executor(self.executor, self._compute_hashes_from_buffer, out).await
        }
        None
    }
    /// Shut down the thread pool executor.
    pub fn shutdown(&mut self) -> () {
        // Shut down the thread pool executor.
        self.executor.shutdown(/* wait= */ false);
    }
}

/// Handles file scanning and metadata extraction.
#[derive(Debug, Clone)]
pub struct MediaScanner {
    pub config: String,
    pub pm: String,
    pub logger: String,
    pub cache: String,
    pub ffmpeg_bin: String,
    pub ffprobe_bin: String,
    pub non_movie_regexes: String,
}

impl MediaScanner {
    /// Initialize the scanner with configuration and backend services.
    pub fn new(config: Config, process_mgr: ProcessManager, logger: logging::Logger, cache: MetadataCache) -> Self {
        Self {
            config,
            pm: process_mgr,
            logger,
            cache,
            ffmpeg_bin: (shutil::which("ffmpeg".to_string()) || "ffmpeg".to_string()),
            ffprobe_bin: (shutil::which("ffprobe".to_string()) || "ffprobe".to_string()),
            non_movie_regexes: config.exclude_keywords.iter().map(|kw| regex::Regex::new(&format!("\\b{}\\b", kw)).unwrap()).collect::<Vec<_>>(),
        }
    }
    /// Verify that FFprobe is available and working.
    pub async fn check_binaries(&mut self) -> Result<()> {
        // Verify that FFprobe is available and working.
        let (mut rc, _, mut err) = self.pm.run_command(vec![self.ffprobe_bin, "-version".to_string()], 5).await;
        if rc != 0 {
            return Err(anyhow::anyhow!("RuntimeError(f'FFprobe check failed: {err}')"));
        }
    }
    /// Extract (title, content_type, episode_info) from a filename.
    pub fn _parse_filename(&self, path: PathBuf) -> (String, String, String) {
        // Extract (title, content_type, episode_info) from a filename.
        let mut name = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
        let mut clean = regex::Regex::new(&"[\\._]".to_string()).unwrap().replace_all(&" ".to_string(), name).to_string();
        let (mut ctype, mut title, mut extra) = ("movie".to_string(), clean, "0".to_string());
        for pat in Constants.TV_PATTERNS.iter() {
            let mut r#match = pat.search(clean);
            if r#match {
                let mut ctype = "tv".to_string();
                let mut title = (r#match.group("show_title".to_string()) || clean);
                let (mut s, mut e) = (r#match.group("season".to_string()), r#match.group("episode".to_string()));
                if (s && e) {
                    let mut extra = format!("S{:02}E{:02}", s.to_string().parse::<i64>().unwrap_or(0), e.to_string().parse::<i64>().unwrap_or(0));
                }
                break;
            }
        }
        (/* title */ title.trim().to_string().to_string(), ctype, extra)
    }
    /// Detect HDR and Dolby Vision from video stream metadata.
    pub fn _detect_hdr(vid: HashMap<String, serde_json::Value>) -> (bool, bool) {
        // Detect HDR and Dolby Vision from video stream metadata.
        let (mut is_hdr, mut is_dv) = (false, false);
        let mut color_transfer = vid.get(&"color_transfer".to_string()).cloned().unwrap_or("".to_string());
        let mut color_primaries = vid.get(&"color_primaries".to_string()).cloned().unwrap_or("".to_string());
        if (("smpte2084".to_string(), "arib-std-b67".to_string()).contains(&color_transfer) || color_primaries == "bt2020".to_string()) {
            let mut is_hdr = true;
        }
        for side_data in vid.get(&"side_data_list".to_string()).cloned().unwrap_or(vec![]).iter() {
            if (side_data.to_string().to_lowercase().contains(&"dovi".to_string()) || side_data.to_string().to_lowercase().contains(&"dolby".to_string())) {
                // TODO: is_dv = is_hdr = true
                break;
            }
        }
        (is_hdr, is_dv)
    }
    /// Probe a media file with FFprobe and return parsed metadata.
    pub async fn _get_metadata(&mut self, path: PathBuf) -> Result<Option<HashMap>> {
        // Probe a media file with FFprobe and return parsed metadata.
        let mut cmd = vec![self.ffprobe_bin, "-v".to_string(), "error".to_string(), "-print_format".to_string(), "json".to_string(), "-show_format".to_string(), "-show_streams".to_string(), path.to_string()];
        let (mut rc, mut out, _) = self.pm.run_command(cmd, Constants.STRICT_TIMEOUT).await;
        if (rc != 0 || !out) {
            None
        }
        // try:
        {
            let mut data = serde_json::from_str(&out).unwrap();
            let mut vid = next(data.get(&"streams".to_string()).cloned().unwrap_or(vec![]).iter().filter(|s| s.get(&"codec_type".to_string()).cloned() == "video".to_string()).map(|s| s).collect::<Vec<_>>(), None);
            if !vid {
                None
            }
            let mut fmt = data.get(&"format".to_string()).cloned().unwrap_or(HashMap::new());
            let (mut width, mut height) = (vid.get(&"width".to_string()).cloned().unwrap_or(0).to_string().parse::<i64>().unwrap_or(0), vid.get(&"height".to_string()).cloned().unwrap_or(0).to_string().parse::<i64>().unwrap_or(0));
            let mut duration = fmt.get(&"duration".to_string()).cloned().unwrap_or(0).to_string().parse::<f64>().unwrap_or(0.0);
            let mut bitrate = (vid.get(&"bit_rate".to_string()).cloned() || fmt.get(&"bit_rate".to_string()).cloned() || 0).to_string().parse::<f64>().unwrap_or(0.0);
            if (duration > 0 && bitrate == 0) {
                let mut bitrate = ((path.stat().st_size * 8) / duration);
            }
            // try:
            {
                let (mut n, mut d) = vid.get(&"avg_frame_rate".to_string()).cloned().unwrap_or("0/0".to_string()).split("/".to_string()).map(|s| s.to_string()).collect::<Vec<String>>().iter().map(float).collect::<Vec<_>>();
                let mut fps = if d > 0 { (n / d) } else { 0 };
            }
            // except (ValueError, ZeroDivisionError) as _e:
            let mut bpp = if ((width * height) * fps) > 0 { (bitrate / ((width * height) * fps)) } else { 0 };
            let mut auds = data.get(&"streams".to_string()).cloned().unwrap_or(vec![]).iter().filter(|s| s.get(&"codec_type".to_string()).cloned() == "audio".to_string()).map(|s| s).collect::<Vec<_>>();
            let mut best_aud = max(auds, /* key= */ |x| x.get(&"channels".to_string()).cloned().unwrap_or(0).to_string().parse::<i64>().unwrap_or(0), /* default= */ HashMap::new());
            let (mut is_hdr, mut is_dv) = self._detect_hdr(vid);
            HashMap::from([("width".to_string(), width), ("height".to_string(), height), ("duration".to_string(), duration), ("bitrate".to_string(), bitrate), ("video_codec".to_string(), vid.get(&"codec_name".to_string()).cloned().unwrap_or("unknown".to_string())), ("bit_depth".to_string(), if vid.get(&"bits_per_raw_sample".to_string()).cloned().unwrap_or("".to_string()).chars().all(|c| c.is_ascii_digit()) { vid.get(&"bits_per_raw_sample".to_string()).cloned().unwrap_or(8).to_string().parse::<i64>().unwrap_or(0) } else { 8 }), ("audio_channels".to_string(), best_aud.get(&"channels".to_string()).cloned().unwrap_or(0).to_string().parse::<i64>().unwrap_or(0)), ("audio_codec".to_string(), best_aud.get(&"codec_name".to_string()).cloned().unwrap_or("unknown".to_string())), ("audio_streams_count".to_string(), auds.len()), ("bpp".to_string(), bpp), ("subs_count".to_string(), data.get(&"streams".to_string()).cloned().unwrap_or(vec![]).iter().filter(|s| s.get(&"codec_type".to_string()).cloned() == "subtitle".to_string()).map(|s| s).collect::<Vec<_>>().len()), ("is_hdr".to_string(), is_hdr), ("is_dolby_vision".to_string(), is_dv)])
        }
        // except (json::JSONDecodeError, KeyError, ValueError, TypeError) as _e:
    }
    /// Construct a MediaFile from path, metadata dict, and optional hashes.
    pub fn _build_media_file(&mut self, path: PathBuf, meta: HashMap<String, serde_json::Value>, hashes: Option<Vec<i64>>) -> Result<MediaFile> {
        // Construct a MediaFile from path, metadata dict, and optional hashes.
        let (mut title, mut ctype, mut extra) = self._parse_filename(path);
        let mut codec_mult = if Constants.EFFICIENT_CODECS.contains(&meta["video_codec".to_string()]) { Constants.CODEC_EFFICIENCY_MULTIPLIER } else { 1.0_f64 };
        let mut quality_score = ((meta["bitrate".to_string()] * codec_mult) / 1.0_f64.max(meta["duration".to_string()]));
        let mut hdr_bonus = if meta.get(&"is_dolby_vision".to_string()).cloned() { 2000 } else { if meta.get(&"is_hdr".to_string()).cloned() { 1000 } else { 0 } };
        let mut score = ((meta["width".to_string()] * meta["height".to_string()]), (hdr_bonus + if meta["bit_depth".to_string()] >= 10 { 1000 } else { 0 }), (meta["audio_channels".to_string()] * 100), quality_score);
        // try:
        {
            let mut stat = path.stat();
            let (mut created, mut modified) = (stat.st_ctime, stat.st_mtime);
        }
        // except OSError as _e:
        Ok(MediaFile(/* path= */ path, /* name= */ path.file_name().unwrap_or_default().to_str().unwrap_or(""), /* size= */ path.stat().st_size, /* content_type= */ ctype, /* title= */ title, /* width= */ meta["width".to_string()], /* height= */ meta["height".to_string()], /* duration= */ meta["duration".to_string()], /* bitrate= */ meta["bitrate".to_string()], /* video_codec= */ meta["video_codec".to_string()], /* bit_depth= */ meta["bit_depth".to_string()], /* audio_channels= */ meta["audio_channels".to_string()], /* audio_codec= */ meta["audio_codec".to_string()], /* audio_streams_count= */ meta.get(&"audio_streams_count".to_string()).cloned().unwrap_or(1), /* bpp= */ meta["bpp".to_string()], /* subs_count= */ meta["subs_count".to_string()], /* visual_hashes= */ hashes, /* sort_score= */ score, /* created= */ created, /* modified= */ modified, /* is_hdr= */ meta.get(&"is_hdr".to_string()).cloned().unwrap_or(false), /* is_dolby_vision= */ meta.get(&"is_dolby_vision".to_string()).cloned().unwrap_or(false)))
    }
    /// Scan configured directories and return media files grouped by title.
    pub async fn scan(&mut self) -> HashMap<tuple, Vec<MediaFile>> {
        // Scan configured directories and return media files grouped by title.
        let mut files_to_scan = vec![];
        safe_print("--- Phase 1: Finding Files ---".to_string());
        for src in self.config.source_dirs.iter() {
            if !src.exists() {
                continue;
            }
            safe_print(format!(" Scan: {}", src));
            for ext in self.config.video_extensions.iter() {
                for path in src.rglob(format!("*.{}", ext)).iter() {
                    if !self.non_movie_regexes.iter().map(|r| r.search(path.file_name().unwrap_or_default().to_str().unwrap_or(""))).collect::<Vec<_>>().iter().any(|v| *v) {
                        files_to_scan.push(path);
                        if (files_to_scan.len() % 50) == 0 {
                            safe_print(format!("\r  Found {} files...", files_to_scan.len()));
                        }
                    }
                }
            }
        }
        safe_print();
        safe_print(format!("--- Phase 2: Analyzing {} Files ---", files_to_scan.len()));
        self.cache.begin_transaction();
        let mut groups = defaultdict(list);
        let mut sem = asyncio.Semaphore(self.config.max_workers);
        let _process = |path| {
            // Fetch or probe metadata for a single file and group it.
            let _ctx = sem;
            {
                if !path.exists() {
                    return;
                }
                let mut cached = self.cache.get(&path).cloned();
                let (mut meta, mut hashes) = if cached { (cached["meta".to_string()], cached["fingerprint".to_string()]) } else { (None, None) };
                if !meta {
                    let mut meta = self._get_metadata(path).await;
                    if meta {
                        self.cache.set(path, meta, None, /* commit= */ false);
                    }
                }
                if !meta {
                    return;
                }
                let mut mf = self._build_media_file(path, meta, hashes);
                groups[(mf.content_type, mf.title, self._parse_filename(path)[2])].push(mf);
            }
        };
        let mut tasks = files_to_scan.iter().map(|p| _process(p)).collect::<Vec<_>>();
        if HAS_TQDM {
            tqdm_asyncio.gather(/* *tasks */, /* desc= */ "Analyzing".to_string(), /* unit= */ "file".to_string()).await;
        } else {
            asyncio.gather(/* *tasks */).await;
        }
        self.cache.commit_transaction();
        groups
    }
}

/// Runs fingerprinting in background while UI is active.
#[derive(Debug, Clone)]
pub struct BackgroundFingerprinter {
    pub fingerprinter: String,
    pub cache: String,
    pub candidates: String,
    pub pairs: String,
    pub on_match: String,
    pub progress: String,
    pub running: bool,
    pub _lock: std::sync::Mutex<()>,
}

impl BackgroundFingerprinter {
    /// Set up background fingerprinting for the given candidates.
    pub fn new(fingerprinter: FingerprintGenerator, cache: MetadataCache, candidates: Vec<MediaFile>, pairs: Vec<tuple>, on_match: Box<dyn Fn>) -> Self {
        Self {
            fingerprinter,
            cache,
            candidates,
            pairs,
            on_match,
            progress: (0, candidates.len()),
            running: true,
            _lock: std::sync::Mutex::new(()),
        }
    }
    /// Return (completed, total) fingerprinting progress.
    pub fn get_progress(&self) -> (i64, i64) {
        // Return (completed, total) fingerprinting progress.
        let _ctx = self._lock;
        {
            self.progress
        }
    }
    /// Signal the fingerprinter to stop processing.
    pub fn stop(&mut self) -> () {
        // Signal the fingerprinter to stop processing.
        self.running = false;
    }
    /// Run fingerprinting and check for matches.
    pub async fn run(&mut self) -> () {
        // Run fingerprinting and check for matches.
        let mut sem = asyncio.Semaphore(4);
        let _gen_fp = |f| {
            // Generate and store a fingerprint for a single media file.
            if (!self.running || f.visual_hashes) {
                return;
            }
            let _ctx = sem;
            {
                let mut hashes = self.fingerprinter.generate(f.path, f.duration).await;
                if hashes {
                    f.visual_hashes = hashes;
                    self.cache.set(f.path, f.to_meta_dict(), hashes);
                }
                let _ctx = self._lock;
                {
                    self.progress = ((self.progress[0] + 1), self.progress[1]);
                }
            }
        };
        let mut batch_size = 10;
        for i in (0..self.candidates.len()).step_by(batch_size as usize).iter() {
            if !self.running {
                break;
            }
            asyncio.gather(/* *self.candidates[i..(i + batch_size)].iter().map(|f| _gen_fp(f)).collect::<Vec<_>>() */).await;
            for (f1, f2) in self.pairs.iter() {
                if (f1.visual_hashes && f2.visual_hashes) {
                    let mut matches = 0..3.iter().filter(|k| DuplicateDetector.hamming_distance(f1.visual_hashes[&k], f2.visual_hashes[&k]) <= Constants.VISUAL_MATCH_THRESHOLD).map(|k| 1).collect::<Vec<_>>().iter().sum::<i64>();
                    if matches >= 2 {
                        self.on_match(f1, f2);
                    }
                }
            }
        }
    }
}

/// Handles duplicate detection and user interaction.
#[derive(Debug, Clone)]
pub struct DuplicateDetector {
    pub config: String,
    pub logger: String,
    pub cache: String,
    pub fingerprinter: String,
    pub history: Vec<serde_json::Value>,
    pub auto_play: bool,
    pub bg_fingerprinter: Option<BackgroundFingerprinter>,
    pub visual_match_queue: Queue,
}

impl DuplicateDetector {
    /// Initialize the duplicate detector with configuration and services.
    pub fn new(config: Config, logger: logging::Logger, cache: MetadataCache, fingerprinter: FingerprintGenerator) -> Self {
        Self {
            config,
            logger,
            cache,
            fingerprinter,
            history: vec![],
            auto_play: false,
            bg_fingerprinter: None,
            visual_match_queue: Default::default(),
        }
    }
    /// Return the Hamming distance between two integer hashes.
    pub fn hamming_distance(h1: i64, h2: i64) -> i64 {
        // Return the Hamming distance between two integer hashes.
        format!("0b{:b}", (h1 ^ h2)).iter().filter(|v| **v == "1".to_string()).count()
    }
    /// Queue a visual match pair for later processing.
    pub fn _on_visual_match(&self, f1: MediaFile, f2: MediaFile) -> () {
        // Queue a visual match pair for later processing.
        self.visual_match_queue.put((f1, f2));
    }
    /// Collect files needing fingerprinting and their candidate pairs.
    pub fn _collect_fingerprint_candidates(&self, singles: Vec<MediaFile>) -> (Vec<MediaFile>, Vec<tuple>) {
        // Collect files needing fingerprinting and their candidate pairs.
        let (mut candidates, mut pairs) = (vec![], vec![]);
        for (i, f1) in singles.iter().enumerate().iter() {
            let mut max_diff = (f1.duration * Constants.DURATION_TOLERANCE_PERCENT);
            for j in (i + 1)..singles.len().iter() {
                let mut f2 = singles[&j];
                if (f2.duration - f1.duration) > max_diff {
                    break;
                }
                pairs.push((f1, f2));
                if (!candidates.contains(&f1) && !f1.visual_hashes) {
                    candidates.push(f1);
                }
                if (!candidates.contains(&f2) && !f2.visual_hashes) {
                    candidates.push(f2);
                }
            }
        }
        (candidates, pairs)
    }
    /// Process any visual matches found by background fingerprinter.
    pub async fn _drain_visual_matches(&mut self) -> Result<()> {
        // Process any visual matches found by background fingerprinter.
        while !self.visual_match_queue.empty() {
            // try:
            {
                let (mut f1, mut f2) = self.visual_match_queue.get_nowait();
                let mut new_key = (f1.content_type, format!("VISUAL: {} / {}", f1.title, f2.title), "Match".to_string());
                self._handle_group(0, 1, new_key, vec![f1, f2]).await;
            }
            // except Empty as _e:
        }
    }
    /// Process duplicate groups and handle user interaction.
    pub async fn process(&mut self, groups: HashMap<tuple, Vec<MediaFile>>) -> () {
        // Process duplicate groups and handle user interaction.
        let mut ready_groups = groups.iter().iter().filter(|(k, v)| v.len() > 1).map(|(k, v)| (k, v)).collect::<HashMap<_, _>>();
        let mut singles = groups.iter().iter().filter(|(k, v)| v.len() == 1).map(|(k, v)| v[0]).collect::<Vec<_>>();
        singles.sort(/* key= */ |x| x.duration);
        let (mut candidates_to_fingerprint, mut candidate_pairs) = (vec![], vec![]);
        if Constants.ENABLE_VISUAL_MATCHING {
            let (mut candidates_to_fingerprint, mut candidate_pairs) = self._collect_fingerprint_candidates(singles);
        }
        if candidates_to_fingerprint {
            safe_print(format!("--- Starting background fingerprinting for {} files ---", candidates_to_fingerprint.len()));
            self.bg_fingerprinter = BackgroundFingerprinter(self.fingerprinter, self.cache, candidates_to_fingerprint, candidate_pairs, self._on_visual_match);
            asyncio.create_task(self.bg_fingerprinter.run());
        }
        let mut total_waste = ready_groups.values().iter().filter(|files| files).map(|files| files[1..].iter().map(|f| f.size).collect::<Vec<_>>().iter().sum::<i64>()).collect::<Vec<_>>().iter().sum::<i64>();
        for files in ready_groups.values().iter() {
            files.sort(/* key= */ |f| f.sort_score, /* reverse= */ true);
        }
        safe_print(format!("\n{}", ("=".to_string() * 60)));
        safe_print(format!(" Found {} duplicate groups (ready now)", ready_groups.len()));
        if candidates_to_fingerprint {
            safe_print(format!(" + {} files being fingerprinted in background", candidates_to_fingerprint.len()));
        }
        safe_print(format!(" Potential savings: {}", format_file_size(total_waste)));
        safe_print(format!("{}\n", ("=".to_string() * 60)));
        let mut sorted_groups = { let mut v = ready_groups.iter().clone(); v.sort(); v };
        for (i, (key, files)) in sorted_groups.iter().enumerate().iter() {
            self._handle_group(i, sorted_groups.len(), key, files).await;
            self._drain_visual_matches().await;
        }
        if self.bg_fingerprinter {
            self.bg_fingerprinter.stop();
        }
    }
    /// Display group information and file listing to the console.
    pub fn _display_group_info(&self, idx: i64, total: i64, title: String, extra: String, files: Vec<MediaFile>, savings: i64) -> () {
        // Display group information and file listing to the console.
        safe_print(format!("\n{}\n[{}/{}] {} ({}) | Savings: {}\n{}", ("=".to_string() * 80), (idx + 1), total, title, extra, format_file_size(savings), ("=".to_string() * 80)));
        let mut by_dir = defaultdict(list);
        for (j, f) in files.iter().enumerate().iter() {
            by_dir[&f.path.parent().unwrap_or(std::path::Path::new(""))].push((j, f));
        }
        for (parent, items) in by_dir.iter().iter() {
            safe_print(format!(" Folder: {}", parent));
            for (j, f) in items.iter() {
                let mut marker = if j == 0 { " [BEST]".to_string() } else { "".to_string() };
                safe_print(format!("   {}.{} {}", (j + 1), marker, f.get_info_string(f.name)));
            }
            safe_print(("-".to_string() * 40));
        }
        safe_print("(k #) Keep, (d #) Recycle, (p) Play, (s) Skip, (u) Undo, (q) Quit".to_string());
    }
    /// Execute a user command. Returns 'break', 'continue', or None.
    pub fn _execute_command(&mut self, cmd: String, parts: Vec<String>, files: Vec<MediaFile>) -> Result<Option<String>> {
        // Execute a user command. Returns 'break', 'continue', or None.
        if cmd == "q".to_string() {
            if self.bg_fingerprinter {
                self.bg_fingerprinter.stop();
            }
            return Err(anyhow::anyhow!("SystemExit(0)"));
        }
        if cmd == "s".to_string() {
            "break".to_string()
        }
        if cmd == "u".to_string() {
            self.undo_last_group();
            "continue".to_string()
        }
        if cmd == "p".to_string() {
            self.auto_play = true;
            "continue".to_string()
        }
        if (("k".to_string(), "d".to_string()).contains(&cmd) && parts.len() > 1 && parts[1].chars().all(|c| c.is_ascii_digit())) {
            let mut target_idx = (parts[1].to_string().parse::<i64>().unwrap_or(0) - 1);
            if (0 <= target_idx) && (target_idx < files.len()) {
                if cmd == "k".to_string() {
                    self._keep_file(files[&target_idx], files);
                    "break".to_string()
                }
                self._recycle_file(files[&target_idx]);
                files.remove(&target_idx);
                if files.len() < 2 {
                    "break".to_string()
                }
            }
        }
        Ok(None)
    }
    /// Attempt VLC auto-play. Returns 'keep', 'skip', or None.
    pub fn _try_vlc_autoplay(&mut self, files: Vec<MediaFile>) -> Option<String> {
        // Attempt VLC auto-play. Returns 'keep', 'skip', or None.
        if !(self.auto_play && HAS_VLC) {
            None
        }
        let mut bg_progress = if self.bg_fingerprinter { self.bg_fingerprinter.get_progress() } else { None };
        let mut result = VLCPlayer.launch(files, self.fingerprinter.ffmpeg_bin, bg_progress);
        if !result {
            self.auto_play = false;
            None
        }
        if result.get(&"action".to_string()).cloned() == "keep".to_string() {
            let mut target_idx = result["index".to_string()];
            if (0 <= target_idx) && (target_idx < files.len()) {
                self._keep_file(files[&target_idx], files);
                "keep".to_string()
            }
        }
        if result.get(&"action".to_string()).cloned() == "skip".to_string() {
            "skip".to_string()
        }
        self.auto_play = false;
        None
    }
    /// Handle a single duplicate group with user interaction.
    pub async fn _handle_group(&mut self, idx: i64, total: i64, key: tuple, files: Vec<MediaFile>) -> Result<()> {
        // Handle a single duplicate group with user interaction.
        let (mut title, mut extra) = (key[1], key[2]);
        let mut savings = (files.iter().map(|f| f.size).collect::<Vec<_>>().iter().sum::<i64>() - files.iter().map(|f| f.size).collect::<Vec<_>>().iter().max().unwrap());
        while true {
            let mut vlc_result = self._try_vlc_autoplay(files);
            if ("keep".to_string(), "skip".to_string()).contains(&vlc_result) {
                return;
            }
            self._display_group_info(idx, total, title, extra, files, savings);
            // try:
            {
                let mut choice = asyncio.to_thread(input, "Choice: ".to_string()).await;
            }
            // except EOFError as _e:
            let mut parts = choice.to_lowercase().split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
            if !parts {
                continue;
            }
            let mut action = self._execute_command(parts[0], parts, files);
            if action == "break".to_string() {
                break;
            }
            if action == "continue".to_string() {
                continue;
            }
        }
    }
    /// Move a file to the recycle directory.
    pub fn _recycle_file(&mut self, file: MediaFile) -> Result<()> {
        // Move a file to the recycle directory.
        safe_print(format!(">> Recycling: {}", file.name));
        if self.config.dry_run {
            safe_print("[DRY RUN] File would be recycled".to_string());
            return;
        }
        // try:
        {
            let mut dst = (self.config.user_recycle_dir / file.name);
            let mut counter = 1;
            while dst.exists() {
                let mut dst = (self.config.user_recycle_dir / format!("{}_{}{}", dst.file_stem().unwrap_or_default().to_str().unwrap_or(""), counter, dst.extension().unwrap_or_default().to_str().unwrap_or("")));
                counter += 1;
            }
            // try:
            {
                std::fs::rename(file.path.to_string(), dst.to_string());
            }
            // except OSError as _e:
            self.logger.info(format!("Recycled {}", file.path));
            self.history.push(HashMap::from([("type".to_string(), "recycle".to_string()), ("original_path".to_string(), file.path), ("temp_path".to_string(), dst), ("timestamp".to_string(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64())]));
        }
        // except OSError as e:
    }
    /// Keep the chosen file and recycle all others in the group.
    pub fn _keep_file(&mut self, keep: MediaFile, all_files: Vec<MediaFile>) -> () {
        // Keep the chosen file and recycle all others in the group.
        safe_print(format!(">> Keeping: {}", keep.name));
        self.history.push(HashMap::from([("type".to_string(), "group_start".to_string()), ("timestamp".to_string(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64())]));
        for other in all_files.iter() {
            if other != keep {
                self._recycle_file(other);
            }
        }
    }
    /// Undo the most recent group keep/recycle actions.
    pub fn undo_last_group(&mut self) -> Result<()> {
        // Undo the most recent group keep/recycle actions.
        if !self.history {
            safe_print("Nothing to undo.".to_string());
            return;
        }
        safe_print("Undoing last actions...".to_string());
        let mut count = 0;
        while self.history {
            let mut item = self.history.pop().unwrap();
            if item["type".to_string()] == "group_start".to_string() {
                break;
            }
            if (item["type".to_string()] == "recycle".to_string() && item["temp_path".to_string()].exists()) {
                // try:
                {
                    std::fs::rename(item["temp_path".to_string()].to_string(), item["original_path".to_string()].to_string());
                    safe_print(format!("Restored: {}", item["original_path".to_string()].name));
                    count += 1;
                }
                // except OSError as e:
            }
        }
        Ok(safe_print(format!("Undo complete. Restored {} files.", count)))
    }
    /// For MediaFile - moved here to keep MediaFile clean.
    pub fn get_info_string(&self, display_name: Option<String>) -> String {
        // For MediaFile - moved here to keep MediaFile clean.
        // pass
    }
}

/// Side-by-side frame comparison with zoom/pan using FFmpeg.
#[derive(Debug, Clone)]
pub struct ZoomCompareWindow {
    pub files: String,
    pub timestamp: String,
    pub ffmpeg_bin: String,
    pub zoom_level: f64,
    pub pan_offset: Vec<i64>,
    pub images: Vec<serde_json::Value>,
    pub cached_resized: HashMap<String, serde_json::Value>,
    pub photo_images: Vec<serde_json::Value>,
    pub root: Option<serde_json::Value>,
    pub canvases: Vec<serde_json::Value>,
    pub dragging: bool,
    pub drag_start: String,
}

impl ZoomCompareWindow {
    /// Prepare a zoom-comparison window for the given files.
    pub fn new(files: Vec<MediaFile>, timestamp: f64, ffmpeg_bin: String) -> Self {
        Self {
            files: files[..2],
            timestamp,
            ffmpeg_bin,
            zoom_level: 1.0_f64,
            pan_offset: vec![0, 0],
            images: vec![],
            cached_resized: HashMap::new(),
            photo_images: vec![],
            root: None,
            canvases: vec![],
            dragging: false,
            drag_start: (0, 0),
        }
    }
    /// Extract a single frame from each file using FFmpeg.
    pub fn extract_frames(&mut self) -> Result<bool> {
        // Extract a single frame from each file using FFmpeg.
        if !HAS_PILLOW {
            false
        }
        self.images = vec![];
        for mf in self.files.iter() {
            let mut cmd = vec![self.ffmpeg_bin, "-ss".to_string(), self.timestamp.to_string(), "-i".to_string(), mf.path.to_string(), "-vframes".to_string(), "1".to_string(), "-f".to_string(), "image2pipe".to_string(), "-vcodec".to_string(), "png".to_string(), "-".to_string()];
            // try:
            {
                let mut proc = subprocess::Popen(cmd, /* stdout= */ subprocess::PIPE, /* stderr= */ subprocess::PIPE, /* creationflags= */ if platform.system() == "Windows".to_string() { subprocess::CREATE_NEW_PROCESS_GROUP } else { 0 });
                let (mut stdout, _) = proc.communicate(/* timeout= */ 10);
                if stdout {
                    self.images.push(Image.open(io.BytesIO(stdout)));
                } else {
                    false
                }
            }
            // except (OSError, subprocess::SubprocessError) as e:
        }
        Ok(self.images.len() == self.files.len())
    }
    /// Create side-by-side canvas panels with mouse bindings for each image.
    pub fn _build_canvas_panels(&mut self, container: String) -> () {
        // Create side-by-side canvas panels with mouse bindings for each image.
        for (i, (img, mf)) in self.images.iter().zip(self.files.iter()).iter().enumerate().iter() {
            let mut frame = tkinter.Frame(container, /* bg= */ "#1a1a1a".to_string(), /* highlightthickness= */ 2, /* highlightbackground= */ if i == 0 { "#2ea043".to_string() } else { "#333".to_string() });
            frame.grid(/* row= */ 0, /* column= */ i, /* sticky= */ "nsew".to_string(), /* padx= */ 3, /* pady= */ 3);
            container.grid_columnconfigure(i, /* weight= */ 1, /* uniform= */ "zoom_col".to_string());
            container.grid_rowconfigure(0, /* weight= */ 1);
            tkinter.Label(frame, /* text= */ format!("#{}: {}", (i + 1), mf.name[..50]), /* bg= */ "#1a1a1a".to_string(), /* fg= */ "#fff".to_string(), /* font= */ Constants.FONTS["mono_bold".to_string()]).pack(/* fill= */ tkinter.X);
            let mut canvas = tkinter.Canvas(frame, /* bg= */ "black".to_string(), /* highlightthickness= */ 0);
            canvas.pack(/* fill= */ tkinter.BOTH, /* expand= */ true);
            self.canvases.push(canvas);
            canvas.bind("<MouseWheel>".to_string(), |e| self._render(if e.delta > 0 { 1.1_f64 } else { 0.9_f64 }));
            canvas.bind("<Button-4>".to_string(), |e| self._render(1.1_f64));
            canvas.bind("<Button-5>".to_string(), |e| self._render(0.9_f64));
            canvas.bind("<ButtonPress-1>".to_string(), self._start_drag);
            canvas.bind("<B1-Motion>".to_string(), self._on_drag);
            canvas.bind("<ButtonRelease-1>".to_string(), self._end_drag);
        }
    }
    /// Build and display the zoom-comparison window.
    pub fn run(&mut self) -> () {
        // Build and display the zoom-comparison window.
        if !self.extract_frames() {
            return;
        }
        self.root = tkinter.Toplevel();
        self.root.title(format!("Zoom Compare @ {:.1}s", self.timestamp));
        self.root.configure(/* bg= */ "#1e1e1e".to_string());
        let (mut screen_w, mut screen_h) = (self.root.winfo_screenwidth(), self.root.winfo_screenheight());
        self.root.geometry(format!("{}x{}", (screen_w * 0.9_f64).to_string().parse::<i64>().unwrap_or(0), (screen_h * 0.85_f64).to_string().parse::<i64>().unwrap_or(0)));
        for (key, r#fn) in vec![("+".to_string(), |e| self._render(1.25_f64)), ("-".to_string(), |e| self._render(0.8_f64)), ("=".to_string(), |e| self._render(1.25_f64)), ("r".to_string(), |e| self._reset()), ("<Escape>".to_string(), |e| self.root.destroy()), ("<Left>".to_string(), |e| self._pan(-50, 0)), ("<Right>".to_string(), |e| self._pan(50, 0)), ("<Up>".to_string(), |e| self._pan(0, -50)), ("<Down>".to_string(), |e| self._pan(0, 50))].iter() {
            self.root.bind(key, r#fn);
        }
        let mut info = tkinter.Frame(self.root, /* bg= */ "#252526".to_string());
        info.pack(/* fill= */ tkinter.X);
        tkinter.Label(info, /* text= */ "Zoom: +/- | Pan: Arrows/Drag | Reset: R | Close: Esc".to_string(), /* bg= */ "#252526".to_string(), /* fg= */ "#888".to_string(), /* font= */ ("Segoe UI".to_string(), 9)).pack(/* pady= */ 5);
        let mut container = tkinter.Frame(self.root, /* bg= */ "#1e1e1e".to_string());
        container.pack(/* fill= */ tkinter.BOTH, /* expand= */ true, /* padx= */ 5, /* pady= */ 5);
        self._build_canvas_panels(container);
        let mut ctrl = tkinter.Frame(self.root, /* bg= */ "#252526".to_string());
        ctrl.pack(/* fill= */ tkinter.X);
        self.zoom_label = tkinter.Label(ctrl, /* text= */ "Zoom: 100%".to_string(), /* bg= */ "#252526".to_string(), /* fg= */ "#fff".to_string(), /* font= */ Constants.FONTS["mono_bold".to_string()]);
        self.zoom_label.pack(/* side= */ tkinter.LEFT, /* padx= */ 10, /* pady= */ 5);
        for (text, cmd, color) in vec![("Zoom +".to_string(), || self._render(1.25_f64), "#333".to_string()), ("Zoom -".to_string(), || self._render(0.8_f64), "#333".to_string()), ("Reset".to_string(), self._reset, "#333".to_string()), ("Close".to_string(), self.root.destroy, "#c42b1c".to_string())].iter() {
            tkinter.Button(ctrl, /* text= */ text, /* command= */ cmd, /* bg= */ color, /* fg= */ "#fff".to_string(), /* relief= */ tkinter.FLAT).pack(/* side= */ if text != "Close".to_string() { tkinter.LEFT } else { tkinter.RIGHT }, /* padx= */ 2);
        }
        self._render();
        self.root.focus_set();
    }
    /// COMBINED: Single method for zoom/pan updates.
    pub fn _render(&mut self, zoom_factor: f64, fast: bool) -> () {
        // COMBINED: Single method for zoom/pan updates.
        if zoom_factor {
            self.zoom_level = 0.1_f64.max(10.0_f64.min((self.zoom_level * zoom_factor)));
            self.zoom_label.configure(/* text= */ format!("Zoom: {}%", (self.zoom_level * 100).to_string().parse::<i64>().unwrap_or(0)));
            self.cached_resized.clear();
        }
        self.photo_images = vec![];
        let mut resample = if fast { Image.Resampling.BILINEAR } else { Image.Resampling.LANCZOS };
        for (i, (canvas, img)) in self.canvases.iter().zip(self.images.iter()).iter().enumerate().iter() {
            canvas.update_idletasks();
            let (mut cw, mut ch) = (canvas.winfo_width(), canvas.winfo_height());
            if (cw < 10 || ch < 10) {
                continue;
            }
            let mut cache_key = (i, self.zoom_level, cw, ch);
            if (self.cached_resized.contains(&cache_key) && !fast) {
                let mut resized = self.cached_resized[&cache_key];
            } else {
                let (mut img_w, mut img_h) = img.size;
                let mut scale = ((cw / img_w).min((ch / img_h)) * self.zoom_level);
                let (mut new_w, mut new_h) = ((img_w * scale).to_string().parse::<i64>().unwrap_or(0), (img_h * scale).to_string().parse::<i64>().unwrap_or(0));
                let mut resized = img.resize((new_w, new_h), resample);
                if !fast {
                    self.cached_resized[cache_key] = resized;
                }
            }
            let mut x = (((cw - resized.width) / 2) + self.pan_offset[0]);
            let mut y = (((ch - resized.height) / 2) + self.pan_offset[1]);
            let mut photo = ImageTk.PhotoImage(resized);
            self.photo_images.push(photo);
            canvas.delete("all".to_string());
            canvas.create_image(x, y, /* anchor= */ tkinter.NW, /* image= */ photo);
        }
    }
    /// Reset zoom level and pan offset.
    pub fn _reset(&mut self) -> () {
        // Reset zoom level and pan offset.
        let (self.zoom_level, self.pan_offset) = (1.0_f64, vec![0, 0]);
        self.zoom_label.configure(/* text= */ "Zoom: 100%".to_string());
        self.cached_resized.clear();
        self._render();
    }
    /// Pan the view by the given pixel offsets.
    pub fn _pan(&mut self, dx: i64, dy: i64) -> () {
        // Pan the view by the given pixel offsets.
        self.pan_offset[0] += dx;
        self.pan_offset[1] += dy;
        self._render(/* fast= */ true);
    }
    /// Record the starting point of a mouse drag.
    pub fn _start_drag(&mut self, event: String) -> () {
        // Record the starting point of a mouse drag.
        let (self.dragging, self.drag_start) = (true, (event.x, event.y));
    }
    /// Handle mouse drag to pan the zoom view.
    pub fn _on_drag(&mut self, event: String) -> () {
        // Handle mouse drag to pan the zoom view.
        if self.dragging {
            self.pan_offset[0] += (event.x - self.drag_start[0]);
            self.pan_offset[1] += (event.y - self.drag_start[1]);
            self.drag_start = (event.x, event.y);
            self._render(/* fast= */ true);
        }
    }
    /// Finish drag and re-render at full quality.
    pub fn _end_drag(&mut self, event: String) -> () {
        // Finish drag and re-render at full quality.
        self.dragging = false;
        self._render();
    }
}

/// Enhanced side-by-side video comparison player.
#[derive(Debug, Clone)]
pub struct VLCPlayerApp {
    pub media_files: String,
    pub ffmpeg_bin: String,
    pub bg_progress: String,
    pub root: Option<serde_json::Value>,
    pub players: Vec<serde_json::Value>,
    pub frames: Vec<serde_json::Value>,
    pub canvases: Vec<serde_json::Value>,
    pub is_paused: bool,
    pub is_muted: bool,
    pub slider_dragging: bool,
    pub slider_var: Option<serde_json::Value>,
    pub after_id: Option<serde_json::Value>,
    pub active_audio_idx: i64,
    pub result: Option<serde_json::Value>,
    pub best: HashMap<String, serde_json::Value>,
}

impl VLCPlayerApp {
    /// Initialize the VLC player comparison app.
    pub fn new(media_files: Vec<MediaFile>, ffmpeg_bin: String, bg_progress: Option<(i64, i64)>) -> Self {
        Self {
            media_files,
            ffmpeg_bin,
            bg_progress,
            root: None,
            players: vec![],
            frames: vec![],
            canvases: vec![],
            is_paused: false,
            is_muted: false,
            slider_dragging: false,
            slider_var: None,
            after_id: None,
            active_audio_idx: 0,
            result: None,
            best: HashMap::from([("res".to_string(), 0), ("br".to_string(), 0), ("ch".to_string(), 0), ("subs".to_string(), 0), ("depth".to_string(), 0), ("codec".to_string(), 0)]),
        }
    }
    /// Launch the side-by-side comparison player.
    pub fn run(&mut self) -> Result<Option<HashMap>> {
        // Launch the side-by-side comparison player.
        self.root = tkinter.Tk();
        self.root.title("Side-by-Side Comparison - Visual AI v6.1".to_string());
        self.root.configure(/* bg= */ "#1e1e1e".to_string());
        self.root.protocol("WM_DELETE_WINDOW".to_string(), self._cleanup);
        let (mut sw, mut sh) = (self.root.winfo_screenwidth(), self.root.winfo_screenheight());
        self.root.geometry(format!("{}x{}", (sw * 0.9_f64).to_string().parse::<i64>().unwrap_or(0), (sh * 0.85_f64).to_string().parse::<i64>().unwrap_or(0)));
        for (key, r#fn) in vec![("<space>".to_string(), |e| self._toggle_pause()), ("<Left>".to_string(), |e| self._seek_rel(-5)), ("<Right>".to_string(), |e| self._seek_rel(5)), ("m".to_string(), |e| self._mute_all()), ("q".to_string(), |e| self._confirm_exit()), ("s".to_string(), |e| self._skip_group()), ("z".to_string(), |e| self._open_zoom()), ("Z".to_string(), |e| self._open_zoom())].iter() {
            self.root.bind(key, r#fn);
        }
        let mut count = self.media_files.len();
        let (mut cols, mut rows) = (count.min(2), ((count + 1) / 2));
        // try:
        {
            let mut args = vec!["--no-xlib".to_string(), "--quiet".to_string(), "--no-video-title-show".to_string()];
            if platform.system() == "Windows".to_string() {
                args.push("--no-osd".to_string());
            }
            self.vlc_inst = vlc.Instance(/* *args */);
        }
        // except (OSError, AttributeError) as e:
        let mut cont = tkinter.Frame(self.root, /* bg= */ "#1e1e1e".to_string());
        cont.pack(/* fill= */ tkinter.BOTH, /* expand= */ true, /* padx= */ 5, /* pady= */ 5);
        for c in 0..cols.iter() {
            cont.grid_columnconfigure(c, /* weight= */ 1, /* uniform= */ "video_col".to_string());
        }
        for r in 0..rows.iter() {
            cont.grid_rowconfigure(r, /* weight= */ 1, /* uniform= */ "video_row".to_string());
        }
        for (i, mf) in self.media_files.iter().enumerate().iter() {
            _build_video_panel(self, cont, i, mf, cols, self.best);
        }
        self._build_controls();
        for (i, frame) in self.frames.iter().enumerate().iter() {
            let mut color = if i == self.active_audio_idx { "#007acc".to_string() } else { if i == 0 { "#2ea043".to_string() } else { "#1e1e1e".to_string() } };
            frame.configure(/* highlightbackground= */ color, /* highlightthickness= */ 5);
        }
        self._update_loop();
        self.root.mainloop();
        Ok(self.result)
    }
    /// Build the playback controls bar at the bottom of the window.
    pub fn _build_controls(&mut self) -> () {
        // Build the playback controls bar at the bottom of the window.
        let mut ctrl = tkinter.Frame(self.root, /* bg= */ "#2d2d2d".to_string(), /* pady= */ 8);
        ctrl.pack(/* fill= */ tkinter.X, /* side= */ tkinter.BOTTOM);
        if self.bg_progress {
            let (mut done, mut total) = self.bg_progress;
            tkinter.Label(ctrl, /* text= */ format!("Fingerprinting: {}/{} files...", done, total), /* bg= */ "#2d2d2d".to_string(), /* fg= */ "#ffaa00".to_string(), /* font= */ ("Segoe UI".to_string(), 9)).pack(/* pady= */ (0, 5));
        }
        self.slider_var = tkinter.DoubleVar();
        let mut slider = tkinter.Scale(ctrl, /* from_= */ 0, /* to= */ 100, /* orient= */ tkinter.HORIZONTAL, /* variable= */ self.slider_var, /* showvalue= */ 0, /* bg= */ "#2d2d2d".to_string(), /* fg= */ "#007acc".to_string(), /* troughcolor= */ "#404040".to_string(), /* activebackground= */ "#0099ff".to_string(), /* command= */ self._on_seek, /* highlightthickness= */ 0, /* bd= */ 0);
        slider.pack(/* fill= */ tkinter.X, /* padx= */ 20, /* pady= */ (0, 10));
        slider.bind("<ButtonPress-1>".to_string(), |e| /* setattr(self, "slider_dragging".to_string(), true) */);
        slider.bind("<ButtonRelease-1>".to_string(), |e| vec![/* setattr(self, "slider_dragging".to_string(), false) */, self._on_seek(slider.get())]);
        let mut bf = tkinter.Frame(ctrl, /* bg= */ "#2d2d2d".to_string());
        bf.pack();
        for (text, cmd, color) in vec![("⏯ Pause (Space)".to_string(), self._toggle_pause, "#3c3c3c".to_string()), ("🔇 Mute (M)".to_string(), self._mute_all, "#3c3c3c".to_string()), ("🔍 Zoom (Z)".to_string(), self._open_zoom, "#555".to_string()), ("⏭ Skip (S)".to_string(), self._skip_group, "#007acc".to_string()), ("❌ Close (Q)".to_string(), self._confirm_exit, "#c42b1c".to_string())].iter() {
            tkinter.Button(bf, /* text= */ text, /* command= */ cmd, /* bg= */ color, /* fg= */ "white".to_string(), /* relief= */ tkinter.FLAT, /* padx= */ 12, /* pady= */ 5, /* font= */ ("Segoe UI".to_string(), 9)).pack(/* side= */ tkinter.LEFT, /* padx= */ 3);
        }
        tkinter.Label(bf, /* text= */ "Click video for audio | ←→ Seek".to_string(), /* bg= */ "#2d2d2d".to_string(), /* fg= */ "#666".to_string(), /* font= */ ("Segoe UI".to_string(), 8)).pack(/* side= */ tkinter.LEFT, /* padx= */ 15);
    }
    /// Open the zoom-comparison window at the current playback position.
    pub fn _open_zoom(&mut self) -> Result<()> {
        // Open the zoom-comparison window at the current playback position.
        if (!HAS_PILLOW || !self.players) {
            return;
        }
        // try:
        {
            let mut pos = self.players[0].get_position();
            let mut timestamp = (pos * self.media_files[0].duration);
        }
        // except (OSError, AttributeError, IndexError) as _e:
        if !self.is_paused {
            self._toggle_pause();
        }
        Ok(ZoomCompareWindow(self.media_files, timestamp, self.ffmpeg_bin).run())
    }
    /// Prompt user to confirm keeping the selected video.
    pub fn _keep_file(&mut self, index: i64) -> () {
        // Prompt user to confirm keeping the selected video.
        if messagebox.askyesno("Confirm Keep".to_string(), format!("Keep video #{}\n'{}'\n\nand recycle the others?", (index + 1), self.media_files[&index].name)) {
            self.result = HashMap::from([("action".to_string(), "keep".to_string()), ("index".to_string(), index)]);
            if self.root {
                self.root.quit();
            }
            self._cleanup();
        }
    }
    /// Prompt user to confirm closing the player.
    pub fn _confirm_exit(&self) -> () {
        // Prompt user to confirm closing the player.
        if messagebox.askyesno("Confirm Exit".to_string(), "Close the player?".to_string()) {
            self._cleanup();
        }
    }
    /// Skip the current group without making a decision.
    pub fn _skip_group(&mut self) -> () {
        // Skip the current group without making a decision.
        self.result = HashMap::from([("action".to_string(), "skip".to_string())]);
        if self.root {
            self.root.quit();
        }
        self._cleanup();
    }
    /// Switch audio focus to the given player index.
    pub fn _focus_audio(&mut self, target_idx: i64) -> () {
        // Switch audio focus to the given player index.
        self.active_audio_idx = target_idx;
        for (i, p) in self.players.iter().enumerate().iter() {
            p.audio_set_mute(i != target_idx);
        }
        for (i, frame) in self.frames.iter().enumerate().iter() {
            let mut color = if i == self.active_audio_idx { "#007acc".to_string() } else { if i == 0 { "#2ea043".to_string() } else { "#1e1e1e".to_string() } };
            frame.configure(/* highlightbackground= */ color, /* highlightthickness= */ 5);
        }
    }
    /// Toggle play/pause for all players.
    pub fn _toggle_pause(&mut self) -> () {
        // Toggle play/pause for all players.
        self.is_paused = !self.is_paused;
        for p in self.players.iter() {
            p.set_pause(if self.is_paused { 1 } else { 0 });
        }
    }
    /// Toggle mute on all players.
    pub fn _mute_all(&mut self) -> () {
        // Toggle mute on all players.
        self.is_muted = !self.is_muted;
        if self.is_muted {
            for p in self.players.iter() {
                p.audio_set_mute(true);
            }
        } else {
            self._focus_audio(self.active_audio_idx);
        }
    }
    /// Seek all players to the position indicated by *val*.
    pub fn _on_seek(&self, val: String) -> () {
        // Seek all players to the position indicated by *val*.
        for p in self.players.iter() {
            if p.is_seekable() {
                p.set_position((val.to_string().parse::<f64>().unwrap_or(0.0) / 100.0_f64));
            }
        }
    }
    /// Seek relative to the current position by *delta* percent.
    pub fn _seek_rel(&mut self, delta: f64) -> Result<()> {
        // Seek relative to the current position by *delta* percent.
        if !self.players {
            return;
        }
        // try:
        {
            let mut cur = (self.players[0].get_position() * 100);
            let mut new_pos = 0.max(100.min((cur + delta)));
            self.slider_var.set(new_pos);
            self._on_seek(new_pos);
        }
        // except (OSError, AttributeError) as _e:
    }
    /// Periodically update the seek slider to match playback position.
    pub fn _update_loop(&mut self) -> Result<()> {
        // Periodically update the seek slider to match playback position.
        if (self.root && !self.slider_dragging && self.players) {
            // try:
            {
                let mut p = if self.active_audio_idx < self.players.len() { self.players[&self.active_audio_idx] } else { self.players[0] };
                let mut pos = p.get_position();
                if pos >= 0 {
                    self.slider_var.set((pos * 100));
                }
            }
            // except (OSError, AttributeError, IndexError) as _e:
        }
        if self.root {
            self.after_id = self.root.after(250, self._update_loop);
        }
    }
    /// Release all VLC players and destroy the window.
    pub fn _cleanup(&mut self) -> Result<()> {
        // Release all VLC players and destroy the window.
        if (self.after_id && self.root) {
            // try:
            {
                self.root.after_cancel(self.after_id);
            }
            // except (ValueError, tkinter.TclError) as _e:
        }
        for p in self.players.iter() {
            // try:
            {
                p.stop();
                p.release();
            }
            // except (OSError, AttributeError) as _e:
        }
        if self.root {
            self.root.destroy();
            self.root = None;
        }
    }
}

#[derive(Debug, Clone)]
pub struct VLCPlayer {
}

impl VLCPlayer {
    /// Create and run a VLC comparison player, returning the user's choice.
    pub fn launch(files: Vec<MediaFile>, ffmpeg_bin: String, bg_progress: Option<(i64, i64)>) -> Option<HashMap> {
        // Create and run a VLC comparison player, returning the user's choice.
        if !HAS_VLC {
            None
        }
        VLCPlayerApp(files, ffmpeg_bin, bg_progress).run()
    }
}

/// Top-level application orchestrating scan, detection, and user flow.
#[derive(Debug, Clone)]
pub struct VisualAIApp {
    pub config: Option<Config>,
    pub logger: Option<logging::Logger>,
    pub pm: Option<ProcessManager>,
    pub cache: Option<MetadataCache>,
}

impl VisualAIApp {
    /// Initialize application state.
    pub fn new() -> Self {
        Self {
            config: None,
            logger: None,
            pm: None,
            cache: None,
        }
    }
    /// Open a native folder-picker dialog and return the selected path.
    pub fn _pick_folder_dialog(&self) -> Result<Option<String>> {
        // Open a native folder-picker dialog and return the selected path.
        // try:
        {
            let mut root = tkinter.Tk();
            root.withdraw();
            root.attributes("-topmost".to_string(), true);
            let mut folder = filedialog.askdirectory(/* title= */ "Select folder to scan for video duplicates".to_string(), /* mustexist= */ true);
            root.destroy();
            if folder { folder } else { None }
        }
        // except (OSError, tkinter.TclError) as e:
    }
    /// Validate source directories, prompting for a folder if none are valid.
    pub fn _validate_source_dirs(&mut self, source_list: Vec<serde_json::Value>) -> Result<Vec<PathBuf>> {
        // Validate source directories, prompting for a folder if none are valid.
        let mut valid_sources = vec![];
        for src_path in source_list.iter() {
            let mut src = PathBuf::from(src_path);
            if !src.exists() {
                safe_print(format!("WARNING: Directory does not exist: {}", src));
                continue;
            }
            // try:
            {
                next(src.iterdir(), None);
                valid_sources.push(src);
            }
            // except (PermissionError, OSError) as e:
        }
        if valid_sources {
            valid_sources
        }
        safe_print(("\n".to_string() + ("=".to_string() * 60)));
        safe_print("No valid source directories configured.".to_string());
        safe_print(("=".to_string() * 60));
        let mut response = input("\nWould you like to pick a folder to scan? (y/n): ".to_string()).trim().to_string().to_lowercase();
        if response == "y".to_string() {
            let mut folder = self._pick_folder_dialog();
            if folder {
                safe_print(format!("Selected: {}", folder));
                vec![PathBuf::from(folder)]
            }
            safe_print("No folder selected.".to_string());
            return Err(anyhow::anyhow!("SystemExit(1)"));
        }
        safe_print("\nUsage: python Keep_1080p_VisualAI_v6_1.py -s <folder>".to_string());
        return Err(anyhow::anyhow!("SystemExit(1)"));
    }
    /// Build a Config from CLI arguments and optional config file.
    pub fn load_config(&mut self, args: String) -> Result<Config> {
        // Build a Config from CLI arguments and optional config file.
        let mut base_dir = PathBuf::from(file!()).canonicalize().unwrap_or_default().parent().unwrap_or(std::path::Path::new(""));
        let mut defaults = HashMap::from([("source_dirs".to_string(), vec![]), ("except_dir".to_string(), (base_dir / "Exceptions".to_string()).to_string()), ("user_recycle_dir".to_string(), (base_dir / "Recycled".to_string()).to_string()), ("exclude_keywords".to_string(), vec!["trailer".to_string(), "sample".to_string()]), ("video_extensions".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "wmv".to_string(), "mov".to_string(), "m4v".to_string(), "mpg".to_string(), "webm".to_string()])]);
        let mut cfg_path = (base_dir / "visual_ai_config.json".to_string());
        if cfg_path.exists() {
            // try:
            {
                let mut f = File::open(cfg_path)?;
                {
                    defaults.extend(json::load(f));
                }
            }
            // except (OSError, json::JSONDecodeError, ValueError) as e:
        }
        if (/* hasattr(args, "source_dirs".to_string()) */ true && args.source_dirs) {
            defaults["source_dirs".to_string()] = args.source_dirs;
        }
        let mut valid_sources = self._validate_source_dirs(defaults["source_dirs".to_string()]);
        for d in vec![defaults["except_dir".to_string()], defaults["user_recycle_dir".to_string()]].iter() {
            PathBuf::from(d).create_dir_all();
        }
        Ok(Config(/* source_dirs= */ valid_sources, /* except_dir= */ PathBuf::from(defaults["except_dir".to_string()]), /* user_recycle_dir= */ PathBuf::from(defaults["user_recycle_dir".to_string()]), /* exclude_keywords= */ defaults["exclude_keywords".to_string()], /* video_extensions= */ defaults["video_extensions".to_string()].into_iter().collect::<HashSet<_>>(), /* max_workers= */ /* getattr */ Constants.DEFAULT_MAX_WORKERS, /* dry_run= */ /* getattr */ false, /* log_file= */ if (/* hasattr(args, "log_file".to_string()) */ true && args.log_file) { PathBuf::from(args.log_file) } else { None }, /* verbose= */ /* getattr */ false, /* cache_db_path= */ (base_dir / "visual_ai_cache.db".to_string())))
    }
    /// Parse arguments, scan directories, and process duplicates.
    pub async fn run(&mut self) -> () {
        // Parse arguments, scan directories, and process duplicates.
        let mut parser = argparse.ArgumentParser(/* description= */ "Visual AI Duplicate Detector v6.1".to_string());
        parser.add_argument("-s".to_string(), "--source-dirs".to_string(), /* nargs= */ "+".to_string(), /* help= */ "Directories to scan".to_string());
        parser.add_argument("-w".to_string(), "--max-workers".to_string(), /* type= */ int, /* default= */ Constants.DEFAULT_MAX_WORKERS);
        parser.add_argument("-n".to_string(), "--dry-run".to_string(), /* action= */ "store_true".to_string());
        parser.add_argument("-l".to_string(), "--log-file".to_string());
        parser.add_argument("-v".to_string(), "--verbose".to_string(), /* action= */ "store_true".to_string());
        let mut args = parser.parse_args();
        self.config = self.load_config(args);
        self.logger = setup_logging(self.config.log_file, self.config.verbose);
        self.pm = ProcessManager(self.logger);
        self.cache = MetadataCache(self.config.cache_db_path, self.logger);
        if self.config.dry_run {
            safe_print("\n*** DRY RUN MODE ***\n".to_string());
        }
        let mut scanner = MediaScanner(self.config, self.pm, self.logger, self.cache);
        scanner.check_binaries().await;
        let mut groups = scanner.scan().await;
        let mut fingerprinter = FingerprintGenerator(self.pm, scanner.ffmpeg_bin, self.logger, self.config.max_workers);
        let mut detector = DuplicateDetector(self.config, self.logger, self.cache, fingerprinter);
        detector.process(groups).await;
        fingerprinter.shutdown();
        self.cache.close();
        safe_print("\nDone.".to_string());
    }
}

/// Thread-safe print with Unicode error handling.
pub fn safe_print(args: Vec<Box<dyn std::any::Any>>, kwargs: HashMap<String, Box<dyn std::any::Any>>) -> Result<()> {
    // Thread-safe print with Unicode error handling.
    let _ctx = PRINT_LOCK;
    {
        // try:
        {
            println!("{}", /* *args */, /* ** */ kwargs);
        }
        // except UnicodeEncodeError as _e:
        sys::stdout.flush();
    }
}

/// Format file size.
pub fn format_file_size(size_bytes: i64) -> String {
    // Format file size.
    let mut size = size_bytes.to_string().parse::<f64>().unwrap_or(0.0);
    for unit in vec!["B".to_string(), "KB".to_string(), "MB".to_string(), "GB".to_string(), "TB".to_string()].iter() {
        if size < 1024 {
            format!("{:.1}{}", size, unit)
        }
        size /= 1024;
    }
    format!("{:.1}TB", size)
}

/// Truncate path for display, keeping end visible.
pub fn truncate_path(path: PathBuf, max_len: i64) -> String {
    // Truncate path for display, keeping end visible.
    let mut s = path.to_string();
    if s.len() <= max_len { s } else { ("...".to_string() + s[-(max_len - 3)..]) }
}

/// Configure and return the application logger.
pub fn setup_logging(log_file: Option<PathBuf>, verbose: bool) -> Result<logging::Logger> {
    // Configure and return the application logger.
    let mut logger = logging::getLogger("VisualAI".to_string());
    logger.setLevel(if verbose { logging::DEBUG } else { logging::INFO });
    logger.handlers = vec![];
    let mut formatter = logging::Formatter("%(asctime)s - %(levelname)s - %(message)s".to_string());
    let mut ch = logging::StreamHandler();
    ch.setFormatter(formatter);
    logger.addHandler(ch);
    if log_file {
        // try:
        {
            log_file.parent().unwrap_or(std::path::Path::new("")).create_dir_all();
            let mut fh = logging::FileHandler(log_file);
            fh.setFormatter(formatter);
            logger.addHandler(fh);
        }
        // except OSError as e:
    }
    Ok(logger)
}

/// Build a multi-line info string summarizing this media file.
pub fn _mf_get_info_string(r#self: String, display_name: Option<String>) -> String {
    // Build a multi-line info string summarizing this media file.
    let mut name = if display_name { display_name } else { self.name };
    let mut br_kb = (self.bitrate / 1000).to_string().parse::<i64>().unwrap_or(0);
    let mut depth_str = if self.bit_depth > 8 { format!("{}bit", self.bit_depth) } else { "".to_string() };
    let mut audio_str = Constants.AUDIO_CHANNEL_NAMES.get(&self.audio_channels).cloned().unwrap_or(format!("{}ch", self.audio_channels));
    let mut hdr_tag = if (self.is_hdr || self.is_dolby_vision) { format!(" [{}]", self.hdr_str) } else { "".to_string() };
    format!("{} | {} | {} {}{} | {} {} ({} tracks) | Subs: {} | {} kbps | {}\nCreated: {} | Modified: {}", name, self.resolution_str, self.video_codec.to_uppercase(), depth_str, hdr_tag, audio_str, self.audio_codec.to_uppercase(), self.audio_streams_count, self.subs_count, br_kb, self.nice_size, self.created_str, self.modified_str)
}

/// Get color for value comparison.
pub fn _compare_color(value: String, best: String, higher_better: bool) -> String {
    // Get color for value comparison.
    if !higher_better {
        Constants.COLOR_NEUTRAL
    }
    if value == best {
        Constants.COLOR_BEST
    }
    if value >= (best * 0.8_f64) {
        Constants.COLOR_GOOD
    }
    if value >= (best * 0.5_f64) {
        Constants.COLOR_NEUTRAL
    }
    Constants.COLOR_WORSE
}

/// Create a color-coded info label.
pub fn _make_info_label(parent: String, text: String, color: String, font_key: String, side: String, opts: HashMap<String, Box<dyn std::any::Any>>) -> () {
    // Create a color-coded info label.
    tkinter.Label(parent, /* text= */ text, /* bg= */ Constants.COLOR_BG, /* fg= */ color, /* font= */ Constants.FONTS[&font_key]).pack(/* side= */ side, /* ** */ opts);
}

/// Populate the info panel with name, folder, video, and audio rows.
pub fn _build_video_info_rows(info: String, index: i64, mf: String, is_best: bool, best: HashMap<String, serde_json::Value>) -> () {
    // Populate the info panel with name, folder, video, and audio rows.
    let mut name_trunc = if mf.name.len() > 53 { (mf.name[..50] + "...".to_string()) } else { mf.name };
    let mut tag = if is_best { " [BEST]".to_string() } else { "".to_string() };
    _make_info_label(info, format!("#{}{} {}", (index + 1), tag, name_trunc), if is_best { Constants.COLOR_BEST } else { "#fff".to_string() }, "title".to_string(), tkinter.TOP);
    _make_info_label(info, format!("📁 {}", truncate_path(mf.path.parent().unwrap_or(std::path::Path::new("")), 60)), "#777".to_string(), "folder".to_string(), tkinter.TOP);
    let mut vf = tkinter.Frame(info, /* bg= */ Constants.COLOR_BG);
    vf.pack(/* fill= */ tkinter.X, /* pady= */ (3, 0));
    _make_info_label(vf, mf.resolution_str, _compare_color(mf.pixels, best["res".to_string()]), "mono_bold".to_string());
    _make_info_label(vf, " | ".to_string(), "#555".to_string());
    let mut codec_text = (format!("{}", mf.video_codec.to_uppercase()) + if mf.bit_depth > 8 { format!(" {}bit", mf.bit_depth) } else { "".to_string() });
    _make_info_label(vf, codec_text, _compare_color(Constants.CODEC_RANK.get(&mf.video_codec.to_lowercase()).cloned().unwrap_or(0), best["codec".to_string()]));
    if mf.is_dolby_vision {
        tkinter.Label(vf, /* text= */ " [DV]".to_string(), /* bg= */ "#7b00ff".to_string(), /* fg= */ "white".to_string(), /* font= */ ("Consolas".to_string(), 8, "bold".to_string())).pack(/* side= */ tkinter.LEFT, /* padx= */ 2);
    } else if mf.is_hdr {
        tkinter.Label(vf, /* text= */ " [HDR]".to_string(), /* bg= */ "#ff8800".to_string(), /* fg= */ "white".to_string(), /* font= */ ("Consolas".to_string(), 8, "bold".to_string())).pack(/* side= */ tkinter.LEFT, /* padx= */ 2);
    }
    _make_info_label(vf, " | ".to_string(), "#555".to_string());
    _make_info_label(vf, format!("{} kbps", (mf.bitrate / 1000).to_string().parse::<i64>().unwrap_or(0)), _compare_color(mf.bitrate, best["br".to_string()]));
    _make_info_label(vf, " | ".to_string(), "#555".to_string());
    _make_info_label(vf, mf.nice_size, Constants.COLOR_NEUTRAL);
    let mut af = tkinter.Frame(info, /* bg= */ Constants.COLOR_BG);
    af.pack(/* fill= */ tkinter.X, /* pady= */ (2, 0));
    let mut audio_str = Constants.AUDIO_CHANNEL_NAMES.get(&mf.audio_channels).cloned().unwrap_or(format!("{}ch", mf.audio_channels));
    _make_info_label(af, format!("Audio: {} {}", audio_str, mf.audio_codec.to_uppercase()), _compare_color(mf.audio_channels, best["ch".to_string()]), "info".to_string());
    _make_info_label(af, format!(" ({} trk)", mf.audio_streams_count), "#888".to_string(), "info".to_string());
    _make_info_label(af, " | ".to_string(), "#555".to_string(), "info".to_string());
    _make_info_label(af, format!("Subs: {}", mf.subs_count), _compare_color(mf.subs_count, best["subs".to_string()]), "info".to_string());
    _make_info_label(af, " | ".to_string(), "#555".to_string(), "info".to_string());
    _make_info_label(af, format!("Date: {}", mf.created_str), "#888".to_string(), "info".to_string());
}

/// Create a VLC media player and attach it to the given canvas.
pub fn _attach_vlc_player(app: String, canvas: String, mf: String, index: i64) -> Result<()> {
    // Create a VLC media player and attach it to the given canvas.
    // try:
    {
        let mut p = app.vlc_inst.media_player_new();
        p.set_media(app.vlc_inst.media_new(mf.path.to_string()));
        let mut wid = canvas.winfo_id();
        if platform.system() == "Windows".to_string() {
            p.set_hwnd(wid);
        } else if platform.system() == "Darwin".to_string() {
            p.set_nsobject(wid);
        } else {
            p.set_xwindow(wid);
        }
        p.play();
        p.audio_set_mute(index != 0);
        app.players.push(p);
    }
    // except (OSError, AttributeError) as e:
}

/// Build a single video comparison panel with info and VLC player.
pub fn _build_video_panel(app: String, container: String, index: i64, mf: String, cols: i64, best: HashMap<String, serde_json::Value>) -> () {
    // Build a single video comparison panel with info and VLC player.
    let (mut row, mut col, mut is_best) = ((index / cols), (index % cols), index == 0);
    let mut outer = tkinter.Frame(container, /* bg= */ "#1a1a1a".to_string(), /* highlightthickness= */ 5, /* highlightbackground= */ if is_best { "#2ea043".to_string() } else { "#1e1e1e".to_string() });
    outer.grid(/* row= */ row, /* column= */ col, /* sticky= */ "nsew".to_string(), /* padx= */ 4, /* pady= */ 4);
    app.frames.push(outer);
    let mut canvas = tkinter.Canvas(outer, /* bg= */ "black".to_string(), /* highlightthickness= */ 0);
    canvas.pack(/* fill= */ tkinter.BOTH, /* expand= */ true);
    app.canvases.push(canvas);
    let mut info = tkinter.Frame(outer, /* bg= */ Constants::COLOR_BG, /* pady= */ 5, /* padx= */ 8);
    info.pack(/* fill= */ tkinter.X, /* side= */ tkinter.BOTTOM);
    _build_video_info_rows(info, index, mf, is_best, best);
    let mut btn = tkinter.Button(info, /* text= */ format!("KEEP #{}", (index + 1)), /* bg= */ if is_best { "#2ea043".to_string() } else { "#444".to
    btn.pack(/* fill= */ tkinter.X, /* pady= */ (5, 0));
    let make_click = |idx| {
        // Return a click handler that focuses audio on the given index.
        |e| vec![app.focus_audio(idx), "break".to_string()][1]
    };
    outer.bind("<Button-1>".to_string(), make_click(index));
    canvas.bind("<Button-1>".to_string(), make_click(index));
    _attach_vlc_player(app, canvas, mf, index);
}