import type { NamedValue, VerificationRecord } from "../types/shell";

const PAGE_WIDTH = 595;
const PAGE_HEIGHT = 842;
const MARGIN_X = 48;
const MARGIN_TOP = 52;
const MARGIN_BOTTOM = 56;
const CONTENT_WIDTH = PAGE_WIDTH - MARGIN_X * 2;

export const NOT_COLLECTED =
  "Not collected in this scan. Deferred in this collector version; Windows can report this later. Physical verification is required for grading.";

const IDENTITY_LABELS = new Set([
  "computer name",
  "operating system",
  "manufacturer",
  "model",
  "bios / oem serial",
  "chassis serial",
  "motherboard serial",
  "smbios uuid",
  "asset tag",
]);

export function isUnusableSerial(value: string): boolean {
  const compact = value.replace(/[^A-Za-z0-9]/g, "");
  if (!compact) {
    return true;
  }
  if (/^0+$/.test(compact)) {
    return true;
  }
  const lower = compact.toLowerCase();
  return (
    lower === "unknown" ||
    lower === "tobefilledbyoem" ||
    lower === "defaultstring" ||
    lower === "systemserialnumber" ||
    lower === "none" ||
    lower === "na"
  );
}

export function displaySerial(value: string | null | undefined): string {
  if (!value || isUnusableSerial(value)) {
    return "Not reported by firmware";
  }
  return value.trim();
}

export function hardwareScanRows(fields: NamedValue[]): NamedValue[] {
  return fields.filter((row) => {
    if (IDENTITY_LABELS.has(row.label.toLowerCase())) {
      return false;
    }
    if (/serial/i.test(row.label) && isUnusableSerial(row.value)) {
      return false;
    }
    return true;
  });
}

export function makeReportId(verification: VerificationRecord, generatedAt: Date): string {
  const day = generatedAt.toISOString().slice(0, 10).replace(/-/g, "");
  const host = verification.hostname.replace(/[^A-Za-z0-9]/g, "").slice(0, 18) || "PC";
  return `ERA-${day}-${host}`;
}

export function lookupField(fields: NamedValue[], needles: string[]): string | null {
  const lowered = needles.map((needle) => needle.toLowerCase());
  for (const row of fields) {
    const label = row.label.toLowerCase();
    if (lowered.some((needle) => label.includes(needle))) {
      const value = row.value.trim();
      if (value && !isUnusableSerial(value) && value !== "Not reported by firmware") {
        return value;
      }
    }
  }
  return null;
}

export function peripheralHealthRows(verification: VerificationRecord): NamedValue[] {
  const fields = verification.hardwareFields;
  const row = (label: string, needles: string[], collectedHint: string): NamedValue => {
    const found = lookupField(fields, needles);
    if (found) {
      return { label, value: found };
    }
    return { label, value: collectedHint };
  };

  return [
    row("Battery health %", ["battery health"], NOT_COLLECTED),
    row("Battery present", ["battery present", "battery status"], NOT_COLLECTED),
    row("Cameras", ["camera"], NOT_COLLECTED),
    row("Microphones", ["microphone", "audio endpoint"], NOT_COLLECTED),
    row("USB ports", ["usb port", "usb-a", "usb-c", "usb type"], NOT_COLLECTED),
    row("HDMI ports", ["hdmi"], NOT_COLLECTED),
    row("DisplayPort", ["displayport", "display port"], NOT_COLLECTED),
    row("Ethernet / audio jacks", ["ethernet", "rj45", "audio jack", "headphone"], NOT_COLLECTED),
  ];
}

function toWinAnsi(text: string): string {
  return text
    .normalize("NFKD")
    .replace(/\r\n/g, "\n")
    .replace(/\r/g, "\n")
    .replace(/[^\u0020-\u007E\n]/g, (character) => {
      const mapped: Record<string, string> = {
        "\u2014": "-",
        "\u2013": "-",
        "\u2018": "'",
        "\u2019": "'",
        "\u201C": '"',
        "\u201D": '"',
        "\u2022": "-",
        "\u00A0": " ",
        "\u2026": "...",
      };
      return mapped[character] ?? "?";
    });
}

function pdfEscape(text: string): string {
  return toWinAnsi(text).replace(/\\/g, "\\\\").replace(/\(/g, "\\(").replace(/\)/g, "\\)");
}

function wrapLine(text: string, maxChars: number): string[] {
  const cleaned = toWinAnsi(text).replace(/\n/g, " ").replace(/\s+/g, " ").trim();
  if (!cleaned) {
    return [""];
  }
  const words = cleaned.split(" ");
  const lines: string[] = [];
  let current = "";
  for (const word of words) {
    const candidate = current ? `${current} ${word}` : word;
    if (candidate.length <= maxChars) {
      current = candidate;
      continue;
    }
    if (current) {
      lines.push(current);
    }
    if (word.length > maxChars) {
      for (let index = 0; index < word.length; index += maxChars) {
        const slice = word.slice(index, index + maxChars);
        if (slice.length === maxChars) {
          lines.push(slice);
        } else {
          current = slice;
        }
      }
    } else {
      current = word;
    }
  }
  if (current) {
    lines.push(current);
  }
  return lines;
}

type PdfStyle = "title" | "kicker" | "heading" | "body" | "meta" | "label" | "value" | "notice" | "footer";

export type PdfItem =
  | { kind: "gap"; size: number }
  | { kind: "rule" }
  | { kind: "text"; style: PdfStyle; text: string; maxChars?: number };

function styleSpec(style: PdfStyle): { font: "F1" | "F2"; size: number; leading: number; maxChars: number } {
  switch (style) {
    case "kicker":
      return { font: "F2", size: 8, leading: 11, maxChars: 92 };
    case "title":
      return { font: "F2", size: 16, leading: 20, maxChars: 48 };
    case "heading":
      return { font: "F2", size: 11, leading: 16, maxChars: 72 };
    case "meta":
      return { font: "F1", size: 9, leading: 12, maxChars: 92 };
    case "notice":
      return { font: "F1", size: 8.5, leading: 11, maxChars: 96 };
    case "label":
      return { font: "F2", size: 9, leading: 12, maxChars: 40 };
    case "value":
      return { font: "F1", size: 9, leading: 12, maxChars: 62 };
    case "footer":
      return { font: "F1", size: 8, leading: 10, maxChars: 96 };
    default:
      return { font: "F1", size: 9, leading: 12, maxChars: 92 };
  }
}

export function exposureSummaryRows(verification: VerificationRecord): NamedValue[] {
  return [
    { label: "Document locations", value: String(verification.personalLocationCount) },
    { label: "Mapped objects", value: String(verification.pdemObjectCount) },
    { label: "File contents opened", value: verification.contentInspected ? "Yes" : "No" },
    { label: "Data erased", value: "No" },
    ...verification.locationGroups,
  ];
}

export function exposureLocationRows(verification: VerificationRecord): NamedValue[] {
  return verification.exposureMap.map((row) => ({
    label: `${row.category} · ${row.folder}`,
    value: `${row.files === 1 ? "1 file" : `${row.files} files`} · ${row.sizeLabel} · ${row.classification} · contents not opened`,
  }));
}

export function addSection(items: PdfItem[], heading: string, rows: NamedValue[], empty: string): void {
  items.push({ kind: "gap", size: 10 });
  items.push({ kind: "text", style: "heading", text: heading });
  items.push({ kind: "rule" });
  if (rows.length === 0) {
    items.push({ kind: "text", style: "body", text: empty });
    return;
  }
  for (const row of rows) {
    items.push({ kind: "text", style: "label", text: row.label });
    for (const line of wrapLine(row.value, 88)) {
      items.push({ kind: "text", style: "value", text: line });
    }
    items.push({ kind: "gap", size: 3 });
  }
}

export function buildAssessmentDocument(verification: VerificationRecord, generatedAt: Date): PdfItem[] {
  const reportId = makeReportId(verification, generatedAt);
  const generatedLabel = generatedAt.toLocaleString();
  const hardwareResult =
    verification.hardwareResult === "pass"
      ? "Passed"
      : verification.hardwareResult === "fail"
        ? "Needs review"
        : "Not available on this PC";
  const items: PdfItem[] = [
    { kind: "text", style: "kicker", text: "CYVRA ERASE  |  REPORT A  |  INTAKE & PRE-SANITIZATION ASSESSMENT" },
    { kind: "text", style: "kicker", text: "CYVORIQ SOLUTIONS PVT. LTD." },
    { kind: "text", style: "meta", text: "Controlled assessment record  |  Not evidence of sanitization or destruction" },
    { kind: "text", style: "title", text: "Intake & Pre-Sanitization Assessment Record" },
    {
      kind: "text",
      style: "notice",
      text: "DOCUMENT STATUS: PRE-SANITIZATION ASSESSMENT — NO DATA ERASURE PERFORMED",
    },
    {
      kind: "text",
      style: "meta",
      text: `Document no. ${reportId}    Generated ${generatedLabel}    Host ${verification.hostname}`,
    },
    { kind: "gap", size: 8 },
    {
      kind: "text",
      style: "notice",
      text: "This computer-generated record was produced by CYVRA Erase on the assessed Windows PC. It documents the pre-sanitization assessment evidence available during the recorded scan. It is not a Certificate of Sanitization or Destruction, is not evidence of NIST SP 800-88 purge completion, and does not by itself certify compliance with the Digital Personal Data Protection Act, 2023 (DPDP Act), ISO standards, or any other legal or regulatory requirement. No drive was erased and file contents were not opened during this assessment. Values not obtained from the available collection sources are explicitly identified as Not collected or Not reported by source. Such values must not be interpreted as zero, a pass, a failure, or evidence of absence.",
    },
  ];

  addSection(
    items,
    "Executive Assessment Snapshot",
    [
      { label: "Report identifier", value: reportId },
      { label: "Assessed device / host", value: verification.hostname },
      { label: "Manufacturer / model", value: `${verification.manufacturer} / ${verification.model}` },
      { label: "Operating system", value: verification.osCaption },
      { label: "Assessment mode", value: "Serialized local assessment" },
      { label: "Reported drive scope", value: verification.scannedDrives },
      { label: "File contents opened", value: verification.contentInspected ? "Yes" : "No" },
      { label: "Data erased during this assessment", value: "No" },
      { label: "Cloud authentication status", value: "Not enabled in this version" },
      { label: "Hardware result", value: hardwareResult },
    ],
    "No snapshot available.",
  );

  addSection(
    items,
    "1. Document Control & Evidence Status",
    [
      { label: "Report identifier", value: reportId },
      { label: "Report type", value: "Report A — Intake & Pre-Sanitization Assessment Record" },
      { label: "Generated on assessed device", value: `${generatedLabel} local device time` },
      { label: "Assessed host", value: verification.hostname },
      { label: "Issuing software", value: "CYVRA Erase" },
      { label: "Publisher", value: "CYVORIQ Solutions Pvt. Ltd." },
      { label: "Assessment execution mode", value: "Serialized local assessment" },
      { label: "Cloud authentication", value: "Not enabled in this version" },
      {
        label: "Evidence state",
        value:
          "Locally generated computer record; production integrity and external verification controls should be added before representing the report as organization-authenticated evidence.",
      },
      { label: "Document classification", value: "Operator / controlled assessment record" },
    ],
    "Document control was not available.",
  );

  items.push({ kind: "gap", size: 10 });
  items.push({ kind: "text", style: "heading", text: "2. Purpose & Permitted Use" });
  items.push({ kind: "rule" });
  items.push({
    kind: "text",
    style: "body",
    text: "The purpose of Report A is to establish a documented baseline before any data sanitization, device refurbishment, resale, transfer, reuse, return, disposal or other downstream disposition activity. The report is intended to support internal asset handling, evidence review, technician workflows and customer audit trails.",
  });
  items.push({
    kind: "text",
    style: "body",
    text: "This report should be read together with the organization's approved asset-disposition policy, authorization records and, where applicable, a subsequent sanitization or verification record. The report must not be used as standalone proof that personal data, confidential information or any other data has been removed.",
  });

  addSection(
    items,
    "3. Scope, Boundary & Collection Method",
    [
      { label: "Reported drive scope", value: verification.scannedDrives },
      {
        label: "Collection approach",
        value:
          "Read-only Windows firmware, CIM and PnP inventory, together with document-location metadata (names and sizes).",
      },
      { label: "File content inspection", value: verification.contentInspected ? "Performed" : "Not performed" },
      { label: "Data sanitization / purge", value: "Not performed" },
      { label: "Physical inspection", value: "Not completed as part of this automated local assessment" },
      {
        label: "Boundary warning",
        value:
          "This assessment does not establish complete coverage of removable media, inaccessible storage, encrypted volumes, cloud repositories, network locations or data outside the explicitly reported scan scope unless separately recorded.",
      },
    ],
    "Scope was not recorded.",
  );

  addSection(
    items,
    "4. Device Identity & Asset Evidence",
    [
      { label: "Computer name", value: verification.hostname },
      { label: "Manufacturer", value: verification.manufacturer },
      { label: "Model", value: verification.model },
      {
        label: "BIOS / OEM serial",
        value: displaySerial(lookupField(verification.hardwareFields, ["bios / oem serial"])),
      },
      {
        label: "Chassis serial",
        value: displaySerial(lookupField(verification.hardwareFields, ["chassis serial"])),
      },
      {
        label: "Motherboard serial",
        value: displaySerial(lookupField(verification.hardwareFields, ["motherboard serial"])),
      },
      {
        label: "SMBIOS UUID",
        value: displaySerial(lookupField(verification.hardwareFields, ["smbios uuid"])),
      },
      { label: "Operating system", value: verification.osCaption },
      { label: "Hardware result", value: hardwareResult },
    ],
    "Identity was not available.",
  );

  addSection(
    items,
    "5. Hardware Inventory Recorded During This Assessment",
    hardwareScanRows(verification.hardwareFields),
    "Hardware details were not available on this PC.",
  );

  addSection(
    items,
    "6. Deferred, Unknown & Physical-Verification Items",
    peripheralHealthRows(verification),
    NOT_COLLECTED,
  );

  addSection(
    items,
    "7. Metadata-Based Data-Exposure Indicators",
    [...exposureSummaryRows(verification), ...exposureLocationRows(verification)],
    "No document categories were recorded on the selected drives.",
  );

  items.push({ kind: "gap", size: 10 });
  items.push({ kind: "text", style: "heading", text: "8. Privacy, Data-Minimisation & Handling Notes" });
  items.push({ kind: "rule" });
  items.push({
    kind: "text",
    style: "body",
    text: "Report A should be treated as a potentially sensitive asset record because device identifiers, host names and storage-related evidence may be operationally sensitive. Access should therefore be controlled according to the issuing organization's information-security and privacy policies.",
  });
  items.push({
    kind: "text",
    style: "body",
    text: "This assessment alone does not establish whether the device contains Digital Personal Data or other regulated information. Category indicators are metadata only; contents were not opened.",
  });

  addSection(
    items,
    "9. Authorization & Chain-of-Custody Record",
    [
      { label: "Asset owner / custodian", value: "Not recorded in this version" },
      { label: "Assessment authorization reference", value: "Not recorded in this version" },
      { label: "Operator identity", value: "Not recorded in this version" },
      { label: "Collection location", value: "Not recorded in this version" },
      { label: "Custody received timestamp", value: "Not recorded in this version" },
      { label: "Intended disposition", value: "Not recorded in this version" },
    ],
    "Not recorded in this version.",
  );

  addSection(
    items,
    "10. Assessment Findings & Recommended Next Actions",
    [
      {
        label: "Device identity evidence",
        value: "Host, manufacturer, model and serials reported where Windows supplied them. Reconcile against the asset register before downstream disposition.",
      },
      {
        label: "Potential data-bearing locations",
        value: `${verification.personalLocationCount} document locations and ${verification.pdemObjectCount} mapped objects. Contents were not opened. Determine whether a controlled sanitization or preservation workflow is required.`,
      },
      {
        label: "No erasure performed",
        value: "Explicitly reported as No. Do not represent this report as sanitization evidence.",
      },
      {
        label: "Physical verification",
        value: "Automated inventory cannot confirm cosmetic condition or actual connector functionality. Complete a technician checklist if condition or resale grading is required.",
      },
      {
        label: "Local-only issuance",
        value: "Cloud authentication is not enabled in this version. For audit-ready production, add canonical evidence hashing, signing and an independent verification workflow.",
      },
    ],
    "No findings were recorded.",
  );

  items.push({ kind: "gap", size: 10 });
  items.push({ kind: "text", style: "heading", text: "11. Evidence Limitations" });
  items.push({ kind: "rule" });
  items.push({
    kind: "text",
    style: "body",
    text: "This assessment records only the evidence available through the stated collection method and scope. A missing value does not prove that a feature, device, port, battery, data category or risk is absent. Automated enumeration is not a substitute for physical inspection where physical confirmation is required.",
  });
  items.push({
    kind: "text",
    style: "body",
    text: "The report does not prove data deletion, cryptographic erasure, overwrite completion, media sanitization, destruction, resale condition or legal/regulatory compliance. Those conclusions require separate controlled processes.",
  });

  items.push({ kind: "gap", size: 10 });
  items.push({ kind: "text", style: "heading", text: "12. Production Audit-Readiness Controls Recommended" });
  items.push({ kind: "rule" });
  items.push({
    kind: "text",
    style: "body",
    text: "Recommended for a later production issuance: immutable report identity, canonical evidence package, SHA-256 digest, organisation-controlled signature, trusted time, source provenance, authorization binding, retention and change control. These controls are not enabled in this version.",
  });

  items.push({ kind: "gap", size: 8 });
  items.push({ kind: "text", style: "heading", text: "13. Issuing Organisation & Controlled Statement" });
  items.push({ kind: "rule" });
  items.push({
    kind: "text",
    style: "body",
    text: "Issued by CYVORIQ Solutions Pvt. Ltd. as publisher of CYVRA Erase. This document is computer-generated on the assessed PC. It is not cloud-authenticated in this version. Controlled statement: this report establishes a pre-sanitization assessment baseline only. It does not certify sanitization, destruction, resale grade, regulatory compliance or legal compliance.",
  });

  addSection(
    items,
    "14. Verification & Sign-Off Status",
    [
      { label: "Automated assessment", value: "Completed — report generated from recorded evidence" },
      { label: "Physical verification", value: "Not completed in this automated assessment" },
      { label: "Sanitization verification", value: "Not applicable to Report A" },
      { label: "Organization authentication", value: "Not enabled in this version" },
      { label: "Final disposition decision", value: "Not recorded" },
      {
        label: "Physical verification block",
        value:
          "Technician name: recorded at sign-off on the assessed PC. Date of inspection: recorded at sign-off on the assessed PC. Result: see the hardware and location tables above. This block is not a handwritten signature line.",
      },
    ],
    "Sign-off was not recorded.",
  );

  items.push({
    kind: "text",
    style: "body",
    text: "END OF REPORT A. Recommended evidence family: Report A — Intake & Pre-Sanitization Assessment; Report D — Technical Diagnostic & Condition Evidence; Report S — Sanitization & Verification Record (not generated in this version).",
  });

  return items;
}

function emitText(x: number, y: number, font: "F1" | "F2", size: number, text: string): string {
  return `BT /${font} ${size} Tf 1 0 0 1 ${x.toFixed(2)} ${y.toFixed(2)} Tm (${pdfEscape(text)}) Tj ET`;
}

export function paginate(items: PdfItem[]): string[] {
  const pages: string[][] = [];
  let ops: string[] = [];
  let y = PAGE_HEIGHT - MARGIN_TOP;

  const flush = () => {
    pages.push(ops);
    ops = [];
    y = PAGE_HEIGHT - MARGIN_TOP;
  };

  const ensure = (needed: number) => {
    if (y - needed < MARGIN_BOTTOM) {
      flush();
    }
  };

  for (const item of items) {
    if (item.kind === "gap") {
      y -= item.size;
      continue;
    }
    if (item.kind === "rule") {
      ensure(8);
      ops.push("0.024 0.161 0.373 RG 0.6 w");
      ops.push(`${MARGIN_X} ${y.toFixed(2)} m ${MARGIN_X + CONTENT_WIDTH} ${y.toFixed(2)} l S`);
      y -= 8;
      continue;
    }
    const spec = styleSpec(item.style);
    const lines = wrapLine(item.text, item.maxChars ?? spec.maxChars);
    for (const line of lines) {
      ensure(spec.leading);
      const x = item.style === "value" ? MARGIN_X + 14 : MARGIN_X;
      ops.push(emitText(x, y - spec.size, spec.font, spec.size, line));
      y -= spec.leading;
    }
  }

  if (ops.length > 0 || pages.length === 0) {
    pages.push(ops);
  }
  return pages.map((pageOps) => pageOps.join("\n"));
}

export function assemblePdf(pageStreams: string[], footer: string): Uint8Array {
  const objects: string[] = [];
  objects.push("1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n");

  const pageCount = pageStreams.length;
  const fontStart = 3 + pageCount * 2;
  const kids = pageStreams.map((_, index) => `${3 + index * 2} 0 R`).join(" ");
  objects.push(`2 0 obj << /Type /Pages /Kids [ ${kids} ] /Count ${pageCount} >> endobj\n`);

  const pageObjects: string[] = [];
  pageStreams.forEach((stream, index) => {
    const pageObjectNumber = 3 + index * 2;
    const contentObjectNumber = pageObjectNumber + 1;
    const decorated = [
      "0.024 0.161 0.373 rg",
      `${MARGIN_X} ${PAGE_HEIGHT - 28} ${CONTENT_WIDTH} 10 re f`,
      "1 1 1 rg",
      emitText(MARGIN_X + 6, PAGE_HEIGHT - 26, "F2", 7, "CYVORIQ SOLUTIONS PVT. LTD.  ·  COMPUTER-GENERATED  ·  NOT A SANITIZATION CERTIFICATE"),
      "0 0 0 rg",
      stream,
      "0.37 0.44 0.53 rg",
      emitText(MARGIN_X, 28, "F1", 8, `${footer}  ·  page ${index + 1} of ${pageCount}`),
    ].join("\n");
    pageObjects.push(
      `${pageObjectNumber} 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 ${PAGE_WIDTH} ${PAGE_HEIGHT}] /Resources << /Font << /F1 ${fontStart} 0 R /F2 ${fontStart + 1} 0 R >> >> /Contents ${contentObjectNumber} 0 R >> endobj\n`,
    );
    pageObjects.push(
      `${contentObjectNumber} 0 obj << /Length ${decorated.length} >> stream\n${decorated}\nendstream endobj\n`,
    );
  });
  objects.push(...pageObjects);
  objects.push(`${fontStart} 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n`);
  objects.push(
    `${fontStart + 1} 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >> endobj\n`,
  );

  let body = "%PDF-1.4\n";
  const offsets = [0];
  for (const object of objects) {
    offsets.push(body.length);
    body += object;
  }
  const xrefStart = body.length;
  let xref = `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`;
  for (let index = 1; index < offsets.length; index += 1) {
    xref += `${String(offsets[index]).padStart(10, "0")} 00000 n \n`;
  }
  body += xref;
  body += `trailer << /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xrefStart}\n%%EOF\n`;

  const bytes = new Uint8Array(body.length);
  for (let index = 0; index < body.length; index += 1) {
    bytes[index] = body.charCodeAt(index) & 0xff;
  }
  return bytes;
}

export function buildAssessmentPdf(verification: VerificationRecord, generatedAt = new Date()): Uint8Array {
  const items = buildAssessmentDocument(verification, generatedAt);
  const reportId = makeReportId(verification, generatedAt);
  const pages = paginate(items);
  return assemblePdf(pages, `${reportId}  ·  CYVORIQ Solutions Pvt. Ltd.  ·  not a sanitization certificate`);
}

/** Hand a generated PDF to the operator as a download. Shared by every report. */
export function downloadPdf(bytes: Uint8Array, filename: string): void {
  const copy = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(copy).set(bytes);
  const blob = new Blob([copy], { type: "application/pdf" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.rel = "noopener";
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 4000);
}

export function saveAssessmentPdf(verification: VerificationRecord): { filename: string; reportId: string } {
  const generatedAt = new Date();
  const reportId = makeReportId(verification, generatedAt);
  const filename = `CYVRA-Erase-assessment-${reportId}.pdf`;
  downloadPdf(buildAssessmentPdf(verification, generatedAt), filename);
  return { filename, reportId };
}
