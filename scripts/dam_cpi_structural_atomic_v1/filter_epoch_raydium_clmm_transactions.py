#!/usr/bin/env python3
"""
Filter epoch-style transaction dumps down to transactions that contain at least
one Raydium CLMM swap-family invocation.

The filter preserves the full original transaction row. It does not emit a
normalized row schema. The goal is to carve a Raydium CLMM-involved transaction
corpus out of a full-epoch dump while keeping complete transaction context
intact.
"""

from __future__ import annotations

import argparse
import base64
import concurrent.futures
import contextlib
import hashlib
import json
import multiprocessing
import os
import subprocess
import sys
import time
from collections import Counter
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, Iterable, Iterator, List, Optional, Sequence, Set, Tuple


DEFAULT_RAYDIUM_CLMM_PROGRAM_ID = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK"
DEFAULT_COMPRESSION_LEVEL = 3
DEFAULT_PROGRESS_EVERY = 100_000
DEFAULT_STATUS_EVERY_SEC = 5.0

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
BASE58_INDEX = {ch: idx for idx, ch in enumerate(BASE58_ALPHABET)}

SWAP_DISCRIMINATOR = hashlib.sha256(b"global:swap").digest()[:8]
SWAP_V2_DISCRIMINATOR = hashlib.sha256(b"global:swap_v2").digest()[:8]
SWAP_ROUTER_BASE_IN_DISCRIMINATOR = hashlib.sha256(b"global:swap_router_base_in").digest()[:8]


def safe_get_dict(value: Any) -> Dict[str, Any]:
    return value if isinstance(value, dict) else {}


def safe_get_list(value: Any) -> List[Any]:
    return value if isinstance(value, list) else []


def get_field(obj: Dict[str, Any], *names: str, default: Any = None) -> Any:
    if not isinstance(obj, dict):
        return default
    for name in names:
        if name in obj:
            return obj[name]
    return default


def coalesce_nonempty(*values: Any, default: str = "") -> str:
    for value in values:
        if value is None:
            continue
        text = str(value).strip()
        if text:
            return text
    return default


def to_int(value: Any, default: int = 0) -> int:
    if value is None:
        return default
    if isinstance(value, bool):
        return int(value)
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        return int(value)
    if isinstance(value, str):
        text = value.strip()
        if not text:
            return default
        try:
            return int(text)
        except ValueError:
            try:
                return int(float(text))
            except ValueError:
                return default
    return default


def decode_base64_maybe(text: str) -> bytes:
    raw = text.strip()
    if not raw:
        return b""
    try:
        return base64.b64decode(raw)
    except Exception:
        return b""


def base58_decode(text: str) -> bytes:
    raw = text.strip()
    if not raw:
        return b""

    num = 0
    for ch in raw:
        value = BASE58_INDEX.get(ch)
        if value is None:
            raise ValueError(f"invalid base58 character: {ch}")
        num = (num * 58) + value

    out = bytearray()
    while num > 0:
        num, rem = divmod(num, 256)
        out.append(rem)
    result = bytes(reversed(out))

    leading_zeros = 0
    for ch in raw:
        if ch == "1":
            leading_zeros += 1
        else:
            break
    if leading_zeros:
        result = (b"\x00" * leading_zeros) + result
    return result


def decode_u64_le(buf: bytes, offset: int) -> Tuple[int, int]:
    if offset + 8 > len(buf):
        raise ValueError("short buffer for u64")
    return int.from_bytes(buf[offset : offset + 8], "little"), offset + 8


def decode_u128_le(buf: bytes, offset: int) -> Tuple[int, int]:
    if offset + 16 > len(buf):
        raise ValueError("short buffer for u128")
    return int.from_bytes(buf[offset : offset + 16], "little"), offset + 16


def decode_bool(buf: bytes, offset: int) -> Tuple[bool, int]:
    if offset + 1 > len(buf):
        raise ValueError("short buffer for bool")
    value = buf[offset]
    if value not in (0, 1):
        raise ValueError("invalid bool encoding")
    return value == 1, offset + 1


def decode_raydium_clmm_instruction_args(data_bytes: bytes) -> Optional[Dict[str, Any]]:
    if len(data_bytes) < 8:
        return None

    discriminator = data_bytes[:8]
    offset = 8

    try:
        if discriminator == SWAP_DISCRIMINATOR:
            amount, offset = decode_u64_le(data_bytes, offset)
            other_amount_threshold, offset = decode_u64_le(data_bytes, offset)
            sqrt_price_limit_x64, offset = decode_u128_le(data_bytes, offset)
            is_base_input, offset = decode_bool(data_bytes, offset)
            return {
                "method": "swap",
                "amount": amount,
                "other_amount_threshold": other_amount_threshold,
                "sqrt_price_limit_x64": sqrt_price_limit_x64,
                "is_base_input": is_base_input,
            }

        if discriminator == SWAP_V2_DISCRIMINATOR:
            amount, offset = decode_u64_le(data_bytes, offset)
            other_amount_threshold, offset = decode_u64_le(data_bytes, offset)
            sqrt_price_limit_x64, offset = decode_u128_le(data_bytes, offset)
            is_base_input, offset = decode_bool(data_bytes, offset)
            return {
                "method": "swap_v2",
                "amount": amount,
                "other_amount_threshold": other_amount_threshold,
                "sqrt_price_limit_x64": sqrt_price_limit_x64,
                "is_base_input": is_base_input,
            }

        if discriminator == SWAP_ROUTER_BASE_IN_DISCRIMINATOR:
            amount_in, offset = decode_u64_le(data_bytes, offset)
            amount_out_minimum, offset = decode_u64_le(data_bytes, offset)
            return {
                "method": "swap_router_base_in",
                "amount_in": amount_in,
                "amount_out_minimum": amount_out_minimum,
            }
    except ValueError:
        return None

    return None


def iso_now() -> str:
    return datetime.now().astimezone().isoformat(timespec="seconds")


def iso_at(ts: float) -> str:
    return datetime.fromtimestamp(ts).astimezone().isoformat(timespec="seconds")


def write_json_atomic(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_name(path.name + ".tmp")
    tmp_path.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
    tmp_path.replace(path)


class JsonlAppendWriter:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._fh = self.path.open("a", encoding="utf-8")

    def write_row(self, row: Dict[str, Any]) -> None:
        self._fh.write(json.dumps(row, separators=(",", ":")))
        self._fh.write("\n")
        self._fh.flush()

    def close(self) -> None:
        self._fh.flush()
        self._fh.close()


class LazyShardWriter:
    def __init__(self, output_path: Path, *, compress: bool, compression_level: int, zstd_threads: int) -> None:
        self.output_path = output_path
        self.compress = compress
        self.compression_level = compression_level
        self.zstd_threads = zstd_threads
        self.temp_path = output_path.with_name(output_path.name + ".tmp")
        self._fh = None
        self._proc: Optional[subprocess.Popen[str]] = None
        self.rows_written = 0

    def _ensure_open(self) -> None:
        if self._fh is not None or self._proc is not None:
            return

        self.output_path.parent.mkdir(parents=True, exist_ok=True)
        if self.temp_path.exists():
            self.temp_path.unlink()

        if self.compress:
            self._proc = subprocess.Popen(
                [
                    "zstd",
                    "-q",
                    f"-{self.compression_level}",
                    f"-T{self.zstd_threads}",
                    "-f",
                    "-o",
                    str(self.temp_path),
                ],
                stdin=subprocess.PIPE,
                text=True,
                encoding="utf-8",
                errors="strict",
            )
            if self._proc.stdin is None:
                raise RuntimeError(f"failed to open zstd writer for {self.temp_path}")
            self._fh = self._proc.stdin
            return

        self._fh = self.temp_path.open("w", encoding="utf-8")

    def write_row(self, row: Dict[str, Any]) -> None:
        self._ensure_open()
        assert self._fh is not None
        self._fh.write(json.dumps(row, separators=(",", ":")))
        self._fh.write("\n")
        self.rows_written += 1

    def close(self) -> bool:
        wrote_anything = self.rows_written > 0

        if self._fh is None and self._proc is None:
            return False

        assert self._fh is not None
        self._fh.close()

        if self._proc is not None:
            rc = self._proc.wait()
            if rc != 0:
                raise RuntimeError(f"zstd exited with code {rc} while writing {self.output_path}")

        self._fh = None
        self._proc = None

        if wrote_anything:
            self.temp_path.replace(self.output_path)
        elif self.temp_path.exists():
            self.temp_path.unlink()

        return wrote_anything

    def abort(self) -> None:
        try:
            if self._fh is not None:
                self._fh.close()
        except Exception:
            pass
        if self._proc is not None:
            try:
                if self._proc.poll() is None:
                    self._proc.terminate()
                    self._proc.wait(timeout=2)
            except Exception:
                try:
                    if self._proc.poll() is None:
                        self._proc.kill()
                except Exception:
                    pass
        self._fh = None
        self._proc = None
        if self.temp_path.exists():
            self.temp_path.unlink()


@contextlib.contextmanager
def open_text_stream(path: Path) -> Iterator[Iterable[str]]:
    if str(path).endswith(".zst"):
        proc = subprocess.Popen(
            ["zstd", "-q", "-dc", str(path)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        try:
            if proc.stdout is None:
                raise RuntimeError(f"failed to open zstd stream for {path}")
            yield proc.stdout
        finally:
            if proc.stdout is not None:
                proc.stdout.close()
            if proc.stderr is not None:
                proc.stderr.close()
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=2)
    else:
        with path.open("r", encoding="utf-8", errors="replace") as fh:
            yield fh


def iter_jsonl_rows(path: Path) -> Iterator[Dict[str, Any]]:
    with open_text_stream(path) as lines:
        for line_no, raw_line in enumerate(lines, start=1):
            line = raw_line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError as exc:
                raise RuntimeError(f"invalid JSON at {path}:{line_no}: {exc}") from exc
            if isinstance(row, dict):
                yield row


def discover_input_files(input_paths: Sequence[str]) -> List[Tuple[Path, Path]]:
    discovered: List[Tuple[Path, Path]] = []
    seen: Set[str] = set()
    for raw in input_paths:
        path = Path(raw).expanduser().resolve()
        if not path.exists():
            raise RuntimeError(f"input does not exist: {path}")
        if path.is_dir():
            files = sorted(
                p
                for p in path.rglob("*")
                if p.is_file() and (p.name.endswith(".jsonl") or p.name.endswith(".jsonl.zst"))
            )
            for file_path in files:
                key = str(file_path)
                if key in seen:
                    continue
                seen.add(key)
                discovered.append((file_path, path))
            continue
        if not path.is_file():
            continue
        key = str(path)
        if key in seen:
            continue
        seen.add(key)
        discovered.append((path, path.parent))
    return discovered


def relative_path_for_output(file_path: Path, root: Path) -> str:
    try:
        return str(file_path.relative_to(root))
    except ValueError:
        return file_path.name


def resolve_full_account_keys_plain_rpc(tx: Dict[str, Any]) -> List[str]:
    tx_obj = safe_get_dict(get_field(tx, "transaction"))
    msg = safe_get_dict(get_field(tx_obj, "message"))
    meta = safe_get_dict(get_field(tx, "meta"))

    static_keys: List[str] = []
    for raw in safe_get_list(get_field(msg, "account_keys", "accountKeys")):
        if isinstance(raw, str):
            static_keys.append(raw)
        elif isinstance(raw, dict):
            static_keys.append(coalesce_nonempty(get_field(raw, "pubkey", "key")))

    loaded = safe_get_dict(get_field(meta, "loaded_addresses", "loadedAddresses"))
    writable = [str(x) for x in safe_get_list(get_field(loaded, "writable"))]
    readonly = [str(x) for x in safe_get_list(get_field(loaded, "readonly"))]
    return static_keys + writable + readonly


def resolve_compiled_instruction_rpc(ix: Dict[str, Any], full_keys: Sequence[str]) -> Optional[Dict[str, Any]]:
    if "program_id" in ix or "programId" in ix:
        program_id = coalesce_nonempty(get_field(ix, "program_id", "programId"))
    else:
        program_index = to_int(get_field(ix, "program_id_index", "programIdIndex"), default=-1)
        if program_index < 0 or program_index >= len(full_keys):
            return None
        program_id = full_keys[program_index]

    raw_accounts = safe_get_list(get_field(ix, "accounts", "account_indexes", "accountIndexes"))
    account_indices = [to_int(v, default=-1) for v in raw_accounts]
    accounts: List[str] = []
    if get_field(ix, "account_keys", default=None) is not None:
        accounts = [str(v) for v in safe_get_list(get_field(ix, "account_keys"))]
    elif raw_accounts and all(isinstance(v, str) for v in raw_accounts):
        accounts = [str(v) for v in raw_accounts]
        account_indices = []
    else:
        for idx in account_indices:
            if 0 <= idx < len(full_keys):
                accounts.append(full_keys[idx])
            else:
                accounts.append("")

    if get_field(ix, "data_bytes", default=None) is not None:
        data_bytes = bytes(get_field(ix, "data_bytes"))
    else:
        data_bytes = b""
        data_base64 = coalesce_nonempty(get_field(ix, "data_base64", "dataBase64"))
        if data_base64:
            data_bytes = decode_base64_maybe(data_base64)
        else:
            data_text = coalesce_nonempty(get_field(ix, "data", "data_base58"))
            try:
                data_bytes = base58_decode(data_text) if data_text else b""
            except ValueError:
                data_bytes = b""

    return {
        "program_id": program_id,
        "account_indices": account_indices,
        "accounts": accounts,
        "data_bytes": data_bytes,
        "stack_height": get_field(ix, "stack_height", "stackHeight"),
    }


def strip_zst_suffix(path: Path) -> Path:
    name = path.name
    if name.endswith(".zst"):
        return path.with_name(name[:-4])
    return path


def shard_output_path(
    file_path: Path,
    root: Path,
    output_root: Path,
    *,
    compress: bool,
) -> Path:
    relative_path = Path(relative_path_for_output(file_path, root))
    output_path = output_root / relative_path
    if compress:
        if output_path.name.endswith(".zst"):
            return output_path
        return output_path.with_name(output_path.name + ".zst")
    return strip_zst_suffix(output_path)


def tx_succeeded(row: Dict[str, Any]) -> bool:
    if "succeeded" in row:
        return bool(row["succeeded"])
    if "error" in row:
        return row["error"] is None
    meta = safe_get_dict(get_field(row, "meta"))
    if meta:
        return get_field(meta, "err") is None
    return True


def summarize_local_epoch_match(
    row: Dict[str, Any],
    *,
    raydium_program_id: str,
) -> Optional[Dict[str, Any]]:
    method_counts: Counter[str] = Counter()
    matched_ixs: List[Dict[str, Any]] = []

    for position, raw_ix in enumerate(safe_get_list(get_field(row, "outer_instructions"))):
        ix = safe_get_dict(raw_ix)
        program_id = coalesce_nonempty(get_field(ix, "program_id", "programId"))
        if program_id != raydium_program_id:
            continue
        data_bytes = b""
        data_base64 = coalesce_nonempty(get_field(ix, "data_base64", "dataBase64"))
        if data_base64:
            data_bytes = decode_base64_maybe(data_base64)
        decoded = decode_raydium_clmm_instruction_args(data_bytes)
        if decoded is None:
            continue
        method = str(decoded["method"])
        method_counts[method] += 1
        matched_ixs.append(
            {
                "top_level_ix_index": to_int(get_field(ix, "instruction_index"), default=position),
                "inner_ordinal": -1,
                "method": method,
            }
        )

    for raw_group in safe_get_list(get_field(row, "inner_instructions")):
        group = safe_get_dict(raw_group)
        top_level_ix_index = to_int(get_field(group, "parent_instruction_index"), default=-1)
        if top_level_ix_index < 0:
            continue
        for position, raw_ix in enumerate(safe_get_list(get_field(group, "instructions"))):
            ix = safe_get_dict(raw_ix)
            program_id = coalesce_nonempty(get_field(ix, "program_id", "programId"))
            if program_id != raydium_program_id:
                continue
            data_bytes = b""
            data_base64 = coalesce_nonempty(get_field(ix, "data_base64", "dataBase64"))
            if data_base64:
                data_bytes = decode_base64_maybe(data_base64)
            decoded = decode_raydium_clmm_instruction_args(data_bytes)
            if decoded is None:
                continue
            method = str(decoded["method"])
            method_counts[method] += 1
            matched_ixs.append(
                {
                    "top_level_ix_index": top_level_ix_index,
                    "inner_ordinal": to_int(get_field(ix, "inner_instruction_index"), default=position),
                    "method": method,
                }
            )

    if not method_counts:
        return None

    return {
        "matched_invocation_count": sum(method_counts.values()),
        "matched_method_counts": dict(sorted(method_counts.items())),
        "matched_invocations": matched_ixs,
    }


def summarize_plain_rpc_match(
    tx: Dict[str, Any],
    *,
    raydium_program_id: str,
) -> Optional[Dict[str, Any]]:
    tx_obj = safe_get_dict(get_field(tx, "transaction"))
    msg = safe_get_dict(get_field(tx_obj, "message"))
    meta = safe_get_dict(get_field(tx, "meta"))
    full_keys = resolve_full_account_keys_plain_rpc(tx)

    method_counts: Counter[str] = Counter()
    matched_ixs: List[Dict[str, Any]] = []

    for idx, raw_ix in enumerate(safe_get_list(get_field(msg, "instructions"))):
        ix = resolve_compiled_instruction_rpc(safe_get_dict(raw_ix), full_keys)
        if ix is None or ix.get("program_id") != raydium_program_id:
            continue
        decoded = decode_raydium_clmm_instruction_args(ix.get("data_bytes", b""))
        if decoded is None:
            continue
        method = str(decoded["method"])
        method_counts[method] += 1
        matched_ixs.append(
            {
                "top_level_ix_index": idx,
                "inner_ordinal": -1,
                "method": method,
            }
        )

    for raw_group in safe_get_list(get_field(meta, "inner_instructions", "innerInstructions")):
        group = safe_get_dict(raw_group)
        top_level_ix_index = to_int(
            get_field(group, "index", "parent_instruction_index", "parentInstructionIndex"),
            default=-1,
        )
        if top_level_ix_index < 0:
            continue
        for position, raw_ix in enumerate(safe_get_list(get_field(group, "instructions"))):
            ix = resolve_compiled_instruction_rpc(safe_get_dict(raw_ix), full_keys)
            if ix is None or ix.get("program_id") != raydium_program_id:
                continue
            decoded = decode_raydium_clmm_instruction_args(ix.get("data_bytes", b""))
            if decoded is None:
                continue
            method = str(decoded["method"])
            method_counts[method] += 1
            matched_ixs.append(
                {
                    "top_level_ix_index": top_level_ix_index,
                    "inner_ordinal": to_int(
                        get_field(safe_get_dict(raw_ix), "inner_instruction_index", "innerInstructionIndex"),
                        default=position,
                    ),
                    "method": method,
                }
            )

    if not method_counts:
        return None

    return {
        "matched_invocation_count": sum(method_counts.values()),
        "matched_method_counts": dict(sorted(method_counts.items())),
        "matched_invocations": matched_ixs,
    }


def summarize_row_match(
    row: Dict[str, Any],
    *,
    raydium_program_id: str,
) -> Optional[Dict[str, Any]]:
    if get_field(row, "outer_instructions", default=None) is not None:
        return summarize_local_epoch_match(row, raydium_program_id=raydium_program_id)

    if isinstance(get_field(row, "rpc_like_tx"), dict):
        return summarize_plain_rpc_match(
            safe_get_dict(get_field(row, "rpc_like_tx")),
            raydium_program_id=raydium_program_id,
        )

    if isinstance(get_field(row, "transaction"), dict) and isinstance(get_field(row, "meta"), dict):
        return summarize_plain_rpc_match(row, raydium_program_id=raydium_program_id)

    return None


def resolve_worker_count(requested_jobs: int) -> int:
    if requested_jobs > 0:
        return requested_jobs
    cpu_count = os.cpu_count() or 1
    return max(1, cpu_count)


def resolve_zstd_threads(*, requested_threads: Optional[int], jobs: int) -> int:
    if requested_threads is not None:
        return max(0, requested_threads)
    if jobs <= 1:
        return 0
    return 1


def process_planned_entry(
    entry: Dict[str, Any],
    *,
    file_index: int,
    raydium_program_id: str,
    include_failed: bool,
    compress_output: bool,
    compression_level: int,
    zstd_threads: int,
) -> Dict[str, Any]:
    file_path = Path(str(entry["file_path"]))
    output_path = Path(str(entry["output_path"]))
    relative_source_path = str(entry["source_relative_path"])
    absolute_source_path = str(entry["source_path"])
    stale_partial_output_path = output_path.with_name(output_path.name + ".partial")
    if stale_partial_output_path.exists():
        stale_partial_output_path.unlink()

    per_file_rows_seen = 0
    per_file_failed_rows_skipped = 0
    per_file_rows_kept = 0
    per_file_matched_invocations = 0
    per_file_method_counts: Counter[str] = Counter()
    writer = LazyShardWriter(
        output_path,
        compress=compress_output,
        compression_level=compression_level,
        zstd_threads=zstd_threads,
    )
    file_start = time.time()

    try:
        for row in iter_jsonl_rows(file_path):
            per_file_rows_seen += 1

            if not include_failed and not tx_succeeded(row):
                per_file_failed_rows_skipped += 1
                continue

            match_summary = summarize_row_match(
                row,
                raydium_program_id=raydium_program_id,
            )
            if match_summary is None:
                continue

            writer.write_row(row)
            per_file_rows_kept += 1
            invocation_count = to_int(match_summary.get("matched_invocation_count"), default=0)
            per_file_matched_invocations += invocation_count
            for method, count in safe_get_dict(match_summary.get("matched_method_counts")).items():
                per_file_method_counts[str(method)] += to_int(count, default=0)

        wrote_output = writer.close()
    except Exception as exc:
        writer.abort()
        raise RuntimeError(f"failed while processing {absolute_source_path}: {exc}") from exc

    duration_sec = round(time.time() - file_start, 3)
    return {
        "status": "completed",
        "source_path": absolute_source_path,
        "source_relative_path": relative_source_path,
        "output_path": str(output_path) if wrote_output else "",
        "input_file_index": file_index,
        "rows_seen": per_file_rows_seen,
        "failed_rows_skipped": per_file_failed_rows_skipped,
        "rows_kept": per_file_rows_kept,
        "matched_invocations": per_file_matched_invocations,
        "matched_method_counts": dict(sorted(per_file_method_counts.items())),
        "duration_sec": duration_sec,
        "compressed_output": bool(compress_output),
    }


def load_completed_source_paths(manifest_path: Path) -> Set[str]:
    completed: Set[str] = set()
    if not manifest_path.exists():
        return completed

    with manifest_path.open("r", encoding="utf-8", errors="replace") as fh:
        for line in fh:
            text = line.strip()
            if not text:
                continue
            try:
                row = json.loads(text)
            except json.JSONDecodeError:
                continue
            if row.get("status") != "completed":
                continue
            source_path = str(row.get("source_path", "")).strip()
            if source_path:
                completed.add(source_path)
    return completed


def print_progress(payload: Dict[str, Any]) -> None:
    print(json.dumps(payload, separators=(",", ":")), file=sys.stderr)


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", action="append", required=True, help="Input file or directory. Repeatable.")
    parser.add_argument("--output-root", required=True, help="Root directory for filtered output shards.")
    parser.add_argument("--manifest-path", default=None, help="JSONL manifest path. Defaults under output root.")
    parser.add_argument("--status-path", default=None, help="Heartbeat JSON path. Defaults under output root.")
    parser.add_argument("--raydium-program-id", default=DEFAULT_RAYDIUM_CLMM_PROGRAM_ID)
    parser.add_argument("--include-failed", action="store_true", help="Keep failed transactions if they match.")
    parser.add_argument("--compress-output", action="store_true", default=True, help="Write .zst shards. Default on.")
    parser.add_argument("--no-compress-output", dest="compress_output", action="store_false", help="Write plain JSONL shards.")
    parser.add_argument("--compression-level", type=int, default=DEFAULT_COMPRESSION_LEVEL)
    parser.add_argument("--jobs", type=int, default=1, help="Worker processes. Use 0 for all CPUs.")
    parser.add_argument("--zstd-threads", type=int, default=None, help="zstd threads per output writer. Default auto: 0 for single-worker, 1 for multi-worker.")
    parser.add_argument("--resume", action="store_true", default=True, help="Skip source shards already marked completed in the manifest. Default on.")
    parser.add_argument("--no-resume", dest="resume", action="store_false", help="Ignore manifest completion state.")
    parser.add_argument("--overwrite", action="store_true", help="Overwrite existing output shards.")
    parser.add_argument("--limit-files", type=int, default=0, help="Stop after N input shards.")
    parser.add_argument("--limit-rows", type=int, default=0, help="Stop after N total input rows.")
    parser.add_argument("--progress-every", type=int, default=DEFAULT_PROGRESS_EVERY, help="Emit stderr progress every N input rows.")
    parser.add_argument("--status-every-sec", type=float, default=DEFAULT_STATUS_EVERY_SEC, help="Emit heartbeat progress at least this often in multi-worker mode.")
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(argv)
    jobs = resolve_worker_count(args.jobs)
    zstd_threads = resolve_zstd_threads(requested_threads=args.zstd_threads, jobs=jobs)

    if jobs > 1 and args.limit_rows > 0:
        raise SystemExit("--limit-rows is only supported with --jobs 1")

    output_root = Path(args.output_root).expanduser().resolve()
    output_root.mkdir(parents=True, exist_ok=True)

    manifest_path = (
        Path(args.manifest_path).expanduser().resolve()
        if args.manifest_path
        else output_root / "manifests" / "filter_manifest.jsonl"
    )
    status_path = (
        Path(args.status_path).expanduser().resolve()
        if args.status_path
        else output_root / "manifests" / "filter_status.json"
    )
    manifest_writer = JsonlAppendWriter(manifest_path)

    completed_sources = load_completed_source_paths(manifest_path) if args.resume and not args.overwrite else set()
    input_files = discover_input_files(args.input)

    planned_entries: List[Dict[str, Any]] = []
    total_files_seen = 0
    total_files_skipped_completed = 0
    total_files_skipped_existing_output = 0
    for file_path, root in input_files:
        total_files_seen += 1
        absolute_source_path = str(file_path)
        output_path = shard_output_path(
            file_path,
            root,
            output_root,
            compress=args.compress_output,
        )
        if absolute_source_path in completed_sources:
            total_files_skipped_completed += 1
            continue
        if output_path.exists() and not args.overwrite:
            total_files_skipped_existing_output += 1
            continue
        planned_entries.append(
            {
                "file_path": file_path,
                "root": root,
                "source_path": absolute_source_path,
                "source_relative_path": relative_path_for_output(file_path, root),
                "output_path": output_path,
                "size_bytes": file_path.stat().st_size,
            }
        )

    if args.limit_files > 0:
        planned_entries = planned_entries[: args.limit_files]

    planned_files_total = len(planned_entries)
    planned_input_bytes_total = sum(int(entry["size_bytes"]) for entry in planned_entries)

    total_files_completed = 0
    total_files_partial = 0
    total_input_rows = 0
    total_failed_rows_skipped = 0
    total_rows_kept = 0
    total_matched_invocations = 0
    total_method_counts: Counter[str] = Counter()
    completed_input_bytes = 0
    completed_input_rows = 0
    overall_start = time.time()

    def build_status_payload(
        event: str,
        *,
        current_entry: Optional[Dict[str, Any]] = None,
        current_file_rows_seen: int = 0,
        current_file_elapsed_sec: float = 0.0,
        active_entries: Optional[List[Dict[str, Any]]] = None,
    ) -> Dict[str, Any]:
        elapsed_sec = max(0.0, time.time() - overall_start)
        current_file_size_bytes = int(current_entry["size_bytes"]) if current_entry is not None else 0
        current_file_bytes_estimate = 0
        active_source_paths: List[str] = []

        if current_entry is not None and current_file_rows_seen > 0 and completed_input_rows > 0 and completed_input_bytes > 0:
            avg_bytes_per_row = completed_input_bytes / completed_input_rows
            current_file_bytes_estimate = min(
                current_file_size_bytes,
                int(current_file_rows_seen * avg_bytes_per_row),
            )

        if active_entries:
            active_source_paths = [str(entry["source_relative_path"]) for entry in active_entries[:8]]
        input_bytes_done_estimate = completed_input_bytes + current_file_bytes_estimate
        progress_fraction_estimate = (
            (input_bytes_done_estimate / planned_input_bytes_total)
            if planned_input_bytes_total > 0
            else 1.0
        )
        rows_per_sec = (
            (total_input_rows / elapsed_sec)
            if elapsed_sec > 0 and total_input_rows > 0
            else None
        )
        input_bytes_per_sec = (
            (input_bytes_done_estimate / elapsed_sec)
            if elapsed_sec > 0 and input_bytes_done_estimate > 0
            else None
        )
        estimated_remaining_sec = None
        estimated_completion_time = None
        if (
            input_bytes_per_sec is not None
            and input_bytes_per_sec > 0
            and planned_input_bytes_total > input_bytes_done_estimate
        ):
            estimated_remaining_sec = (planned_input_bytes_total - input_bytes_done_estimate) / input_bytes_per_sec
            estimated_completion_time = iso_at(time.time() + estimated_remaining_sec)
        elif planned_input_bytes_total > 0 and input_bytes_done_estimate >= planned_input_bytes_total:
            estimated_remaining_sec = 0.0
            estimated_completion_time = iso_now()

        return {
            "event": event,
            "status_time": iso_now(),
            "output_root": str(output_root),
            "manifest_path": str(manifest_path),
            "jobs": jobs,
            "zstd_threads": zstd_threads,
            "planned_files_total": planned_files_total,
            "planned_files_completed": total_files_completed,
            "planned_files_partial": total_files_partial,
            "planned_files_remaining": max(0, planned_files_total - total_files_completed - total_files_partial),
            "pre_skipped_completed": total_files_skipped_completed,
            "pre_skipped_existing_output": total_files_skipped_existing_output,
            "planned_input_bytes_total": planned_input_bytes_total,
            "completed_input_bytes": completed_input_bytes,
            "input_bytes_done_estimate": input_bytes_done_estimate,
            "progress_fraction_estimate": progress_fraction_estimate,
            "elapsed_sec": round(elapsed_sec, 3),
            "rows_per_sec": round(rows_per_sec, 3) if rows_per_sec is not None else None,
            "input_mib_per_sec": round((input_bytes_per_sec / (1024 * 1024)), 3) if input_bytes_per_sec is not None else None,
            "estimated_remaining_sec": round(estimated_remaining_sec, 3) if estimated_remaining_sec is not None else None,
            "estimated_completion_time": estimated_completion_time,
            "total_input_rows": total_input_rows,
            "failed_rows_skipped": total_failed_rows_skipped,
            "total_rows_kept": total_rows_kept,
            "matched_invocations": total_matched_invocations,
            "matched_method_counts": dict(sorted(total_method_counts.items())),
            "active_workers": len(active_entries) if active_entries is not None else (1 if current_entry is not None else 0),
            "active_source_paths": active_source_paths or ([str(current_entry["source_relative_path"])] if current_entry is not None else []),
            "current_source_path": current_entry["source_path"] if current_entry is not None else None,
            "current_source_relative_path": current_entry["source_relative_path"] if current_entry is not None else None,
            "current_file_size_bytes": current_file_size_bytes if current_entry is not None else None,
            "current_file_rows_seen": current_file_rows_seen if current_entry is not None else None,
            "current_file_elapsed_sec": round(current_file_elapsed_sec, 3) if current_entry is not None else None,
        }

    def emit_status(
        event: str,
        *,
        current_entry: Optional[Dict[str, Any]] = None,
        current_file_rows_seen: int = 0,
        current_file_elapsed_sec: float = 0.0,
        active_entries: Optional[List[Dict[str, Any]]] = None,
    ) -> None:
        payload = build_status_payload(
            event,
            current_entry=current_entry,
            current_file_rows_seen=current_file_rows_seen,
            current_file_elapsed_sec=current_file_elapsed_sec,
            active_entries=active_entries,
        )
        write_json_atomic(status_path, payload)
        print_progress(payload)

    try:
        if jobs == 1:
            emit_status("started")
            for file_index, entry in enumerate(planned_entries, start=1):
                file_path = Path(entry["file_path"])
                relative_source_path = str(entry["source_relative_path"])
                absolute_source_path = str(entry["source_path"])
                output_path = Path(entry["output_path"])
                stale_partial_output_path = output_path.with_name(output_path.name + ".partial")
                if stale_partial_output_path.exists():
                    stale_partial_output_path.unlink()
                per_file_rows_seen = 0
                per_file_failed_rows_skipped = 0
                per_file_rows_kept = 0
                per_file_matched_invocations = 0
                per_file_method_counts: Counter[str] = Counter()
                file_hit_limit = False
                writer = LazyShardWriter(
                    output_path,
                    compress=args.compress_output,
                    compression_level=args.compression_level,
                    zstd_threads=zstd_threads,
                )
                file_start = time.time()

                try:
                    for row in iter_jsonl_rows(file_path):
                        if args.limit_rows > 0 and total_input_rows >= args.limit_rows:
                            file_hit_limit = True
                            break

                        total_input_rows += 1
                        per_file_rows_seen += 1
                        current_file_elapsed_sec = time.time() - file_start

                        if not args.include_failed and not tx_succeeded(row):
                            total_failed_rows_skipped += 1
                            per_file_failed_rows_skipped += 1
                            continue

                        match_summary = summarize_row_match(
                            row,
                            raydium_program_id=args.raydium_program_id,
                        )
                        if match_summary is None:
                            if args.progress_every > 0 and total_input_rows % args.progress_every == 0:
                                emit_status(
                                    "progress",
                                    current_entry=entry,
                                    current_file_rows_seen=per_file_rows_seen,
                                    current_file_elapsed_sec=current_file_elapsed_sec,
                                )
                            continue

                        writer.write_row(row)
                        per_file_rows_kept += 1
                        total_rows_kept += 1
                        invocation_count = to_int(match_summary.get("matched_invocation_count"), default=0)
                        per_file_matched_invocations += invocation_count
                        total_matched_invocations += invocation_count
                        for method, count in safe_get_dict(match_summary.get("matched_method_counts")).items():
                            int_count = to_int(count, default=0)
                            per_file_method_counts[str(method)] += int_count
                            total_method_counts[str(method)] += int_count

                        if args.progress_every > 0 and total_input_rows % args.progress_every == 0:
                            emit_status(
                                "progress",
                                current_entry=entry,
                                current_file_rows_seen=per_file_rows_seen,
                                current_file_elapsed_sec=current_file_elapsed_sec,
                            )

                    wrote_output = writer.close()
                except Exception:
                    writer.abort()
                    raise

                final_output_path = output_path
                if file_hit_limit and wrote_output:
                    partial_output_path = output_path.with_name(output_path.name + ".partial")
                    if partial_output_path.exists():
                        partial_output_path.unlink()
                    output_path.replace(partial_output_path)
                    final_output_path = partial_output_path

                duration_sec = round(time.time() - file_start, 3)
                if file_hit_limit:
                    total_files_partial += 1
                else:
                    total_files_completed += 1
                    completed_input_bytes += int(entry["size_bytes"])
                    completed_input_rows += per_file_rows_seen

                manifest_writer.write_row(
                    {
                        "status": "partial" if file_hit_limit else "completed",
                        "source_path": absolute_source_path,
                        "source_relative_path": relative_source_path,
                        "output_path": str(final_output_path) if wrote_output else "",
                        "input_file_index": file_index,
                        "rows_seen": per_file_rows_seen,
                        "failed_rows_skipped": per_file_failed_rows_skipped,
                        "rows_kept": per_file_rows_kept,
                        "matched_invocations": per_file_matched_invocations,
                        "matched_method_counts": dict(sorted(per_file_method_counts.items())),
                        "duration_sec": duration_sec,
                        "compressed_output": bool(args.compress_output),
                    }
                )

                emit_status(
                    "file_partial" if file_hit_limit else "file_completed",
                    current_entry=entry if file_hit_limit else None,
                    current_file_rows_seen=per_file_rows_seen if file_hit_limit else 0,
                    current_file_elapsed_sec=duration_sec if file_hit_limit else 0.0,
                )

                if args.limit_rows > 0 and total_input_rows >= args.limit_rows:
                    break
        else:
            process_ctx = multiprocessing.get_context("spawn")
            pending_index = 0
            inflight: Dict[concurrent.futures.Future[Dict[str, Any]], Dict[str, Any]] = {}

            def submit_next(executor: concurrent.futures.ProcessPoolExecutor) -> bool:
                nonlocal pending_index
                if pending_index >= planned_files_total:
                    return False
                file_index = pending_index + 1
                entry = planned_entries[pending_index]
                future = executor.submit(
                    process_planned_entry,
                    entry,
                    file_index=file_index,
                    raydium_program_id=args.raydium_program_id,
                    include_failed=args.include_failed,
                    compress_output=args.compress_output,
                    compression_level=args.compression_level,
                    zstd_threads=zstd_threads,
                )
                inflight[future] = {
                    **entry,
                    "file_index": file_index,
                    "started_at": time.time(),
                }
                pending_index += 1
                return True

            with concurrent.futures.ProcessPoolExecutor(max_workers=jobs, mp_context=process_ctx) as executor:
                for _ in range(min(jobs, planned_files_total)):
                    submit_next(executor)

                emit_status("started", active_entries=list(inflight.values()))
                last_status_emit = time.time()

                while inflight:
                    done, _ = concurrent.futures.wait(
                        list(inflight.keys()),
                        timeout=1.0,
                        return_when=concurrent.futures.FIRST_COMPLETED,
                    )

                    if not done:
                        if args.status_every_sec > 0 and (time.time() - last_status_emit) >= args.status_every_sec:
                            emit_status("progress", active_entries=list(inflight.values()))
                            last_status_emit = time.time()
                        continue

                    for future in done:
                        try:
                            result = future.result()
                        except Exception:
                            for pending_future in inflight:
                                pending_future.cancel()
                            executor.shutdown(wait=False, cancel_futures=True)
                            raise
                        active_entry = inflight.pop(future)

                        total_files_completed += 1
                        total_input_rows += int(result["rows_seen"])
                        total_failed_rows_skipped += int(result["failed_rows_skipped"])
                        total_rows_kept += int(result["rows_kept"])
                        total_matched_invocations += int(result["matched_invocations"])
                        completed_input_bytes += int(active_entry["size_bytes"])
                        completed_input_rows += int(result["rows_seen"])
                        for method, count in safe_get_dict(result["matched_method_counts"]).items():
                            total_method_counts[str(method)] += to_int(count, default=0)

                        manifest_writer.write_row(result)

                        submit_next(executor)
                        emit_status("file_completed", active_entries=list(inflight.values()))
                        last_status_emit = time.time()
    finally:
        manifest_writer.close()

    summary = {
        "output_root": str(output_root),
        "manifest_path": str(manifest_path),
        "status_path": str(status_path),
        "jobs": jobs,
        "zstd_threads": zstd_threads,
        "files_seen": total_files_seen,
        "files_completed": total_files_completed,
        "files_partial": total_files_partial,
        "files_skipped_completed": total_files_skipped_completed,
        "files_skipped_existing_output": total_files_skipped_existing_output,
        "input_rows": total_input_rows,
        "planned_files_total": planned_files_total,
        "planned_input_bytes_total": planned_input_bytes_total,
        "completed_input_bytes": completed_input_bytes,
        "failed_rows_skipped": total_failed_rows_skipped,
        "rows_kept": total_rows_kept,
        "matched_invocations": total_matched_invocations,
        "matched_method_counts": dict(sorted(total_method_counts.items())),
        "duration_sec": round(time.time() - overall_start, 3),
    }
    final_status = build_status_payload("finished")
    final_status.update(summary)
    write_json_atomic(status_path, final_status)
    print(json.dumps(summary, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
