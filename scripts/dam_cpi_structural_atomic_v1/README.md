# DAM CPI Structural Atomic V1 Raydium Extraction

This directory holds the off-chain transaction filter used to carve a Raydium
CLMM-involved corpus out of a full Solana epoch dump.

## Script

- [filter_epoch_raydium_clmm_transactions.py](filter_epoch_raydium_clmm_transactions.py)

The filter:

- matches Raydium CLMM `swap`, `swap_v2`, and `swap_router_base_in`
- keys on program id `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK`
- preserves the full original transaction row
- supports `.jsonl` and `.jsonl.zst` inputs
- supports resume, manifest tracking, heartbeat status, and multi-process
  shard execution

## Current Local Artifact

The extraction completed on `2026-04-16`.

On the machine used for this run, the artifacts lived under local, unversioned
paths. The exact workstation-specific paths are intentionally omitted from the
repository copy of this README.

- source corpus:
  `<local epoch dump root>`
- output root:
  `<local filtered output root>`
- manifest:
  `<local filtered output root>/manifests/filter_manifest.jsonl`
- heartbeat:
  `<local filtered output root>/manifests/filter_status.json`

These data artifacts are local working outputs. They are not checked into this
repository.

## Extracted Totals

- `877` source shards scanned
- `877` completed output shards
- `2,763,698` kept transactions
- `3,368,063` matched swap-family invocations
- matched method counts:
  - `swap`: `525,350`
  - `swap_v2`: `2,842,713`
  - `swap_router_base_in`: `0`
- `50,675,671` failed transactions skipped because the run did not use
  `--include-failed`
- `218,093,708` total input rows scanned
- `186,123,801,663` total input bytes scanned
- `1,406.987` seconds total wall time

Transactions and invocations are separate counts. A single kept transaction may
contain more than one Raydium CLMM invocation.

## Validation Notes

- `event: finished`
- `files_completed: 877`
- `files_partial: 0`
- `planned_files_remaining: 0`
- all output shards were nonempty
- no `.partial` or `.tmp` artifacts remained after completion

This run observed `swap` and `swap_v2` in epoch `946`. It did not observe
`swap_router_base_in` in this extraction.

## Downstream Use

- input corpus for Raydium row extraction
- input corpus for Raydium structural labeling
- transaction-context validation for DAM v1 feature work

This output is a transaction-preserving filter corpus. It is not yet a
normalized training-row dataset and it is not yet labeled.
