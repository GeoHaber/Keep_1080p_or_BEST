Rev = """
Keep 1080p or Best - Visual AI v6.1 (Simplify to Amplify)

CHANGELOG v6.1:
  - REFACTOR: "Simplify to Amplify" - reduced ~300 lines while keeping all features
  - REFACTOR: DependencyManager consolidated (4 methods -> 1 check_import)
  - REFACTOR: Constants.CODEC_RANK and FONTS centralized
  - REFACTOR: MediaFile.to_meta_dict() for cleaner cache operations
  - REFACTOR: VLCPlayerApp simplified - single-pass best value computation
  - REFACTOR: ZoomCompareWindow - cached images, combined handlers, faster rendering
  - REFACTOR: BackgroundFingerprinter - uses DuplicateDetector.hamming_distance
  - PERF: Zoom uses BILINEAR during drag, LANCZOS on release
  - PERF: Single-pass comparison stats in VLC player

All features from v6.0 preserved:
  - Equal window sizes, color-coded info panel
  - Background fingerprinting with progress bar
  - Zoom/magnify comparison (Press Z)
  - Folder path display
"""

import os  # noqa: E402
import sys  # noqa: E402
import io  # noqa: E402

# Fix Windows console Unicode output FIRST
if sys.platform == 'win32':
    try:
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')
    except Exception:
        pass

print("--- RUNNING VERSION: Visual AI v6.1 (Simplify to Amplify) ---")

import re  # noqa: E402
import json  # noqa: E402
import time  # noqa: E402
import atexit  # noqa: E402
import shutil  # noqa: E402
import asyncio  # noqa: E402
import sqlite3  # noqa: E402
import logging  # noqa: E402
import argparse  # noqa: E402
import platform  # noqa: E402
import threading  # noqa: E402
import subprocess  # noqa: E402
import tkinter as tk  # noqa: E402
import concurrent.futures  # noqa: E402

from tkinter import messagebox, filedialog  # noqa: E402
from pathlib import Path  # noqa: E402
from datetime import datetime  # noqa: E402
from collections import defaultdict  # noqa: E402
from dataclasses import dataclass  # noqa: E402
from typing import List, Dict, Optional, Tuple, Set, Any, Callable  # noqa: E402
from queue import Queue, Empty  # noqa: E402

# -----------------------------------------------------------------------------
# Thread-safe printing & utilities
# -----------------------------------------------------------------------------
PRINT_LOCK = threading.Lock()

def safe_print(*args, **kwargs) -> None:
    """Thread-safe print with Unicode error handling."""
    with PRINT_LOCK:
        try:
            print(*args, **kwargs)
        except UnicodeEncodeError:
            text = ' '.join(str(a) for a in args)
            print(text.encode('ascii', errors='replace').decode('ascii'), **kwargs)
        sys.stdout.flush()

def format_file_size(size_bytes: int) -> str:
    """Format file size."""
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if size_bytes < 1024:
            return f"{size_bytes:.1f}{unit}"
        size_bytes /= 1024
    return f"{size_bytes:.1f}TB"

def truncate_path(path: Path, max_len: int = 50) -> str:
    """Truncate path for display, keeping end visible."""
    s = str(path)
    return s if len(s) <= max_len else "..." + s[-(max_len - 3):]

# -----------------------------------------------------------------------------
# Dependency Management (SIMPLIFIED: 4 methods -> 1)
# -----------------------------------------------------------------------------

class DependencyManager:
    """Manages optional dependencies with single unified check method."""

    @staticmethod
    def check_import(name: str) -> bool:
        """Single method for all dependency checks."""
        try:
            __import__(name)
            return True
        except ImportError:
            return False

    @staticmethod
    def try_install(package_name: str, import_name: str = None) -> bool:
        """Attempt to pip-install a package if not already available."""
        if import_name is None:
            import_name = package_name
        if DependencyManager.check_import(import_name):
            return True
        safe_print(f"\n{package_name} not found. Installing...")
        try:
            subprocess.check_call(
                [sys.executable, "-m", "pip", "install", package_name, "--quiet"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
            )
            safe_print(f"OK {package_name} installed successfully")
            return True
        except Exception as e:
            safe_print(f"FAIL Failed to install {package_name}: {e}")
            return False

# Dependency initialization
HAS_NUMPY = DependencyManager.check_import("numpy") or DependencyManager.try_install("numpy")
HAS_VLC = DependencyManager.check_import("vlc") or DependencyManager.try_install("python-vlc", "vlc")
HAS_TQDM = DependencyManager.check_import("tqdm") or DependencyManager.try_install("tqdm")
HAS_PILLOW = DependencyManager.check_import("PIL") or DependencyManager.try_install("Pillow", "PIL")

if HAS_NUMPY:
    import numpy as np
if HAS_VLC:
    import vlc
if HAS_TQDM:
    from tqdm.asyncio import tqdm_asyncio
else:
    class tqdm_asyncio:
        """Fallback when tqdm is not installed."""

        @staticmethod
        async def gather(*args, **kwargs):
            """Delegate to asyncio.gather without progress display."""
            return await asyncio.gather(*args)
if HAS_PILLOW:
    from PIL import Image, ImageTk

# -----------------------------------------------------------------------------
# Configuration & Constants (CONSOLIDATED)
# -----------------------------------------------------------------------------

class Constants:
    STRICT_TIMEOUT = 15.0
    ENABLE_VISUAL_MATCHING = True
    VISUAL_MATCH_THRESHOLD = 14
    FINGERPRINT_TIMESTAMPS = (0.20, 0.50, 0.80)
    FINGERPRINT_SIZE = 32
    FINGERPRINT_BUFFER_SIZE = 3072
    FINGERPRINT_TIMEOUT = 20.0
    MIN_DURATION_FOR_FINGERPRINT = 5.0
    PHASH_DCT_SIZE = 8
    DURATION_TOLERANCE_PERCENT = 0.05
    DEFAULT_MAX_WORKERS = 4
    EFFICIENT_CODECS = {'hevc', 'av1', 'vp9'}
    CODEC_EFFICIENCY_MULTIPLIER = 1.5
    AUDIO_CHANNEL_NAMES = {6: "5.1", 8: "7.1"}

    TV_PATTERNS = [
        re.compile(r'^(?P<show_title>.*?)(?:[\s\._]*)(?:[Ss](?:eason)?[\s\._-]*?(?P<season>\d{1,2}))(?:[\s\._-]*(?:[EeXx](?:pisode)?[\s\._-]*?(?P<episode>\d{1,3})))(?P<remaining_title>.*)', re.IGNORECASE),
        re.compile(r'^(?P<show_title>.*?)(?:[\s\._]*)(?:(?P<season>\d{1,2})[xX](?P<episode>\d{1,3}))(?P<remaining_title>.*)', re.IGNORECASE),
    ]

    # CONSOLIDATED: Codec ranking (was duplicated in 2 places)
    CODEC_RANK = {'av1': 3, 'hevc': 2, 'h265': 2, 'vp9': 2, 'h264': 1, 'avc': 1}

    # CONSOLIDATED: Font definitions
    FONTS = {
        'title': ("Segoe UI", 10, "bold"),
        'info': ("Segoe UI", 8),
        'folder': ("Segoe UI", 9),  # Slightly larger for folder paths
        'mono': ("Consolas", 9),
        'mono_bold': ("Consolas", 9, "bold"),
    }

    # UI Colors
    COLOR_BEST = "#00ff00"
    COLOR_GOOD = "#ffaa00"
    COLOR_NEUTRAL = "#cccccc"
    COLOR_WORSE = "#ff6666"
    COLOR_BG = "#252526"
    COLOR_BG_DARK = "#1e1e1e"

@dataclass
class Config:
    source_dirs: List[Path]
    except_dir: Path
    user_recycle_dir: Path
    exclude_keywords: List[str]
    video_extensions: Set[str]
    max_workers: int
    dry_run: bool
    log_file: Optional[Path]
    verbose: bool
    cache_db_path: Path

# -----------------------------------------------------------------------------
# Logging
# -----------------------------------------------------------------------------

def setup_logging(log_file: Optional[Path] = None, verbose: bool = False) -> logging.Logger:
    """Configure and return the application logger."""
    logger = logging.getLogger("VisualAI")
    logger.setLevel(logging.DEBUG if verbose else logging.INFO)
    logger.handlers = []
    formatter = logging.Formatter('%(asctime)s - %(levelname)s - %(message)s')
    ch = logging.StreamHandler()
    ch.setFormatter(formatter)
    logger.addHandler(ch)
    if log_file:
        try:
            log_file.parent.mkdir(parents=True, exist_ok=True)
            fh = logging.FileHandler(log_file)
            fh.setFormatter(formatter)
            logger.addHandler(fh)
        except Exception as e:
            safe_print(f"Warning: Could not create log file: {e}")
    return logger

# -----------------------------------------------------------------------------
# Caching System
# -----------------------------------------------------------------------------

class MetadataCache:
    """SQLite-based cache for file metadata and fingerprints."""

    SCHEMA_VERSION = 2
    APP_VERSION = "v6.1"

    def __init__(self, db_path: Path, logger: logging.Logger):
        """Initialize the cache with a SQLite database at *db_path*."""
        self.db_path = db_path
        self.logger = logger
        self.conn = None
        self._lock = threading.Lock()
        self._init_db()

    def _init_db(self):
        """Open or create the SQLite database and verify schema version."""
        try:
            self.conn = sqlite3.connect(self.db_path, check_same_thread=False)
            self.conn.row_factory = sqlite3.Row
            current_version = self._get_meta("schema_version")
            if current_version is None:
                self._create_schema()
            elif int(current_version) != self.SCHEMA_VERSION:
                self.logger.warning("Cache schema mismatch. Rebuilding...")
                self._reset_db()
            self.conn.execute("PRAGMA journal_mode=WAL;")
            self.conn.commit()
        except Exception as e:
            self.logger.error(f"Failed to init cache DB: {e}")
            try:
                self._reset_db()
            except Exception as e2:
                self.logger.critical(f"Critical cache failure: {e2}")

    def _create_schema(self):
        """Create the initial database tables and metadata entries."""
        if not self.conn:
            return
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS file_cache (
                path TEXT PRIMARY KEY, size INTEGER, mtime REAL,
                metadata_json TEXT, fingerprint_json TEXT, last_seen REAL
            )
        """)
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS db_info (key TEXT PRIMARY KEY, value TEXT)
        """)
        self._set_meta("schema_version", str(self.SCHEMA_VERSION))
        self._set_meta("app_version", self.APP_VERSION)
        self.conn.commit()

    def _reset_db(self):
        """Drop and recreate the database from scratch."""
        if self.conn:
            self.conn.close()
        if self.db_path.exists():
            try:
                self.db_path.unlink()
            except Exception:
                pass
        self.conn = sqlite3.connect(self.db_path, check_same_thread=False)
        self.conn.row_factory = sqlite3.Row
        self._create_schema()

    def _get_meta(self, key: str) -> Optional[str]:
        """Retrieve a value from the db_info metadata table."""
        try:
            cur = self.conn.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='db_info'")
            if not cur.fetchone():
                return None
            cur = self.conn.execute("SELECT value FROM db_info WHERE key = ?", (key,))
            row = cur.fetchone()
            return row['value'] if row else None
        except Exception:
            return None

    def _set_meta(self, key: str, value: str):
        """Insert or update a key-value pair in the db_info table."""
        self.conn.execute("INSERT OR REPLACE INTO db_info (key, value) VALUES (?, ?)", (key, value))

    def get(self, path: Path) -> Optional[Dict]:
        """Return cached metadata and fingerprint for *path*, or None."""
        if not self.conn:
            return None
        with self._lock:
            try:
                stat = path.stat()
                cur = self.conn.execute(
                    "SELECT metadata_json, fingerprint_json FROM file_cache WHERE path = ? AND size = ? AND mtime = ?",
                    (str(path), stat.st_size, stat.st_mtime)
                )
                row = cur.fetchone()
                if row:
                    meta = json.loads(row['metadata_json'])
                    fp = json.loads(row['fingerprint_json']) if row['fingerprint_json'] else None
                    self.conn.execute("UPDATE file_cache SET last_seen = ? WHERE path = ?", (time.time(), str(path)))
                    self.conn.commit()
                    return {'meta': meta, 'fingerprint': fp}
            except Exception:
                pass
        return None

    def begin_transaction(self):
        """Start an explicit database transaction."""
        if self.conn:
            with self._lock:
                self.conn.execute("BEGIN TRANSACTION")

    def commit_transaction(self):
        """Commit the current database transaction."""
        if self.conn:
            with self._lock:
                self.conn.commit()

    def set(self, path: Path, meta: Dict, fingerprint: Optional[List[int]], commit: bool = True):
        """Store metadata and optional fingerprint for *path* in the cache."""
        if not self.conn:
            return
        with self._lock:
            try:
                stat = path.stat()
                self.conn.execute(
                    """INSERT OR REPLACE INTO file_cache (path, size, mtime, metadata_json, fingerprint_json, last_seen)
                       VALUES (?, ?, ?, ?, ?, ?)""",
                    (str(path), stat.st_size, stat.st_mtime, json.dumps(meta),
                     json.dumps(fingerprint) if fingerprint else None, time.time())
                )
                if commit:
                    self.conn.commit()
            except Exception as e:
                self.logger.error(f"Cache write error: {e}")
                try:
                    self.conn.rollback()
                except Exception:
                    pass

    def close(self):
        """Close the database connection."""
        if self.conn:
            self.conn.close()

# -----------------------------------------------------------------------------
# Process Management
# -----------------------------------------------------------------------------

class ProcessManager:
    """Manages subprocess lifecycle and cleanup."""

    def __init__(self, logger: logging.Logger):
        """Initialize the process manager with cleanup on program exit."""
        self._procs: Dict[int, subprocess.Popen] = {}
        self._lock = threading.Lock()
        self.logger = logger
        atexit.register(self.terminate_all)

    def register(self, proc: subprocess.Popen) -> None:
        """Track a subprocess for lifecycle management."""
        if proc.pid:
            with self._lock:
                self._procs[proc.pid] = proc

    def unregister(self, proc: subprocess.Popen) -> None:
        """Remove a subprocess from tracking."""
        if proc.pid:
            with self._lock:
                self._procs.pop(proc.pid, None)

    def terminate_all(self) -> None:
        """Kill all tracked subprocesses that are still running."""
        with self._lock:
            procs = list(self._procs.values())
        for proc in procs:
            if proc.returncode is None:
                try:
                    proc.kill()
                except Exception:
                    pass

    async def run_command(self, cmd: List[str], timeout: float, binary_output: bool = False) -> Tuple[int, Any, Any]:
        """Run a command asynchronously with timeout, returning (returncode, stdout, stderr)."""
        proc = None
        try:
            proc = await asyncio.create_subprocess_exec(
                *cmd, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE,
                creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if platform.system() == "Windows" else 0
            )
            self.register(proc)
            stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=timeout)
            if binary_output:
                return proc.returncode, stdout, stderr
            return proc.returncode, stdout.decode(errors='ignore'), stderr.decode(errors='ignore')
        except asyncio.TimeoutError:
            if proc:
                try:
                    proc.kill()
                except Exception:
                    pass
            return -1, b"" if binary_output else "", b"Timeout" if binary_output else "Timeout"
        except Exception as e:
            return -1, b"" if binary_output else "", str(e).encode() if binary_output else str(e)
        finally:
            if proc:
                self.unregister(proc)

# -----------------------------------------------------------------------------
# Data Structures (MediaFile with to_meta_dict)
# -----------------------------------------------------------------------------

@dataclass
class MediaFile:
    path: Path
    name: str
    size: int
    content_type: str
    title: str
    width: int
    height: int
    duration: float
    bitrate: float
    video_codec: str
    bit_depth: int
    audio_channels: int
    audio_codec: str
    audio_streams_count: int
    bpp: float
    subs_count: int
    visual_hashes: Optional[List[int]]
    sort_score: tuple
    created: float = 0.0
    modified: float = 0.0
    is_hdr: bool = False
    is_dolby_vision: bool = False

    @property
    def created_str(self) -> str:
        """Human-readable creation date."""
        return datetime.fromtimestamp(self.created).strftime('%Y-%m-%d')

    @property
    def modified_str(self) -> str:
        """Human-readable modification date."""
        return datetime.fromtimestamp(self.modified).strftime('%Y-%m-%d')

    @property
    def resolution_str(self) -> str:
        """Resolution as 'WIDTHxHEIGHT' or 'Unknown'."""
        return f"{self.width}x{self.height}" if self.width > 0 else "Unknown"

    @property
    def pixels(self) -> int:
        """Total pixel count (width * height)."""
        return self.width * self.height

    @property
    def nice_size(self) -> str:
        """File size formatted for display."""
        return format_file_size(self.size)

    @property
    def hdr_str(self) -> str:
        """HDR format label: 'Dolby Vision', 'HDR10', or 'SDR'."""
        if self.is_dolby_vision:
            return "Dolby Vision"
        elif self.is_hdr:
            return "HDR10"
        return "SDR"

    # NEW: Serialize to dict for cache (was duplicated in 3 places)
    def to_meta_dict(self) -> Dict:
        """Serialize media attributes to a dict for cache storage."""
        return {
            'width': self.width, 'height': self.height, 'duration': self.duration,
            'bitrate': self.bitrate, 'video_codec': self.video_codec, 'bit_depth': self.bit_depth,
            'audio_channels': self.audio_channels, 'audio_codec': self.audio_codec,
            'audio_streams_count': self.audio_streams_count, 'bpp': self.bpp,
            'subs_count': self.subs_count, 'is_hdr': self.is_hdr, 'is_dolby_vision': self.is_dolby_vision
        }

# -----------------------------------------------------------------------------
# Core Logic
# -----------------------------------------------------------------------------

class FingerprintGenerator:
    """Handles visual fingerprinting and pHash computation."""

    def __init__(self, process_mgr: ProcessManager, ffmpeg_bin: str, logger: logging.Logger, max_workers: int):
        """Initialize the fingerprint generator with FFmpeg and thread pool."""
        self.pm = process_mgr
        self.ffmpeg_bin = ffmpeg_bin
        self.logger = logger
        self.executor = concurrent.futures.ThreadPoolExecutor(max_workers=min(os.cpu_count() or 4, max_workers))

    def _dct_2d(self, matrix_8x8: Any) -> Any:
        """Compute a 2-D DCT on an 8x8 block for perceptual hashing."""
        if HAS_NUMPY:
            N = Constants.PHASH_DCT_SIZE
            dct = np.zeros((N, N), dtype=np.float64)
            x = np.arange(N)
            u = np.arange(N)
            cos_table = np.cos(np.outer(2 * x + 1, u) * np.pi / (2 * N))
            c = np.ones(N)
            c[0] = 1.0 / np.sqrt(2)
            for u_idx in range(N):
                for v_idx in range(N):
                    sum_val = np.sum(matrix_8x8 * np.outer(cos_table[:, u_idx], cos_table[:, v_idx]))
                    dct[u_idx, v_idx] = 0.25 * c[u_idx] * c[v_idx] * sum_val
            return dct
        return np.zeros((8, 8))

    def _compute_single_hash(self, image_bytes: bytes) -> int:
        """Compute a single perceptual hash from raw grayscale image bytes."""
        size = Constants.FINGERPRINT_SIZE
        if not image_bytes or len(image_bytes) != size * size:
            return 0
        if HAS_NUMPY:
            img = np.frombuffer(image_bytes, dtype=np.uint8).reshape(size, size)
            matrix_8x8 = np.zeros((8, 8), dtype=np.float64)
            block_size = size // 8
            for y in range(8):
                for x in range(8):
                    block = img[y * block_size:(y + 1) * block_size, x * block_size:(x + 1) * block_size]
                    matrix_8x8[y, x] = np.mean(block)
            dct_vals = self._dct_2d(matrix_8x8)
            vals = [dct_vals[y, x] for y in range(8) for x in range(8) if not (x == 0 and y == 0)]
            median = np.median(vals)
            hash_val = 0
            for y in range(8):
                for x in range(8):
                    hash_val = (hash_val << 1) | (1 if dct_vals[y, x] > median else 0)
            return hash_val
        return 0

    def _compute_hashes_from_buffer(self, buffer: bytes) -> List[int]:
        """Split a raw buffer into three frames and hash each one."""
        chunk_size = Constants.FINGERPRINT_SIZE * Constants.FINGERPRINT_SIZE
        if len(buffer) != 3 * chunk_size:
            return []
        return [self._compute_single_hash(buffer[i * chunk_size:(i + 1) * chunk_size]) for i in range(3)]

    async def generate(self, path: Path, duration: float) -> Optional[List[int]]:
        """Generate visual fingerprint hashes for a video file."""
        if duration < Constants.MIN_DURATION_FOR_FINGERPRINT:
            return None
        ts = [duration * p for p in Constants.FINGERPRINT_TIMESTAMPS]
        cmd = [
            self.ffmpeg_bin, "-y", "-hide_banner", "-loglevel", "error",
            "-ss", f"{ts[0]:.2f}", "-i", str(path),
            "-ss", f"{ts[1]:.2f}", "-i", str(path),
            "-ss", f"{ts[2]:.2f}", "-i", str(path),
            "-filter_complex",
            f"[0:v]scale={Constants.FINGERPRINT_SIZE}:{Constants.FINGERPRINT_SIZE},format=gray[v0];"
            f"[1:v]scale={Constants.FINGERPRINT_SIZE}:{Constants.FINGERPRINT_SIZE},format=gray[v1];"
            f"[2:v]scale={Constants.FINGERPRINT_SIZE}:{Constants.FINGERPRINT_SIZE},format=gray[v2];"
            f"[v0][v1][v2]hstack=inputs=3",
            "-vframes", "1", "-f", "rawvideo", "-"
        ]
        rc, out, _ = await self.pm.run_command(cmd, Constants.FINGERPRINT_TIMEOUT, binary_output=True)
        if rc == 0 and len(out) == Constants.FINGERPRINT_BUFFER_SIZE:
            loop = asyncio.get_running_loop()
            return await loop.run_in_executor(self.executor, self._compute_hashes_from_buffer, out)
        return None

    def shutdown(self):
        """Shut down the thread pool executor."""
        self.executor.shutdown(wait=False)


class MediaScanner:
    """Handles file scanning and metadata extraction."""

    def __init__(self, config: Config, process_mgr: ProcessManager, logger: logging.Logger, cache: MetadataCache):
        """Initialize the scanner with configuration and backend services."""
        self.config = config
        self.pm = process_mgr
        self.logger = logger
        self.cache = cache
        self.ffmpeg_bin = shutil.which("ffmpeg") or "ffmpeg"
        self.ffprobe_bin = shutil.which("ffprobe") or "ffprobe"
        self.non_movie_regexes = [re.compile(rf'\b{kw}\b', re.IGNORECASE) for kw in config.exclude_keywords]

    async def check_binaries(self) -> None:
        """Verify that FFprobe is available and working."""
        rc, _, err = await self.pm.run_command([self.ffprobe_bin, "-version"], 5)
        if rc != 0:
            raise RuntimeError(f"FFprobe check failed: {err}")

    def _parse_filename(self, path: Path) -> Tuple[str, str, str]:
        """Extract (title, content_type, episode_info) from a filename."""
        name = path.stem
        clean = re.sub(r'[\._]', ' ', name)
        ctype, title, extra = 'movie', clean, "0"
        for pat in Constants.TV_PATTERNS:
            match = pat.search(clean)
            if match:
                ctype = 'tv'
                title = match.group('show_title') or clean
                s, e = match.group('season'), match.group('episode')
                if s and e:
                    extra = f"S{int(s):02d}E{int(e):02d}"
                break
        return (title.strip().title(), ctype, extra)

    @staticmethod
    def _detect_hdr(vid: Dict) -> Tuple[bool, bool]:
        """Detect HDR and Dolby Vision from video stream metadata."""
        is_hdr, is_dv = False, False
        color_transfer = vid.get("color_transfer", "")
        color_primaries = vid.get("color_primaries", "")
        if color_transfer in ("smpte2084", "arib-std-b67") or color_primaries == "bt2020":
            is_hdr = True
        for side_data in vid.get("side_data_list", []):
            if "dovi" in str(side_data).lower() or "dolby" in str(side_data).lower():
                is_dv = is_hdr = True
                break
        return is_hdr, is_dv

    async def _get_metadata(self, path: Path) -> Optional[Dict]:
        """Probe a media file with FFprobe and return parsed metadata."""
        cmd = [self.ffprobe_bin, "-v", "error", "-print_format", "json", "-show_format", "-show_streams", str(path)]
        rc, out, _ = await self.pm.run_command(cmd, Constants.STRICT_TIMEOUT)
        if rc != 0 or not out:
            return None

        try:
            data = json.loads(out)
            vid = next((s for s in data.get("streams", []) if s.get("codec_type") == "video"), None)
            if not vid:
                return None
            fmt = data.get("format", {})

            width, height = int(vid.get("width", 0)), int(vid.get("height", 0))
            duration = float(fmt.get("duration", 0))
            bitrate = float(vid.get("bit_rate") or fmt.get("bit_rate") or 0)
            if duration > 0 and bitrate == 0:
                bitrate = (path.stat().st_size * 8) / duration

            try:
                n, d = map(float, vid.get("avg_frame_rate", "0/0").split('/'))
                fps = n / d if d > 0 else 0
            except (ValueError, ZeroDivisionError):
                fps = 0
            bpp = bitrate / (width * height * fps) if (width * height * fps) > 0 else 0

            auds = [s for s in data.get("streams", []) if s.get("codec_type") == "audio"]
            best_aud = max(auds, key=lambda x: int(x.get("channels", 0)), default={})
            is_hdr, is_dv = self._detect_hdr(vid)

            return {
                "width": width, "height": height, "duration": duration,
                "bitrate": bitrate, "video_codec": vid.get("codec_name", "unknown"),
                "bit_depth": int(vid.get("bits_per_raw_sample", 8)) if vid.get("bits_per_raw_sample", "").isdigit() else 8,
                "audio_channels": int(best_aud.get("channels", 0)),
                "audio_codec": best_aud.get("codec_name", "unknown"),
                "audio_streams_count": len(auds), "bpp": bpp,
                "subs_count": len([s for s in data.get("streams", []) if s.get("codec_type") == "subtitle"]),
                "is_hdr": is_hdr, "is_dolby_vision": is_dv
            }
        except Exception:
            return None

    def _build_media_file(self, path: Path, meta: Dict, hashes: Optional[List[int]]) -> MediaFile:
        """Construct a MediaFile from path, metadata dict, and optional hashes."""
        title, ctype, extra = self._parse_filename(path)
        codec_mult = Constants.CODEC_EFFICIENCY_MULTIPLIER if meta['video_codec'] in Constants.EFFICIENT_CODECS else 1.0
        quality_score = (meta['bitrate'] * codec_mult) / max(1.0, meta['duration'])
        hdr_bonus = 2000 if meta.get('is_dolby_vision') else (1000 if meta.get('is_hdr') else 0)

        score = (
            meta['width'] * meta['height'],
            hdr_bonus + (1000 if meta['bit_depth'] >= 10 else 0),
            meta['audio_channels'] * 100,
            quality_score
        )

        try:
            stat = path.stat()
            created, modified = stat.st_ctime, stat.st_mtime
        except OSError:
            created = modified = 0

        return MediaFile(
            path=path, name=path.name, size=path.stat().st_size,
            content_type=ctype, title=title,
            width=meta['width'], height=meta['height'], duration=meta['duration'],
            bitrate=meta['bitrate'], video_codec=meta['video_codec'],
            bit_depth=meta['bit_depth'], audio_channels=meta['audio_channels'],
            audio_codec=meta['audio_codec'], audio_streams_count=meta.get('audio_streams_count', 1),
            bpp=meta['bpp'], subs_count=meta['subs_count'],
            visual_hashes=hashes, sort_score=score,
            created=created, modified=modified,
            is_hdr=meta.get('is_hdr', False), is_dolby_vision=meta.get('is_dolby_vision', False)
        )

    async def scan(self) -> Dict[Tuple, List[MediaFile]]:
        """Scan configured directories and return media files grouped by title."""
        files_to_scan = []
        safe_print("--- Phase 1: Finding Files ---")

        for src in self.config.source_dirs:
            if not src.exists():
                continue
            safe_print(f" Scan: {src}")
            for ext in self.config.video_extensions:
                for path in src.rglob(f"*.{ext}"):
                    if not any(r.search(path.name) for r in self.non_movie_regexes):
                        files_to_scan.append(path)
                        if len(files_to_scan) % 50 == 0:
                            safe_print(f"\r  Found {len(files_to_scan)} files...", end="")

        print()
        safe_print(f"--- Phase 2: Analyzing {len(files_to_scan)} Files ---")

        self.cache.begin_transaction()
        groups = defaultdict(list)
        sem = asyncio.Semaphore(self.config.max_workers)

        async def _process(path: Path):
            """Fetch or probe metadata for a single file and group it."""
            async with sem:
                if not path.exists():
                    return
                cached = self.cache.get(path)
                meta, hashes = (cached['meta'], cached['fingerprint']) if cached else (None, None)
                if not meta:
                    meta = await self._get_metadata(path)
                    if meta:
                        self.cache.set(path, meta, None, commit=False)
                if not meta:
                    return

                mf = self._build_media_file(path, meta, hashes)
                groups[(mf.content_type, mf.title, self._parse_filename(path)[2])].append(mf)

        tasks = [_process(p) for p in files_to_scan]
        if HAS_TQDM:
            await tqdm_asyncio.gather(*tasks, desc="Analyzing", unit="file")
        else:
            await asyncio.gather(*tasks)

        self.cache.commit_transaction()
        return groups

# -----------------------------------------------------------------------------
# Background Fingerprinting (SIMPLIFIED)
# -----------------------------------------------------------------------------

class BackgroundFingerprinter:
    """Runs fingerprinting in background while UI is active."""

    def __init__(self, fingerprinter: FingerprintGenerator, cache: MetadataCache,
                 candidates: List[MediaFile], pairs: List[Tuple], on_match: Callable):
        """Set up background fingerprinting for the given candidates."""
        self.fingerprinter = fingerprinter
        self.cache = cache
        self.candidates = candidates
        self.pairs = pairs
        self.on_match = on_match
        self.progress = (0, len(candidates))
        self.running = True
        self._lock = threading.Lock()

    def get_progress(self) -> Tuple[int, int]:
        """Return (completed, total) fingerprinting progress."""
        with self._lock:
            return self.progress

    def stop(self):
        """Signal the fingerprinter to stop processing."""
        self.running = False

    async def run(self):
        """Run fingerprinting and check for matches."""
        sem = asyncio.Semaphore(4)

        async def _gen_fp(f: MediaFile):
            """Generate and store a fingerprint for a single media file."""
            if not self.running or f.visual_hashes:
                return
            async with sem:
                hashes = await self.fingerprinter.generate(f.path, f.duration)
                if hashes:
                    f.visual_hashes = hashes
                    self.cache.set(f.path, f.to_meta_dict(), hashes)  # Uses new method
                with self._lock:
                    self.progress = (self.progress[0] + 1, self.progress[1])

        batch_size = 10
        for i in range(0, len(self.candidates), batch_size):
            if not self.running:
                break
            await asyncio.gather(*[_gen_fp(f) for f in self.candidates[i:i + batch_size]])
            # Check for matches using unified hamming_distance
            for f1, f2 in self.pairs:
                if f1.visual_hashes and f2.visual_hashes:
                    matches = sum(1 for k in range(3)
                                  if DuplicateDetector.hamming_distance(f1.visual_hashes[k], f2.visual_hashes[k]) <= Constants.VISUAL_MATCH_THRESHOLD)
                    if matches >= 2:
                        self.on_match(f1, f2)

# -----------------------------------------------------------------------------
# Duplicate Detection & UI
# -----------------------------------------------------------------------------

class DuplicateDetector:
    """Handles duplicate detection and user interaction."""

    def __init__(self, config: Config, logger: logging.Logger, cache: MetadataCache, fingerprinter: FingerprintGenerator):
        """Initialize the duplicate detector with configuration and services."""
        self.config = config
        self.logger = logger
        self.cache = cache
        self.fingerprinter = fingerprinter
        self.history = []
        self.auto_play = False
        self.bg_fingerprinter: Optional[BackgroundFingerprinter] = None
        self.visual_match_queue: Queue = Queue()

    @staticmethod
    def hamming_distance(h1: int, h2: int) -> int:
        """Return the Hamming distance between two integer hashes."""
        return bin(h1 ^ h2).count('1')

    def _on_visual_match(self, f1: MediaFile, f2: MediaFile):
        """Queue a visual match pair for later processing."""
        self.visual_match_queue.put((f1, f2))

    def _collect_fingerprint_candidates(self, singles: List[MediaFile]) -> Tuple[List[MediaFile], List[Tuple]]:
        """Collect files needing fingerprinting and their candidate pairs."""
        candidates, pairs = [], []
        for i, f1 in enumerate(singles):
            max_diff = f1.duration * Constants.DURATION_TOLERANCE_PERCENT
            for j in range(i + 1, len(singles)):
                f2 = singles[j]
                if f2.duration - f1.duration > max_diff:
                    break
                pairs.append((f1, f2))
                if f1 not in candidates and not f1.visual_hashes:
                    candidates.append(f1)
                if f2 not in candidates and not f2.visual_hashes:
                    candidates.append(f2)
        return candidates, pairs

    async def _drain_visual_matches(self):
        """Process any visual matches found by background fingerprinter."""
        while not self.visual_match_queue.empty():
            try:
                f1, f2 = self.visual_match_queue.get_nowait()
                new_key = (f1.content_type, f"VISUAL: {f1.title} / {f2.title}", "Match")
                await self._handle_group(0, 1, new_key, [f1, f2])
            except Empty:
                break

    async def process(self, groups: Dict[Tuple, List[MediaFile]]) -> None:
        """Process duplicate groups and handle user interaction."""
        ready_groups = {k: v for k, v in groups.items() if len(v) > 1}
        singles = [v[0] for k, v in groups.items() if len(v) == 1]
        singles.sort(key=lambda x: x.duration)

        candidates_to_fingerprint, candidate_pairs = [], []
        if Constants.ENABLE_VISUAL_MATCHING:
            candidates_to_fingerprint, candidate_pairs = self._collect_fingerprint_candidates(singles)

        if candidates_to_fingerprint:
            safe_print(f"--- Starting background fingerprinting for {len(candidates_to_fingerprint)} files ---")
            self.bg_fingerprinter = BackgroundFingerprinter(
                self.fingerprinter, self.cache, candidates_to_fingerprint, candidate_pairs, self._on_visual_match
            )
            asyncio.create_task(self.bg_fingerprinter.run())

        total_waste = sum(sum(f.size for f in files[1:]) for files in ready_groups.values() if files)
        for files in ready_groups.values():
            files.sort(key=lambda f: f.sort_score, reverse=True)

        safe_print(f"\n{'=' * 60}")
        safe_print(f" Found {len(ready_groups)} duplicate groups (ready now)")
        if candidates_to_fingerprint:
            safe_print(f" + {len(candidates_to_fingerprint)} files being fingerprinted in background")
        safe_print(f" Potential savings: {format_file_size(total_waste)}")
        safe_print(f"{'=' * 60}\n")

        sorted_groups = sorted(ready_groups.items(), key=lambda x: sum(f.size for f in x[1]), reverse=True)

        for i, (key, files) in enumerate(sorted_groups):
            await self._handle_group(i, len(sorted_groups), key, files)
            await self._drain_visual_matches()

        if self.bg_fingerprinter:
            self.bg_fingerprinter.stop()

    def _display_group_info(self, idx: int, total: int, title: str, extra: str,
                            files: List[MediaFile], savings: int) -> None:
        """Display group information and file listing to the console."""
        safe_print(f"\n{'=' * 80}\n[{idx + 1}/{total}] {title} ({extra}) | Savings: {format_file_size(savings)}\n{'=' * 80}")
        by_dir = defaultdict(list)
        for j, f in enumerate(files):
            by_dir[f.path.parent].append((j, f))
        for parent, items in by_dir.items():
            safe_print(f" Folder: {parent}")
            for j, f in items:
                marker = " [BEST]" if j == 0 else ""
                safe_print(f"   {j + 1}.{marker} {f.get_info_string(f.name)}")
            safe_print("-" * 40)
        safe_print("(k #) Keep, (d #) Recycle, (p) Play, (s) Skip, (u) Undo, (q) Quit")

    def _execute_command(self, cmd: str, parts: List[str], files: List[MediaFile]) -> Optional[str]:
        """Execute a user command. Returns 'break', 'continue', or None."""
        if cmd == 'q':
            if self.bg_fingerprinter:
                self.bg_fingerprinter.stop()
            sys.exit(0)
        if cmd == 's':
            return 'break'
        if cmd == 'u':
            self.undo_last_group()
            return 'continue'
        if cmd == 'p':
            self.auto_play = True
            return 'continue'
        if cmd in ('k', 'd') and len(parts) > 1 and parts[1].isdigit():
            target_idx = int(parts[1]) - 1
            if 0 <= target_idx < len(files):
                if cmd == 'k':
                    self._keep_file(files[target_idx], files)
                    return 'break'
                self._recycle_file(files[target_idx])
                files.pop(target_idx)
                if len(files) < 2:
                    return 'break'
        return None

    def _try_vlc_autoplay(self, files: List[MediaFile]) -> Optional[str]:
        """Attempt VLC auto-play. Returns 'keep', 'skip', or None."""
        if not (self.auto_play and HAS_VLC):
            return None
        bg_progress = self.bg_fingerprinter.get_progress() if self.bg_fingerprinter else None
        result = VLCPlayer.launch(files, self.fingerprinter.ffmpeg_bin, bg_progress)
        if not result:
            self.auto_play = False
            return None
        if result.get('action') == 'keep':
            target_idx = result['index']
            if 0 <= target_idx < len(files):
                self._keep_file(files[target_idx], files)
                return 'keep'
        if result.get('action') == 'skip':
            return 'skip'
        self.auto_play = False
        return None

    async def _handle_group(self, idx: int, total: int, key: Tuple, files: List[MediaFile]) -> None:
        """Handle a single duplicate group with user interaction."""
        title, extra = key[1], key[2]
        savings = sum(f.size for f in files) - max(f.size for f in files)

        while True:
            vlc_result = self._try_vlc_autoplay(files)
            if vlc_result in ('keep', 'skip'):
                return

            self._display_group_info(idx, total, title, extra, files, savings)

            try:
                choice = await asyncio.to_thread(input, "Choice: ")
            except EOFError:
                return

            parts = choice.lower().split()
            if not parts:
                continue

            action = self._execute_command(parts[0], parts, files)
            if action == 'break':
                break
            if action == 'continue':
                continue

    def _recycle_file(self, file: MediaFile) -> None:
        """Move a file to the recycle directory."""
        safe_print(f">> Recycling: {file.name}")
        if self.config.dry_run:
            safe_print("[DRY RUN] File would be recycled")
            return
        try:
            dst = self.config.user_recycle_dir / file.name
            counter = 1
            while dst.exists():
                dst = self.config.user_recycle_dir / f"{dst.stem}_{counter}{dst.suffix}"
                counter += 1
            try:
                shutil.move(str(file.path), str(dst))
            except OSError:
                shutil.copy2(str(file.path), str(dst))
                file.path.unlink()
            self.logger.info(f"Recycled {file.path}")
            self.history.append({'type': 'recycle', 'original_path': file.path, 'temp_path': dst, 'timestamp': time.time()})
        except Exception as e:
            safe_print(f"Error recycling: {e}")

    def _keep_file(self, keep: MediaFile, all_files: List[MediaFile]) -> None:
        """Keep the chosen file and recycle all others in the group."""
        safe_print(f">> Keeping: {keep.name}")
        self.history.append({'type': 'group_start', 'timestamp': time.time()})
        for other in all_files:
            if other != keep:
                self._recycle_file(other)

    def undo_last_group(self):
        """Undo the most recent group keep/recycle actions."""
        if not self.history:
            safe_print("Nothing to undo.")
            return
        safe_print("Undoing last actions...")
        count = 0
        while self.history:
            item = self.history.pop()
            if item['type'] == 'group_start':
                break
            if item['type'] == 'recycle' and item['temp_path'].exists():
                try:
                    shutil.move(str(item['temp_path']), str(item['original_path']))
                    safe_print(f"Restored: {item['original_path'].name}")
                    count += 1
                except Exception as e:
                    safe_print(f"Failed to restore {item['original_path'].name}: {e}")
        safe_print(f"Undo complete. Restored {count} files.")

    def get_info_string(self, display_name: Optional[str] = None) -> str:
        """For MediaFile - moved here to keep MediaFile clean."""
        pass  # Implemented on MediaFile directly

# Add get_info_string to MediaFile (was missing)
def _mf_get_info_string(self, display_name: Optional[str] = None) -> str:
    """Build a multi-line info string summarizing this media file."""
    name = display_name if display_name else self.name
    br_kb = int(self.bitrate / 1000)
    depth_str = f"{self.bit_depth}bit" if self.bit_depth > 8 else ""
    audio_str = Constants.AUDIO_CHANNEL_NAMES.get(self.audio_channels, f"{self.audio_channels}ch")
    hdr_tag = f" [{self.hdr_str}]" if self.is_hdr or self.is_dolby_vision else ""
    return (
        f"{name} | {self.resolution_str} | {self.video_codec.upper()} {depth_str}{hdr_tag} | "
        f"{audio_str} {self.audio_codec.upper()} ({self.audio_streams_count} tracks) | "
        f"Subs: {self.subs_count} | {br_kb} kbps | {self.nice_size}\n"
        f"Created: {self.created_str} | Modified: {self.modified_str}"
    )

MediaFile.get_info_string = _mf_get_info_string

# -----------------------------------------------------------------------------
# Zoom Comparison Window (SIMPLIFIED: cached images, combined handlers)
# -----------------------------------------------------------------------------

class ZoomCompareWindow:
    """Side-by-side frame comparison with zoom/pan using FFmpeg."""

    def __init__(self, files: List[MediaFile], timestamp: float, ffmpeg_bin: str):
        """Prepare a zoom-comparison window for the given files."""
        self.files = files[:2]
        self.timestamp = timestamp
        self.ffmpeg_bin = ffmpeg_bin
        self.zoom_level = 1.0
        self.pan_offset = [0, 0]
        self.images = []
        self.cached_resized = {}  # NEW: Cache resized images
        self.photo_images = []
        self.root = None
        self.canvases = []
        self.dragging = False
        self.drag_start = (0, 0)

    def extract_frames(self) -> bool:
        """Extract a single frame from each file using FFmpeg."""
        if not HAS_PILLOW:
            return False
        self.images = []
        for mf in self.files:
            cmd = [self.ffmpeg_bin, "-ss", str(self.timestamp), "-i", str(mf.path),
                   "-vframes", "1", "-f", "image2pipe", "-vcodec", "png", "-"]
            try:
                proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                    creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if platform.system() == "Windows" else 0)
                stdout, _ = proc.communicate(timeout=10)
                if stdout:
                    self.images.append(Image.open(io.BytesIO(stdout)))
                else:
                    return False
            except Exception as e:
                safe_print(f"Frame extraction error: {e}")
                return False
        return len(self.images) == len(self.files)

    def _build_canvas_panels(self, container):
        """Create side-by-side canvas panels with mouse bindings for each image."""
        for i, (img, mf) in enumerate(zip(self.images, self.files)):
            frame = tk.Frame(container, bg="#1a1a1a", highlightthickness=2,
                           highlightbackground="#2ea043" if i == 0 else "#333")
            frame.grid(row=0, column=i, sticky="nsew", padx=3, pady=3)
            container.grid_columnconfigure(i, weight=1, uniform="zoom_col")
            container.grid_rowconfigure(0, weight=1)

            tk.Label(frame, text=f"#{i+1}: {mf.name[:50]}", bg="#1a1a1a", fg="#fff",
                    font=Constants.FONTS['mono_bold']).pack(fill=tk.X)

            canvas = tk.Canvas(frame, bg="black", highlightthickness=0)
            canvas.pack(fill=tk.BOTH, expand=True)
            self.canvases.append(canvas)

            canvas.bind("<MouseWheel>", lambda e: self._render(1.1 if e.delta > 0 else 0.9))
            canvas.bind("<Button-4>", lambda e: self._render(1.1))
            canvas.bind("<Button-5>", lambda e: self._render(0.9))
            canvas.bind("<ButtonPress-1>", self._start_drag)
            canvas.bind("<B1-Motion>", self._on_drag)
            canvas.bind("<ButtonRelease-1>", self._end_drag)

    def run(self):
        """Build and display the zoom-comparison window."""
        if not self.extract_frames():
            return

        self.root = tk.Toplevel()
        self.root.title(f"Zoom Compare @ {self.timestamp:.1f}s")
        self.root.configure(bg="#1e1e1e")

        screen_w, screen_h = self.root.winfo_screenwidth(), self.root.winfo_screenheight()
        self.root.geometry(f"{int(screen_w * 0.9)}x{int(screen_h * 0.85)}")

        # Keyboard bindings
        for key, fn in [("+", lambda e: self._render(1.25)), ("-", lambda e: self._render(0.8)),
                        ("=", lambda e: self._render(1.25)), ("r", lambda e: self._reset()),
                        ("<Escape>", lambda e: self.root.destroy()),
                        ("<Left>", lambda e: self._pan(-50, 0)), ("<Right>", lambda e: self._pan(50, 0)),
                        ("<Up>", lambda e: self._pan(0, -50)), ("<Down>", lambda e: self._pan(0, 50))]:
            self.root.bind(key, fn)

        # Info bar
        info = tk.Frame(self.root, bg="#252526")
        info.pack(fill=tk.X)
        tk.Label(info, text="Zoom: +/- | Pan: Arrows/Drag | Reset: R | Close: Esc",
                 bg="#252526", fg="#888", font=("Segoe UI", 9)).pack(pady=5)

        # Canvas container
        container = tk.Frame(self.root, bg="#1e1e1e")
        container.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)
        self._build_canvas_panels(container)

        # Controls
        ctrl = tk.Frame(self.root, bg="#252526")
        ctrl.pack(fill=tk.X)
        self.zoom_label = tk.Label(ctrl, text="Zoom: 100%", bg="#252526", fg="#fff", font=Constants.FONTS['mono_bold'])
        self.zoom_label.pack(side=tk.LEFT, padx=10, pady=5)
        for text, cmd, color in [("Zoom +", lambda: self._render(1.25), "#333"),
                                  ("Zoom -", lambda: self._render(0.8), "#333"),
                                  ("Reset", self._reset, "#333"),
                                  ("Close", self.root.destroy, "#c42b1c")]:
            tk.Button(ctrl, text=text, command=cmd, bg=color, fg="#fff", relief=tk.FLAT).pack(side=tk.LEFT if text != "Close" else tk.RIGHT, padx=2)

        self._render()
        self.root.focus_set()

    def _render(self, zoom_factor: float = None, fast: bool = False):
        """COMBINED: Single method for zoom/pan updates."""
        if zoom_factor:
            self.zoom_level = max(0.1, min(10.0, self.zoom_level * zoom_factor))
            self.zoom_label.configure(text=f"Zoom: {int(self.zoom_level * 100)}%")
            self.cached_resized.clear()  # Invalidate cache on zoom

        self.photo_images = []
        resample = Image.Resampling.BILINEAR if fast else Image.Resampling.LANCZOS

        for i, (canvas, img) in enumerate(zip(self.canvases, self.images)):
            canvas.update_idletasks()
            cw, ch = canvas.winfo_width(), canvas.winfo_height()
            if cw < 10 or ch < 10:
                continue

            cache_key = (i, self.zoom_level, cw, ch)
            if cache_key in self.cached_resized and not fast:
                resized = self.cached_resized[cache_key]
            else:
                img_w, img_h = img.size
                scale = min(cw / img_w, ch / img_h) * self.zoom_level
                new_w, new_h = int(img_w * scale), int(img_h * scale)
                resized = img.resize((new_w, new_h), resample)
                if not fast:
                    self.cached_resized[cache_key] = resized

            x = (cw - resized.width) // 2 + self.pan_offset[0]
            y = (ch - resized.height) // 2 + self.pan_offset[1]

            photo = ImageTk.PhotoImage(resized)
            self.photo_images.append(photo)
            canvas.delete("all")
            canvas.create_image(x, y, anchor=tk.NW, image=photo)

    def _reset(self):
        """Reset zoom level and pan offset."""
        self.zoom_level, self.pan_offset = 1.0, [0, 0]
        self.zoom_label.configure(text="Zoom: 100%")
        self.cached_resized.clear()
        self._render()

    def _pan(self, dx: int, dy: int):
        """Pan the view by the given pixel offsets."""
        self.pan_offset[0] += dx
        self.pan_offset[1] += dy
        self._render(fast=True)

    def _start_drag(self, event):
        """Record the starting point of a mouse drag."""
        self.dragging, self.drag_start = True, (event.x, event.y)

    def _on_drag(self, event):
        """Handle mouse drag to pan the zoom view."""
        if self.dragging:
            self.pan_offset[0] += event.x - self.drag_start[0]
            self.pan_offset[1] += event.y - self.drag_start[1]
            self.drag_start = (event.x, event.y)
            self._render(fast=True)  # BILINEAR during drag

    def _end_drag(self, event):
        """Finish drag and re-render at full quality."""
        self.dragging = False
        self._render()  # LANCZOS on release

# -----------------------------------------------------------------------------
# VLC Player helpers (module-level to keep VLCPlayerApp lean)
# -----------------------------------------------------------------------------

def _compare_color(value, best, higher_better: bool = True) -> str:
    """Get color for value comparison."""
    if not higher_better:
        return Constants.COLOR_NEUTRAL
    if value == best:
        return Constants.COLOR_BEST
    if value >= best * 0.8:
        return Constants.COLOR_GOOD
    if value >= best * 0.5:
        return Constants.COLOR_NEUTRAL
    return Constants.COLOR_WORSE


def _make_info_label(parent, text, color, font_key='mono', side=tk.LEFT, **opts):
    """Create a color-coded info label."""
    tk.Label(parent, text=text, bg=Constants.COLOR_BG, fg=color,
            font=Constants.FONTS[font_key]).pack(side=side, **opts)


def _build_video_info_rows(info, index: int, mf, is_best: bool, best: Dict):
    """Populate the info panel with name, folder, video, and audio rows."""
    # Row 1: Name
    name_trunc = mf.name[:50] + "..." if len(mf.name) > 53 else mf.name
    tag = " [BEST]" if is_best else ""
    _make_info_label(info, f"#{index+1}{tag} {name_trunc}", Constants.COLOR_BEST if is_best else "#fff", 'title', tk.TOP)

    # Row 2: Folder
    _make_info_label(info, f"\U0001f4c1 {truncate_path(mf.path.parent, 60)}", "#777", 'folder', tk.TOP)

    # Row 3: Video info
    vf = tk.Frame(info, bg=Constants.COLOR_BG)
    vf.pack(fill=tk.X, pady=(3, 0))
    _make_info_label(vf, mf.resolution_str, _compare_color(mf.pixels, best['res']), 'mono_bold')
    _make_info_label(vf, " | ", "#555")
    codec_text = f"{mf.video_codec.upper()}" + (f" {mf.bit_depth}bit" if mf.bit_depth > 8 else "")
    _make_info_label(vf, codec_text, _compare_color(Constants.CODEC_RANK.get(mf.video_codec.lower(), 0), best['codec']))
    if mf.is_dolby_vision:
        tk.Label(vf, text=" [DV]", bg="#7b00ff", fg="white", font=("Consolas", 8, "bold")).pack(side=tk.LEFT, padx=2)
    elif mf.is_hdr:
        tk.Label(vf, text=" [HDR]", bg="#ff8800", fg="white", font=("Consolas", 8, "bold")).pack(side=tk.LEFT, padx=2)
    _make_info_label(vf, " | ", "#555")
    _make_info_label(vf, f"{int(mf.bitrate/1000)} kbps", _compare_color(mf.bitrate, best['br']))
    _make_info_label(vf, " | ", "#555")
    _make_info_label(vf, mf.nice_size, Constants.COLOR_NEUTRAL)

    # Row 4: Audio
    af = tk.Frame(info, bg=Constants.COLOR_BG)
    af.pack(fill=tk.X, pady=(2, 0))
    audio_str = Constants.AUDIO_CHANNEL_NAMES.get(mf.audio_channels, f"{mf.audio_channels}ch")
    _make_info_label(af, f"Audio: {audio_str} {mf.audio_codec.upper()}", _compare_color(mf.audio_channels, best['ch']), 'info')
    _make_info_label(af, f" ({mf.audio_streams_count} trk)", "#888", 'info')
    _make_info_label(af, " | ", "#555", 'info')
    _make_info_label(af, f"Subs: {mf.subs_count}", _compare_color(mf.subs_count, best['subs']), 'info')
    _make_info_label(af, " | ", "#555", 'info')
    _make_info_label(af, f"Date: {mf.created_str}", "#888", 'info')


def _attach_vlc_player(app, canvas, mf, index: int):
    """Create a VLC media player and attach it to the given canvas."""
    try:
        p = app.vlc_inst.media_player_new()
        p.set_media(app.vlc_inst.media_new(str(mf.path)))
        wid = canvas.winfo_id()
        if platform.system() == "Windows":
            p.set_hwnd(wid)
        elif platform.system() == "Darwin":
            p.set_nsobject(wid)
        else:
            p.set_xwindow(wid)
        p.play()
        p.audio_set_mute(index != 0)
        app.players.append(p)
    except Exception as e:
        safe_print(f"Player error {mf.name}: {e}")


def _build_video_panel(app, container, index: int, mf, cols: int, best: Dict):
    """Build a single video comparison panel with info and VLC player."""
    row, col, is_best = index // cols, index % cols, (index == 0)

    outer = tk.Frame(container, bg="#1a1a1a", highlightthickness=5,
                    highlightbackground="#2ea043" if is_best else "#1e1e1e")
    outer.grid(row=row, column=col, sticky="nsew", padx=4, pady=4)
    app.frames.append(outer)

    canvas = tk.Canvas(outer, bg="black", highlightthickness=0)
    canvas.pack(fill=tk.BOTH, expand=True)
    app.canvases.append(canvas)

    # Info panel
    info = tk.Frame(outer, bg=Constants.COLOR_BG, pady=5, padx=8)
    info.pack(fill=tk.X, side=tk.BOTTOM)

    _build_video_info_rows(info, index, mf, is_best, best)

    # Keep button
    btn = tk.Button(info, text=f"KEEP #{index + 1}", bg="#2ea043" if is_best else "#444",
                   fg="white", font=Constants.FONTS['title'], relief=tk.FLAT,
                   cursor="hand2", command=lambda idx=index: app._keep_file(idx))
    btn.pack(fill=tk.X, pady=(5, 0))

    # Click handler
    def make_click(idx):
        """Return a click handler that focuses audio on the given index."""
        return lambda e: [app._focus_audio(idx), "break"][1]
    outer.bind("<Button-1>", make_click(index))
    canvas.bind("<Button-1>", make_click(index))

    _attach_vlc_player(app, canvas, mf, index)


# -----------------------------------------------------------------------------
# VLC Player (SIMPLIFIED: single-pass comparison, inline info panel)
# -----------------------------------------------------------------------------

class VLCPlayerApp:
    """Enhanced side-by-side video comparison player."""

    def __init__(self, media_files: List[MediaFile], ffmpeg_bin: str = "ffmpeg",
                 bg_progress: Optional[Tuple[int, int]] = None):
        """Initialize the VLC player comparison app."""
        self.media_files = media_files
        self.ffmpeg_bin = ffmpeg_bin
        self.bg_progress = bg_progress
        self.root = None
        self.players = []
        self.frames = []
        self.canvases = []
        self.is_paused = False
        self.is_muted = False
        self.slider_dragging = False
        self.slider_var = None
        self.after_id = None
        self.active_audio_idx = 0
        self.result = None

        # SINGLE-PASS: Compute best values inline (was separate _compute_comparison)
        self.best = {'res': 0, 'br': 0, 'ch': 0, 'subs': 0, 'depth': 0, 'codec': 0}
        for mf in media_files:
            self.best['res'] = max(self.best['res'], mf.pixels)
            self.best['br'] = max(self.best['br'], mf.bitrate)
            self.best['ch'] = max(self.best['ch'], mf.audio_channels)
            self.best['subs'] = max(self.best['subs'], mf.subs_count)
            self.best['depth'] = max(self.best['depth'], mf.bit_depth)
            self.best['codec'] = max(self.best['codec'], Constants.CODEC_RANK.get(mf.video_codec.lower(), 0))

    def run(self) -> Optional[Dict]:
        """Launch the side-by-side comparison player."""
        self.root = tk.Tk()
        self.root.title("Side-by-Side Comparison - Visual AI v6.1")
        self.root.configure(bg="#1e1e1e")
        self.root.protocol("WM_DELETE_WINDOW", self._cleanup)

        sw, sh = self.root.winfo_screenwidth(), self.root.winfo_screenheight()
        self.root.geometry(f"{int(sw * 0.90)}x{int(sh * 0.85)}")

        # Bindings
        for key, fn in [("<space>", lambda e: self._toggle_pause()), ("<Left>", lambda e: self._seek_rel(-5)),
                        ("<Right>", lambda e: self._seek_rel(5)), ("m", lambda e: self._mute_all()),
                        ("q", lambda e: self._confirm_exit()), ("s", lambda e: self._skip_group()),
                        ("z", lambda e: self._open_zoom()), ("Z", lambda e: self._open_zoom())]:
            self.root.bind(key, fn)

        count = len(self.media_files)
        cols, rows = min(count, 2), (count + 1) // 2

        try:
            args = ["--no-xlib", "--quiet", "--no-video-title-show"]
            if platform.system() == "Windows":
                args.append("--no-osd")
            self.vlc_inst = vlc.Instance(*args)
        except Exception as e:
            safe_print(f"VLC init error: {e}")
            self.root.destroy()
            return None

        cont = tk.Frame(self.root, bg="#1e1e1e")
        cont.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)

        for c in range(cols):
            cont.grid_columnconfigure(c, weight=1, uniform="video_col")
        for r in range(rows):
            cont.grid_rowconfigure(r, weight=1, uniform="video_row")

        for i, mf in enumerate(self.media_files):
            _build_video_panel(self, cont, i, mf, cols, self.best)

        self._build_controls()
        for i, frame in enumerate(self.frames):
            color = "#007acc" if i == self.active_audio_idx else ("#2ea043" if i == 0 else "#1e1e1e")
            frame.configure(highlightbackground=color, highlightthickness=5)
        self._update_loop()
        self.root.mainloop()
        return self.result

    def _build_controls(self):
        """Build the playback controls bar at the bottom of the window."""
        ctrl = tk.Frame(self.root, bg="#2d2d2d", pady=8)
        ctrl.pack(fill=tk.X, side=tk.BOTTOM)

        if self.bg_progress:
            done, total = self.bg_progress
            tk.Label(ctrl, text=f"Fingerprinting: {done}/{total} files...",
                    bg="#2d2d2d", fg="#ffaa00", font=("Segoe UI", 9)).pack(pady=(0, 5))

        self.slider_var = tk.DoubleVar()
        slider = tk.Scale(ctrl, from_=0, to=100, orient=tk.HORIZONTAL, variable=self.slider_var,
                         showvalue=0, bg="#2d2d2d", fg="#007acc", troughcolor="#404040",
                         activebackground="#0099ff", command=self._on_seek, highlightthickness=0, bd=0)
        slider.pack(fill=tk.X, padx=20, pady=(0, 10))
        slider.bind("<ButtonPress-1>", lambda e: setattr(self, 'slider_dragging', True))
        slider.bind("<ButtonRelease-1>", lambda e: [setattr(self, 'slider_dragging', False), self._on_seek(slider.get())])

        bf = tk.Frame(ctrl, bg="#2d2d2d")
        bf.pack()

        for text, cmd, color in [("⏯ Pause (Space)", self._toggle_pause, "#3c3c3c"),
                                  ("🔇 Mute (M)", self._mute_all, "#3c3c3c"),
                                  ("🔍 Zoom (Z)", self._open_zoom, "#555"),
                                  ("⏭ Skip (S)", self._skip_group, "#007acc"),
                                  ("❌ Close (Q)", self._confirm_exit, "#c42b1c")]:
            tk.Button(bf, text=text, command=cmd, bg=color, fg="white",
                     relief=tk.FLAT, padx=12, pady=5, font=("Segoe UI", 9)).pack(side=tk.LEFT, padx=3)

        tk.Label(bf, text="Click video for audio | ←→ Seek", bg="#2d2d2d", fg="#666",
                font=("Segoe UI", 8)).pack(side=tk.LEFT, padx=15)

    def _open_zoom(self):
        """Open the zoom-comparison window at the current playback position."""
        if not HAS_PILLOW or not self.players:
            return
        try:
            pos = self.players[0].get_position()
            timestamp = pos * self.media_files[0].duration
        except Exception:
            timestamp = 0
        if not self.is_paused:
            self._toggle_pause()
        ZoomCompareWindow(self.media_files, timestamp, self.ffmpeg_bin).run()

    def _keep_file(self, index: int):
        """Prompt user to confirm keeping the selected video."""
        if messagebox.askyesno("Confirm Keep", f"Keep video #{index + 1}\n'{self.media_files[index].name}'\n\nand recycle the others?"):
            self.result = {'action': 'keep', 'index': index}
            if self.root:
                self.root.quit()
            self._cleanup()

    def _confirm_exit(self):
        """Prompt user to confirm closing the player."""
        if messagebox.askyesno("Confirm Exit", "Close the player?"):
            self._cleanup()

    def _skip_group(self):
        """Skip the current group without making a decision."""
        self.result = {'action': 'skip'}
        if self.root:
            self.root.quit()
        self._cleanup()

    def _focus_audio(self, target_idx: int):
        """Switch audio focus to the given player index."""
        self.active_audio_idx = target_idx
        for i, p in enumerate(self.players):
            p.audio_set_mute(i != target_idx)
        for i, frame in enumerate(self.frames):
            color = "#007acc" if i == self.active_audio_idx else ("#2ea043" if i == 0 else "#1e1e1e")
            frame.configure(highlightbackground=color, highlightthickness=5)

    def _toggle_pause(self):
        """Toggle play/pause for all players."""
        self.is_paused = not self.is_paused
        for p in self.players:
            p.set_pause(1 if self.is_paused else 0)

    def _mute_all(self):
        """Toggle mute on all players."""
        self.is_muted = not self.is_muted
        if self.is_muted:
            for p in self.players:
                p.audio_set_mute(True)
        else:
            self._focus_audio(self.active_audio_idx)

    def _on_seek(self, val):
        """Seek all players to the position indicated by *val*."""
        for p in self.players:
            if p.is_seekable():
                p.set_position(float(val) / 100.0)

    def _seek_rel(self, delta: float):
        """Seek relative to the current position by *delta* percent."""
        if not self.players:
            return
        try:
            cur = self.players[0].get_position() * 100
            new_pos = max(0, min(100, cur + delta))
            self.slider_var.set(new_pos)
            self._on_seek(new_pos)
        except Exception:
            pass

    def _update_loop(self):
        """Periodically update the seek slider to match playback position."""
        if self.root and not self.slider_dragging and self.players:
            try:
                p = self.players[self.active_audio_idx] if self.active_audio_idx < len(self.players) else self.players[0]
                pos = p.get_position()
                if pos >= 0:
                    self.slider_var.set(pos * 100)
            except Exception:
                pass
        if self.root:
            self.after_id = self.root.after(250, self._update_loop)

    def _cleanup(self):
        """Release all VLC players and destroy the window."""
        if self.after_id and self.root:
            try:
                self.root.after_cancel(self.after_id)
            except Exception:
                pass
        for p in self.players:
            try:
                p.stop()
                p.release()
            except Exception:
                pass
        if self.root:
            self.root.destroy()
            self.root = None


class VLCPlayer:
    @staticmethod
    def launch(files: List[MediaFile], ffmpeg_bin: str = "ffmpeg",
               bg_progress: Optional[Tuple[int, int]] = None) -> Optional[Dict]:
        """Create and run a VLC comparison player, returning the user's choice."""
        if not HAS_VLC:
            return None
        return VLCPlayerApp(files, ffmpeg_bin, bg_progress).run()

# -----------------------------------------------------------------------------
# Main Application
# -----------------------------------------------------------------------------

class VisualAIApp:
    """Top-level application orchestrating scan, detection, and user flow."""

    def __init__(self):
        """Initialize application state."""
        self.config: Optional[Config] = None
        self.logger: Optional[logging.Logger] = None
        self.pm: Optional[ProcessManager] = None
        self.cache: Optional[MetadataCache] = None

    def _pick_folder_dialog(self) -> Optional[str]:
        """Open a native folder-picker dialog and return the selected path."""
        try:
            root = tk.Tk()
            root.withdraw()
            root.attributes('-topmost', True)
            folder = filedialog.askdirectory(title="Select folder to scan for video duplicates", mustexist=True)
            root.destroy()
            return folder if folder else None
        except Exception as e:
            safe_print(f"Error opening folder picker: {e}")
            return None

    def _validate_source_dirs(self, source_list: List) -> List[Path]:
        """Validate source directories, prompting for a folder if none are valid."""
        valid_sources = []
        for src_path in source_list:
            src = Path(src_path)
            if not src.exists():
                safe_print(f"WARNING: Directory does not exist: {src}")
                continue
            try:
                next(src.iterdir(), None)
                valid_sources.append(src)
            except (PermissionError, OSError) as e:
                safe_print(f"WARNING: Cannot access directory: {src} ({e})")

        if valid_sources:
            return valid_sources

        safe_print("\n" + "=" * 60)
        safe_print("No valid source directories configured.")
        safe_print("=" * 60)
        response = input("\nWould you like to pick a folder to scan? (y/n): ").strip().lower()
        if response == 'y':
            folder = self._pick_folder_dialog()
            if folder:
                safe_print(f"Selected: {folder}")
                return [Path(folder)]
            safe_print("No folder selected.")
            raise SystemExit(1)
        safe_print("\nUsage: python Keep_1080p_VisualAI_v6_1.py -s <folder>")
        raise SystemExit(1)

    def load_config(self, args) -> Config:
        """Build a Config from CLI arguments and optional config file."""
        base_dir = Path(__file__).resolve().parent

        defaults = {
            "source_dirs": [], "except_dir": str(base_dir / "Exceptions"),
            "user_recycle_dir": str(base_dir / "Recycled"),
            "exclude_keywords": ["trailer", "sample"],
            "video_extensions": ["mp4", "mkv", "avi", "wmv", "mov", "m4v", "mpg", "webm"],
        }

        cfg_path = base_dir / "visual_ai_config.json"
        if cfg_path.exists():
            try:
                with open(cfg_path, 'r', encoding='utf-8') as f:
                    defaults.update(json.load(f))
            except Exception as e:
                safe_print(f"Config load error: {e}")

        if hasattr(args, 'source_dirs') and args.source_dirs:
            defaults["source_dirs"] = args.source_dirs

        valid_sources = self._validate_source_dirs(defaults["source_dirs"])

        for d in [defaults["except_dir"], defaults["user_recycle_dir"]]:
            Path(d).mkdir(parents=True, exist_ok=True)

        return Config(
            source_dirs=valid_sources,
            except_dir=Path(defaults["except_dir"]),
            user_recycle_dir=Path(defaults["user_recycle_dir"]),
            exclude_keywords=defaults["exclude_keywords"],
            video_extensions=set(defaults["video_extensions"]),
            max_workers=getattr(args, 'max_workers', Constants.DEFAULT_MAX_WORKERS),
            dry_run=getattr(args, 'dry_run', False),
            log_file=Path(args.log_file) if hasattr(args, 'log_file') and args.log_file else None,
            verbose=getattr(args, 'verbose', False),
            cache_db_path=base_dir / "visual_ai_cache.db"
        )

    async def run(self) -> None:
        """Parse arguments, scan directories, and process duplicates."""
        parser = argparse.ArgumentParser(description="Visual AI Duplicate Detector v6.1")
        parser.add_argument('-s', '--source-dirs', nargs='+', help='Directories to scan')
        parser.add_argument('-w', '--max-workers', type=int, default=Constants.DEFAULT_MAX_WORKERS)
        parser.add_argument('-n', '--dry-run', action='store_true')
        parser.add_argument('-l', '--log-file')
        parser.add_argument('-v', '--verbose', action='store_true')

        args = parser.parse_args()
        self.config = self.load_config(args)
        self.logger = setup_logging(self.config.log_file, self.config.verbose)
        self.pm = ProcessManager(self.logger)
        self.cache = MetadataCache(self.config.cache_db_path, self.logger)

        if self.config.dry_run:
            safe_print("\n*** DRY RUN MODE ***\n")

        scanner = MediaScanner(self.config, self.pm, self.logger, self.cache)
        await scanner.check_binaries()
        groups = await scanner.scan()

        fingerprinter = FingerprintGenerator(self.pm, scanner.ffmpeg_bin, self.logger, self.config.max_workers)
        detector = DuplicateDetector(self.config, self.logger, self.cache, fingerprinter)
        await detector.process(groups)

        fingerprinter.shutdown()
        self.cache.close()
        safe_print("\nDone.")


if __name__ == "__main__":
    if sys.platform == 'win32' and sys.version_info < (3, 13):
        try:
            asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
        except AttributeError:
            pass

    app = VisualAIApp()
    try:
        asyncio.run(app.run())
    except KeyboardInterrupt:
        safe_print("\nExiting...")
    except SystemExit:
        pass
    except Exception as e:
        import traceback
        traceback.print_exc()
        safe_print(f"\nCRITICAL ERROR: {e}")
    finally:
        input("\nPress Enter to exit...")
