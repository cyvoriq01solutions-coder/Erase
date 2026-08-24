# CYVORIQ ERASE — W2.2 CYVRA QC Grading Contract — 2026-08-24

> **Status:** PROPOSED — OWNER REVIEW REQUIRED
>
> **Branch:** `w2-2-cyvra-qc-grading-contract`
>
> **Repository baseline:** `06bfb5785f8a9edcdbad469c15ed43a474c0ca0e`
>
> **Scope:** CYVRA QC device-condition grading rules, evidence requirements, grade states, scoring, review, integrity, privacy and customer presentation for the shared CYVRA Windows application.
>
> **Change boundary:** This document authorizes no production deployment, database mutation, secret change, code-signing operation, destructive erasure, resale valuation, auction decision, customer release or grading-engine implementation before separate approval.

## 1. Purpose and precedence

This contract defines when CYVRA QC may issue a customer-facing device grade and what that grade means.

It must be read with:

- [W2.1 Shared Customer GUI/UX Contract — 2026-08-24](w2-1-shared-customer-gui-ux-contract-2026-08-24.md)
- [W1.1 Desktop, Installer and Passive Hardware Inventory Freeze — 2026-08-21](w1-1-desktop-installer-hardware-freeze-2026-08-21.md)
- [Current Product Freeze & Team Handoff — 2026-08-21](current-product-freeze-handoff-2026-08-21.md)
- [Frontend Commercial V1 Freeze](frontend-commercial-v1-freeze.md)

W2.1 freezes CYVRA QC as the device-verification and grading domain inside the single CYVRA Erase application. This document does not create a second product, installer, activation, device binding, scan or report.

Safety, privacy, entitlement, evidence-integrity, report-authentication and release gates remain in force. If a grading convenience conflicts with those controls, grading must stop and the conflict must be reviewed.

## 2. Product meaning of a CYVRA QC grade

A CYVRA QC grade is a versioned, evidence-based description of the assessed physical device condition at a recorded point in time.

The grade combines approved evidence about:

- core system operation;
- integrated display and input where applicable;
- storage condition;
- battery and power where applicable;
- connectivity, ports and audio; and
- cosmetic and structural condition.

A grade is not:

- a resale price;
- an auction reserve or bid recommendation;
- a warranty;
- a safety certification;
- proof that every component works;
- proof that data was erased;
- a benchmark or performance ranking;
- a measure of CPU, memory or storage specification value;
- a legal-compliance certification; or
- a substitute for a buyer's inspection.

Device specification and device condition must remain visibly separate. A lower-spec device in excellent condition may receive a higher condition grade than a higher-spec device with defects.

## 3. Customer-facing grade scale

CYVRA QC V1 proposes five commercial condition grades without plus or minus variants.

| Grade | Customer descriptor | Condition meaning |
| --- | --- | --- |
| A | Excellent | Strong evidence shows excellent assessed condition, no material defect and only negligible wear |
| B | Good | Strong evidence shows good assessed condition with minor wear or minor non-critical limitations |
| C | Fair | Evidence shows usable assessed condition with visible wear or one or more material but disclosed limitations |
| D | Limited | Evidence shows major limitations, significant wear or confirmed defects that materially affect normal use |
| E | Major limitations | Evidence shows severe condition problems or multiple confirmed defects, while still meeting the minimum evidence required to grade |

`Unable to grade` is not Grade E. Grade E describes poor condition supported by sufficient evidence. `Unable to grade` means CYVRA cannot make a defensible condition determination.

The customer interface must show the letter and descriptor together, for example `Grade B — Good`.

The report must include the grading profile and rules version. A customer-visible numeric score is not required in V1 and must not be presented as a monetary value.

## 4. Grade lifecycle and customer states

The allowed lifecycle states are:

| Internal state | Customer presentation | Meaning |
| --- | --- | --- |
| `pending` | `Grade pending` | Assessment has not completed |
| `evidence_in_progress` | `Grade pending` | Required evidence is being collected or reviewed |
| `manual_review_required` | `Grade pending — review required` | Evidence needs authorized review before calculation |
| `issued` | Actual grade and descriptor | A server-authoritative grade record has been issued |
| `unable_insufficient_evidence` | `Unable to grade — insufficient evidence` | Mandatory evidence or coverage is missing, unavailable or conflicting |
| `unable_grading_error` | `Unable to grade — grading error` | A bounded calculation, integrity or service error prevented grading |
| `superseded` | Newer grade available | A later immutable grade revision replaced this revision |
| `revoked` | `Grade unavailable — contact support` | An integrity or authorization decision invalidated the issued revision |

`Grade not issued` is permitted only in unfinished internal development builds. It is not a customer-release result.

Cancellation must not produce an issued grade. Partial evidence may be retained only under the approved privacy and retry policy.

## 5. Supported V1 grading profiles

The initial profile family is `cyvra_qc_condition_v1`.

Proposed supported physical-device profiles are:

- `windows_laptop_v1`;
- `windows_tablet_v1`;
- `windows_desktop_v1`; and
- `windows_all_in_one_v1`.

The device must run a supported Windows 10 or Windows 11 release and complete the approved CYVRA assessment flow.

The following are not eligible for a physical condition grade under this contract:

- virtual machines;
- Windows Server systems;
- devices that cannot run the authorized application;
- standalone peripherals assessed without a supported host-device profile;
- devices whose class cannot be resolved with sufficient confidence; and
- devices outside the approved Windows architecture and test matrix.

An ineligible device must receive an explicit insufficient-evidence or unsupported-profile reason. It must not be assigned a guessed grade.

## 6. Separation of facts, evidence, condition and confidence

CYVRA must keep four concepts separate:

1. **Specification fact:** what the device reports it contains.
2. **Evidence item:** the recorded source supporting an observation.
3. **Condition outcome:** the approved interpretation of that evidence for one grading check.
4. **Evidence confidence:** how strongly the evidence supports the outcome.

Presence is not proof of function. A detected camera, battery, port, memory module or storage device must not automatically receive a passing condition outcome.

The Hardware Inventory V1 statuses remain authoritative:

- reported;
- observed;
- derived;
- unknown;
- not_reported;
- not_applicable;
- permission_denied;
- unsupported; and
- collection_error.

`unknown`, `not_reported`, `permission_denied`, `unsupported` and `collection_error` are evidence limitations. They are never silently converted to a failed component or a zero condition score.

## 7. Evidence classes and trust levels

Every evidence item must declare one evidence class:

- `automated_bound`: collected by the signed CYVRA core and bound to the assessment;
- `guided_interaction`: produced by an approved in-application, non-destructive customer interaction;
- `operator_observed`: recorded by an authorized CYVORIQ or enterprise reviewer;
- `customer_media`: an image or video supplied by the customer;
- `customer_attested`: a customer answer without independently verified supporting evidence; or
- `derived`: calculated from approved source evidence using a versioned rule.

Evidence trust is ordered for eligibility purposes:

1. assessment-bound automated evidence;
2. assessment-bound guided interaction;
3. authorized operator observation;
4. reviewed customer media; and
5. uncorroborated customer attestation.

Customer attestation alone cannot satisfy a mandatory functional check or support an issued Grade A or Grade B.

Customer media is not treated as verified merely because upload succeeded. It becomes grade-eligible only after the required quality, integrity and review controls pass.

## 8. Mandatory issuance gates

No grade may be issued unless all of the following are true:

- customer authorization and scan consent are recorded;
- entitlement and device binding are valid where required;
- a supported physical-device profile is selected;
- device identity is sufficiently consistent across the approved sources;
- the evidence manifest belongs to the same assessment and bound device;
- all profile-critical checks have eligible evidence;
- cosmetic evidence meets the profile's required view coverage;
- no unresolved evidence-integrity conflict exists;
- no unresolved safety concern requires review;
- applicable evidence-weight coverage is at least 90 percent;
- the grading engine and rule profile versions are approved;
- any required operator review is complete; and
- the final grade transaction is idempotently accepted by the server.

Grade A additionally requires:

- at least 95 percent applicable evidence-weight coverage;
- no unresolved limitation in a grade-bearing dimension;
- no grade cap; and
- no mandatory outcome supported solely by customer attestation.

Passing the issuance gates does not guarantee a high grade. It only means sufficient evidence exists to calculate a defensible grade.

## 9. Grade dimensions and base weights

The V1 condition dimensions and base weights are:

| Dimension | Base weight | Purpose |
| --- | ---: | --- |
| Core system operation | 30 | Booted assessment context, system stability signals and critical platform operation |
| Integrated display and input | 20 | Built-in display, keyboard, pointing or touch behavior where applicable |
| Storage condition | 10 | Read-only Windows-reported storage state and approved non-destructive evidence |
| Battery and power | 15 | Battery capacity condition, charge state and power behavior where applicable |
| Connectivity, ports and audio | 10 | Approved evidence for network reachability, relevant connectors and audio output |
| Cosmetic and structural condition | 15 | Visible wear, damage, enclosure condition and structural integrity |

Base weights total 100.

A dimension may be removed only when the approved profile marks it `not_applicable`. Its weight is then excluded and the remaining applicable weights are normalized to 100.

Missing, permission-denied, unsupported or error evidence is not `not_applicable`. It does not trigger automatic reweighting.

CPU model, memory capacity, storage capacity, brand, device age and original sale price are specification facts. They do not add or subtract condition points.

## 10. Dimension outcome scale

Each versioned grading check maps eligible evidence to one of these condition outcomes:

| Outcome | Basis points | Meaning |
| --- | ---: | --- |
| `confirmed_good` | 10000 | No material limitation identified by the approved check |
| `minor_limitation` | 8000 | Minor condition issue that does not materially prevent normal use |
| `degraded` | 6000 | Material degradation or defect with continued partial or normal use |
| `major_limitation` | 3000 | Serious limitation that materially affects normal use |
| `failed` | 0 | Approved evidence confirms the tested function failed |
| `unknown` | none | Evidence cannot support a condition outcome |
| `not_applicable` | none | The approved device profile excludes the check |

`unknown` has no numeric value. It affects evidence coverage and may block issuance; it is never scored as zero.

Individual check weights inside a dimension must be stored in the versioned grading profile, reviewed as code and covered by boundary tests. They must not be hidden in GUI components.

## 11. Grade calculation

The grading engine must use deterministic integer arithmetic.

For each applicable dimension:

1. Verify mandatory check coverage.
2. Calculate the weighted mean of eligible check outcomes in basis points.
3. Record the dimension score, limitations and evidence references.

Calculate the overall condition score as:

`sum(applicable dimension score × base weight) / sum(applicable base weights)`

The engine must then apply all grade caps and safety rules before mapping the final score to a grade.

The same evidence manifest, profile version and engine version must always produce the same calculated result.

## 12. Score-to-grade thresholds

After caps are applied, the proposed V1 thresholds are:

| Grade | Final basis-point range | Equivalent condition range |
| --- | ---: | ---: |
| A | 9000–10000 | 90–100 |
| B | 7500–8999 | 75–89.99 |
| C | 6000–7499 | 60–74.99 |
| D | 4000–5999 | 40–59.99 |
| E | 0–3999 | 0–39.99 |

Threshold boundary behavior must be exact and covered by automated tests.

The internal score may be retained for audit and recalculation. The customer-facing application should emphasize the grade, descriptor, category evidence and limitations rather than false numeric precision.

## 13. Grade caps and critical rules

A calculated score does not override a material defect.

The following proposed caps apply:

| Confirmed condition | Maximum grade or action |
| --- | --- |
| Any unresolved safety concern, suspected battery swelling or exposed electrical hazard | Do not issue; require manual review |
| Core system operation has a confirmed critical failure | Grade E |
| Confirmed storage critical-health warning or unreadable required system storage | Grade C |
| Confirmed failure of an integrated display or primary input required for normal use | Grade C |
| Confirmed major structural damage affecting normal use | Grade D |
| Battery derived health ratio below 60 percent | Grade B |
| Battery derived health ratio below 40 percent | Grade C |
| At least 90 percent coverage but any non-critical grade-bearing check or dimension remains unknown | Grade B |
| Unreviewed customer media is required for cosmetic scoring | Do not issue; require review |
| Identity or evidence-manifest conflict | Do not issue |

Caps are applied after score calculation, and every applied cap must appear in the grade record and customer limitations summary.

An operator cannot waive a safety gate or mandatory evidence gate merely to raise a grade.

## 14. Battery-condition mapping

When reliable designed-capacity and full-charge-capacity values are available, CYVRA may derive the battery health ratio defined by W1.1.

The proposed battery-capacity check maps the ratio as follows:

| Full-charge versus designed capacity | Outcome |
| --- | --- |
| 90 percent or higher | `confirmed_good` |
| 80–89.99 percent | `minor_limitation` |
| 70–79.99 percent | `degraded` |
| 60–69.99 percent | `degraded` with explicit limitation |
| 40–59.99 percent | `major_limitation` |
| Below 40 percent | `failed` for the capacity check |

This is an estimate based on firmware or driver-reported capacity. It is not a runtime test, warranty or battery-safety certification.

Missing designed capacity or full-charge capacity produces `unknown`, not zero. Battery cycle count may support context but does not override the capacity evidence.

## 15. Safe functional-evidence boundary

W2.1 deferred functional testing to this grading contract. This section permits only bounded, non-destructive grading checks after their implementation receives separate security and UX approval.

Permitted evidence patterns include:

- successful execution of the approved assessment as one limited core-operation signal;
- read-only Windows operational and device status;
- a guided display pattern with explicit customer confirmation;
- keyboard, touchpad, mouse or touch interaction contained inside the CYVRA test surface;
- a CYVRA-generated speaker tone followed by customer confirmation;
- service reachability already required for the authorized application flow;
- passive charging and power-state observations;
- a user-guided connector check using a known device class without reading its contents; and
- authorized operator observation recorded against the assessment.

The following remain prohibited:

- CPU, GPU, memory, battery or storage stress tests;
- storage write tests, SMART self-tests, sanitize or erase commands;
- arbitrary command execution;
- camera capture;
- microphone recording;
- biometric collection;
- private sensor measurements;
- location collection;
- screenshots or personal-content capture;
- reading files from an attached test device; and
- any test that can materially change customer data or device configuration.

Successful application launch or component detection alone is insufficient to mark all functions `confirmed_good`.

## 16. Passive automated evidence

Automated grading inputs must come from typed, versioned CYVRA core results rather than GUI-parsed text.

Each automated item must retain:

- evidence identifier;
- assessment identifier;
- source collector and parser version;
- collection timestamp;
- Hardware Inventory V1 status;
- confidence;
- permission state;
- raw-value privacy classification;
- normalized value used by the grading rule; and
- evidence hash.

The grading engine may consume only fields approved for that grading-profile version.

A new collector field does not automatically become a grade input. Its meaning, reliability, privacy treatment and test coverage must be reviewed first.

## 17. Cosmetic and structural evidence

Cosmetic grading must use a guided capture checklist appropriate to the device profile.

Minimum proposed views are:

### Laptop, tablet and all-in-one

- front or powered display view;
- input surface or touch surface;
- rear or outer-lid view;
- bottom or stand view;
- left-side view; and
- right-side view.

### Desktop

- front view;
- rear and connector view;
- left-side view;
- right-side view; and
- top or other profile-required enclosure view.

Close-up evidence is required for any declared crack, dent, missing part, corrosion, hinge damage, screen defect or structural concern.

Internal chassis images are optional and must not be requested unless the device is safely powered down and the customer is explicitly instructed not to open equipment they are not authorized or competent to service.

Cosmetic outcome guidance is:

| Observation | Default outcome |
| --- | --- |
| No material visible defect and negligible wear | `confirmed_good` |
| Light scratches or ordinary minor wear | `minor_limitation` |
| Visible moderate wear, dents or coating damage without structural effect | `degraded` |
| Heavy wear, broken trim, hinge or enclosure limitation affecting use | `major_limitation` |
| Severe structural damage confirmed by safe evidence | `failed` or safety review |

Lighting, framing, resolution and obstruction quality must be validated. Poor-quality media is insufficient evidence, not proof of good cosmetic condition.

## 18. Customer-uploaded images and videos

Customer media requires separate, explicit consent and a clear explanation of purpose.

Before upload, the application must:

- show the required device-only capture guidance;
- warn the customer to remove faces, documents, labels unrelated to the device and private background content;
- provide a local preview and removal option;
- strip location and unnecessary EXIF metadata;
- validate type, size and count using an allowlist;
- calculate a content hash; and
- bind accepted media to the assessment and evidence manifest.

The service must:

- store media privately with least-privilege access;
- scan uploads using approved content and malware controls;
- never expose raw object-store URLs publicly;
- record access and review events;
- apply the approved retention and deletion policy; and
- exclude rejected or deleted media from future grade calculations.

Video is optional supporting evidence in V1. It does not replace required still views unless a later profile revision explicitly permits extracted, integrity-bound frames.

The application must not infer identity, emotion, ethnicity, age or other personal characteristics from media.

## 19. Automated media analysis

An image or video model may assist cosmetic review only after a separate model contract defines:

- approved model and version;
- training and evaluation provenance;
- representative device and damage test sets;
- per-class error thresholds;
- confidence calibration;
- human-review triggers;
- adversarial and manipulated-media handling;
- privacy and retention controls;
- monitoring and rollback; and
- customer correction and appeal treatment.

Until that contract is approved and its evaluations pass, customer media affecting a grade requires authorized human review.

An automated model must never directly issue or override the final commercial grade. It produces evidence annotations consumed by the versioned deterministic grading rules.

## 20. Operator review and manual decisions

An authorized reviewer may:

- accept or reject evidence quality;
- classify an observed defect using the approved checklist;
- resolve a documented evidence conflict;
- request additional evidence; and
- approve a recalculation after corrected evidence.

A reviewer must not directly type an arbitrary grade as a substitute for the grading engine.

Every review decision must record:

- pseudonymous reviewer identity or approved enterprise role;
- timestamp;
- action;
- reason code;
- evidence references;
- before and after classification where applicable; and
- resulting recalculation identifier.

Exceptional administrative correction requires a written reason and second authorized approval. It cannot bypass missing mandatory evidence, safety rules or integrity failures.

Reviewer access must be separated from customer, support and release-administration roles.

## 21. Missing, restricted and conflicting evidence

The grading engine must preserve the difference between:

- not applicable;
- not reported;
- permission denied;
- unsupported;
- collection error;
- customer declined;
- evidence rejected for quality; and
- conflicting evidence.

Required treatment:

| Evidence state | Grading treatment |
| --- | --- |
| `not_applicable` under the approved profile | Remove the dimension or check weight as defined by the profile |
| `not_reported` or `unknown` | Reduce coverage; request alternate approved evidence where possible |
| `permission_denied` | Explain the limitation; do not treat as component failure |
| `unsupported` | Explain profile or platform limitation |
| `collection_error` | Isolate error, permit bounded retry and preserve the error code |
| Customer declined optional evidence | Continue if issuance gates remain satisfied |
| Customer declined mandatory evidence | `Unable to grade — insufficient evidence` |
| Evidence conflict | Require resolution or return insufficient evidence |

The GUI must name the missing or limited category without exposing private raw values.

## 22. Client, server and grading authority

The Windows application collects approved evidence and may display a clearly labelled local preview.

The final commercial grade is server-authoritative. It may be issued only after the service verifies:

- entitlement and device binding;
- assessment identity;
- evidence-manifest integrity;
- rule-profile approval status;
- required review completion;
- idempotency and replay controls; and
- grade calculation consistency.

The customer client must not be able to submit a letter grade as authoritative input.

The server may accept typed evidence and a calculated preview, but it must independently verify the rules version and result before issuance.

Offline grading and later synchronization are not authorized by this contract.

## 23. Versioned grade-record contract

The schema name is `cyvra_qc_grade_v1`.

An issued grade record must contain at least:

- grade-record identifier;
- assessment identifier;
- pseudonymous bound-device identifier;
- supported device-profile identifier and version;
- grading-rules identifier and version;
- grading-engine version;
- rule-set hash;
- evidence-manifest hash;
- grade lifecycle status;
- issued grade and descriptor where applicable;
- internal score in integer basis points;
- evidence-coverage basis points;
- dimension scores and outcome summaries;
- applied caps and reason codes;
- material limitations;
- required-review status and review references;
- calculation timestamp;
- issuance timestamp;
- superseded-record reference where applicable; and
- report-authentication linkage when available.

The grade record must not contain raw activation keys, device tokens, passwords, recovery material or unnecessary raw hardware identifiers.

Exact serials or UUIDs must follow the separate approved identifier, encryption, masking and retention contract.

## 24. Evidence integrity and provenance

The evidence manifest must use deterministic canonical serialization and cryptographic hashes.

Each grade must be traceable to:

- the exact evidence items used;
- excluded evidence and exclusion reasons;
- collector and parser versions;
- guided-test versions;
- reviewer decisions;
- grading profile and rule-set hash;
- grade engine version; and
- all recalculation events.

Changing evidence after issuance must never mutate the original grade record. It creates a new evidence manifest and grade revision.

The GUI and report must not call a grade authenticated until the authenticated-report contract's verification has succeeded.

## 25. Grade recalculation, expiry and versioning

Grades are point-in-time assessments and may change as device condition or evidence changes.

A recalculation is required when:

- approved evidence is added, corrected or rejected;
- a conflict is resolved;
- the customer completes a missing mandatory check;
- an operator decision changes an evidence classification;
- the grading profile changes; or
- an issued result is corrected through the dispute process.

Recalculation must:

- preserve the original grade record;
- create a new immutable revision;
- record the trigger and actor;
- identify the previous revision;
- show whether rules or evidence changed; and
- mark the older revision `superseded` without rewriting it.

A new rule version must not silently recalculate historical grades. Regrading requires an explicit transaction and must identify the new rules version.

Exact commercial grade-validity periods remain a later owner and report-retention decision. Until frozen, the GUI must show the assessment date and avoid claiming a grade remains current indefinitely.

## 26. Customer GUI presentation

The QC results screen must present:

- grade lifecycle state;
- issued grade and descriptor when available;
- assessment date;
- device profile;
- evidence-coverage summary;
- dimension-level condition summaries;
- material limitations and applied grade caps;
- missing or restricted evidence;
- evidence and rules versions; and
- a clear path to report preview, retry, additional evidence or support.

The grade must not be shown as a decorative badge without its evidence summary and limitations.

Required notices include:

`CYVRA QC Grade describes assessed device condition at the recorded assessment time. It is not a resale valuation or warranty.`

Where no grade is issued, the interface must explain the specific missing category or safe recovery action.

## 27. Combined report requirements

The combined CYVRA report must include:

- the issued CYVRA QC grade or explicit unable-to-grade status;
- grade descriptor;
- device profile and grading-rules version;
- assessment and issuance times;
- evidence-coverage summary;
- dimension outcomes;
- material limitations and caps;
- evidence-manifest hash;
- grade-record identifier and revision;
- CYVRA Erase results under the separate privacy contract; and
- authenticity and customer-verification information when implemented.

The report must keep condition, specification, privacy exposure and erasure status as separate sections.

An auction, marketplace or resale service may consume an authenticated grade only through a separately approved contract. It must not treat the grade as a guaranteed price.

## 28. Privacy and data minimization

Grading may collect only evidence necessary for the declared condition-assessment purpose.

It must not collect:

- personal file contents or filenames;
- email or message contents;
- browser-history contents;
- passwords, tokens or recovery keys;
- camera frames or microphone audio from the assessed device;
- biometric data;
- precise location;
- unrelated faces, documents or background media; or
- raw identifiers in logs.

Customer-media purpose, access roles, encryption, retention, deletion and support access must be frozen before production use.

The customer must be able to preview media before upload and understand whether declining it will prevent grading.

## 29. Error, retry and idempotency

Grading errors must be bounded and privacy-safe.

| Condition | Required behavior |
| --- | --- |
| Collector timeout | Preserve explicit limitation; retry only the affected bounded operation |
| Media upload interruption | Resume or restart idempotently without duplicate evidence records |
| Review service unavailable | Keep `Grade pending — review required`; do not guess |
| Grade service unavailable | Preserve evidence and offer safe retry |
| Rule-version mismatch | Reject issuance and refresh approved metadata |
| Evidence-hash mismatch | Reject issuance and raise an integrity-safe support code |
| Duplicate issuance request | Return the existing idempotent result |
| Customer cancellation | Stop safely and issue no completed grade |
| Unexpected grading error | Show `Unable to grade — grading error` and a privacy-safe recovery action |

Retry must not duplicate media, reviews, calculations, grade records, audit events or reports.

## 30. Threat and abuse controls

The implementation must address at least:

- forged or replayed evidence;
- evidence from a different device or assessment;
- manipulated images or videos;
- customer modification of local grade data;
- downgraded or unapproved rule profiles;
- reviewer account misuse;
- arbitrary grade overrides;
- object-store URL leakage;
- malicious upload content;
- duplicate issuance and race conditions;
- raw identifier leakage; and
- tampering with a report after issuance.

No secret grading or signing key may be embedded in the customer application.

Security controls must fail closed for final issuance while preserving a clear, non-destructive customer recovery path.

## 31. Audit requirements

The audit trail must record:

- assessment creation and device-binding reference;
- consent version;
- evidence creation, upload, rejection and deletion events;
- operator access and decisions;
- grading profile and rule-set selection;
- calculation and issuance events;
- caps and reason codes;
- recalculation and supersession;
- dispute and correction events;
- report generation and verification; and
- privileged support actions.

Audit access must be least-privilege, logged and separated by role.

Audit records must use pseudonymous identifiers where possible. Raw serials, activation keys and private media must not be copied into audit descriptions.

## 32. Dispute and correction handling

The customer must have an approved path to question a grade or identify incorrect evidence.

The process must:

1. accept a grade-record identifier rather than exposing raw device identifiers;
2. record the disputed dimension or evidence item;
3. preserve the original issued record;
4. allow approved additional or corrected evidence;
5. require authorized review where applicable;
6. run the deterministic engine again;
7. issue a new revision if the result changes; and
8. show the customer which evidence or rule change caused the revision.

Support must not edit an issued grade in place.

Dispute service levels, retention periods and any commercial remedy require separate owner and legal approval.

## 33. Implementation acceptance criteria

Before any customer-facing grade is enabled, tests must prove:

- exact grade-threshold boundaries;
- deterministic results across supported architectures;
- `unknown` never becomes zero;
- `not_applicable` reweighting follows the selected profile;
- missing mandatory evidence blocks issuance;
- every cap applies at the correct boundary;
- Grade A coverage requirements are enforced;
- customer attestation alone cannot satisfy mandatory functional evidence;
- evidence from another assessment or device is rejected;
- duplicate issuance is idempotent;
- altered evidence or rule hashes are rejected;
- grade revisions are immutable and correctly superseded;
- operator decisions are authorized and audited;
- media metadata is stripped as required;
- private media and raw identifiers do not appear in logs;
- cancellation produces no completed grade;
- the GUI shows all pending, issued and unable-to-grade states accessibly; and
- the report preserves grade, evidence, limitations, rules version and authenticity state.

Required validation includes:

- unit tests for every scoring and cap boundary;
- property tests for score range, determinism and missing-data invariants;
- approved golden evidence fixtures for Grades A through E and every unable-to-grade state;
- Windows 10 and Windows 11 physical-device tests across supported profiles;
- malformed firmware and collector-error tests;
- media quality, privacy and malicious-upload tests;
- reviewer calibration and consistency testing;
- accessibility tests for grade and evidence presentation; and
- authenticated-report tamper tests when the report contract is implemented.

## 34. Customer-release gates

CYVRA QC grading is not customer-ready until:

- this W2.2 contract is approved and merged;
- the owner approves the A–E scale, descriptors, weights, thresholds and caps;
- all grade-bearing collector slices are implemented and validated;
- permitted guided functional checks receive security and UX approval;
- customer-media privacy, storage, retention and deletion contracts are approved;
- an initial operator-review procedure and reviewer handbook are approved;
- the typed grade profile and `cyvra_qc_grade_v1` schemas are frozen;
- Agent-to-Worker authentication, replay and idempotency contracts are implemented;
- device binding and entitlement are enforced;
- the deterministic grading engine passes the acceptance suite;
- the final QC results design is owner-approved;
- the authenticated combined report passes verification and tamper tests;
- an internal pilot validates grading consistency; and
- a separate production go-live decision records exact commits, versions and evidence.

The current passive hardware validator and engineering CLI are not commercial grading products.

## 35. Explicit non-goals

This contract does not authorize:

- destructive erasure;
- password or security-control bypass;
- hardware stress testing;
- camera or microphone activation;
- secret or personal-content collection;
- Windows Server grading;
- virtual-machine physical grading;
- offline grading;
- automatic resale valuation;
- an auction or bid decision;
- an AI-only final grade;
- arbitrary human grade overrides;
- public executable distribution;
- unsigned customer installation;
- production infrastructure changes; or
- customer go-live.

## 36. Owner decisions proposed for approval

The owner must explicitly approve or amend:

1. Five grades: A, B, C, D and E.
2. The customer descriptors: Excellent, Good, Fair, Limited and Major limitations.
3. The six dimension weights in Section 9.
4. The 90 percent minimum issuance coverage and 95 percent Grade A coverage.
5. The score thresholds in Section 12.
6. The caps in Section 13.
7. The battery mapping in Section 14.
8. The bounded functional-evidence permissions in Section 15.
9. Required cosmetic views in Section 17.
10. Human review for grade-bearing customer media until a separate automated-model contract is approved.
11. Server-authoritative final issuance.
12. The rule that grade is condition, not specification, warranty, valuation or auction price.

No grading implementation may silently resolve an unapproved item.

## 37. Change control and handoff

Every grading implementation handoff must state:

- branch and base commit;
- approved grading requirement addressed;
- grade-profile or schema version changed;
- evidence sources added or changed;
- privacy and security impact;
- API, database, object-store and report impact;
- tests and physical-device evidence;
- reviewer or media workflow changes;
- deployment performed or explicitly not performed;
- rollback method;
- open risks; and
- next approval gate.

Any change to grade labels, weights, thresholds, caps, mandatory evidence or issuance authority requires a versioned contract amendment and regression evidence.

## 38. Immediate next action after approval

After this contract is reviewed and merged:

1. Produce the W2.1A wireframes and high-fidelity QC result states using the approved scale.
2. Define typed `cyvra_qc_grading_profile_v1`, `cyvra_qc_evidence_manifest_v1` and `cyvra_qc_grade_v1` schemas.
3. Create approved golden fixtures for every grade and unable-to-grade state.
4. Design the bounded guided functional checks without implementing prohibited tests.
5. Define the customer-media privacy, storage, retention and reviewer workflow.
6. Continue the Agent-to-Worker, device-binding and authenticated-report contracts required for final issuance.
7. Implement the grading engine only in a dedicated reviewed branch after the relevant contracts are approved.

No production deployment, customer installer, public grade or customer release is authorized by this document.
