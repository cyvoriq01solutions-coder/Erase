# A1: Advance scan skeleton, Report D and the CYVRA Grading Engine

Advance scan is the deep, opt-in counterpart to the basic local assessment. This slice lands the
boundary, the report shape and the grading arithmetic **with no low-level collection at all**, so the
honest empty state is provable before a single IOCTL exists.

Basic scan and Report A are unchanged. Navigation still has exactly five destinations; Advance scan is
a button on Verification, not a sixth destination.

## What A1 adds

| Layer | Addition |
| --- | --- |
| Contract | `agent-windows/src/hardware_diagnostics_v1.rs` — Report D records, domain weights, coverage arithmetic, grade bands |
| Engine | `agent-windows/src/diagnostics.rs` — Advance scan orchestrator and Report D row builder |
| Bridge | `run_advance_scan` Tauri command (sixth), read-only, no process or filesystem access in the shell layer |
| UI | Advance scan panel with two off-by-default permissions, and the Report D block with a grade card |

## The grading rule that matters

Rubric CG-1.0 weights six domains to 100 points: battery and power 20, processor and thermal 20,
memory 15, storage health 20, ports and connectivity 10, screen and peripherals 15.

Each domain reports `awarded`, `assessed` and `not_assessable`. Two invariants are enforced in code,
not left to the caller:

- points are never awarded for evidence we do not hold;
- points are never deducted for a subsystem we could not measure.

The headline is therefore always printed as a pair: an Assessed Health Index over the points that were
actually measurable, next to the coverage percentage.

Precedence when choosing a band:

1. a **measured** critical fault (memory mismatch, predicted storage failure, NVMe critical warning,
   pending sectors) forces `F`, because that is evidence we hold;
2. a mandatory domain with nothing assessable **withholds** the grade — storage always, battery only on
   a chassis that should have one;
3. coverage below 70% withholds the grade;
4. otherwise the index selects `A+ / A / B / C / F`.

`Grade withheld` is never rendered as `F`. A desktop with no battery is still gradable: the battery
weight leaves the denominator entirely instead of counting as a gap.

Because collection has not landed yet, A1 on real hardware correctly shows **coverage 0%**, no index,
and a withheld grade naming storage as the missing mandatory area. That is the intended demonstration.

## Boundary

- Advance scan is read-only in this slice. Benchmarks and the temporary write test are both off by
  default and, in A1, nothing runs even when permitted.
- `bytesWritten` is printed on Report D every time, so the write boundary is always visible.
- If any byte were ever recorded as written without consent, the scan fails closed in the core and the
  bridge rejects the result a second time.
- CPU package temperature and fan speed are reported as requiring a kernel-mode sensor driver, which
  CYVRA deliberately does not ship. They are not silently omitted and never inferred.

## Wording

The grade block prints exactly **Graded by CYVRA Grading Engine** plus the rubric identifier. The
engine is a deterministic rubric; no machine-learning inference is used, so no AI claim is made
anywhere in the product. Report D is `Issued by CYVORIQ Solutions Pvt. Ltd.` and states that it is
computer-generated and not cloud-authenticated in this version. Grading issuance stays disabled, so
every grade is provisional and physical verification is required.

## Still off

- Deep collection (battery IOCTL, storage SMART, USB topology, EDID, radios, capture devices)
- Benchmarks and interactive technician checks
- Report D signing, cloud authentication and issuance
- Destructive operations, WinPE, Authenticode signing, unsigned B2 upload

## Next slices

`A2` battery, `A3` cameras and microphones plus USB topology, `A4` storage SMART through a bounded
elevated probe, `A5` display and radios, `A6` consent-gated benchmarks, `A7` interactive suite,
`A8` wires real evidence into the grading engine that A1 already ships.
