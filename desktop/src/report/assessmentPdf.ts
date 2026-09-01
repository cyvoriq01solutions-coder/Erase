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

type PdfItem =
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

function addSection(items: PdfItem[], heading: string, rows: NamedValue[], empty: string): void {
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
  const items: PdfItem[] = [
    { kind: "text", style: "kicker", text: "CYVORIQ SOLUTIONS PVT. LTD." },
    { kind: "text", style: "kicker", text: "CYVRA ERASE  ·  REPORT A  ·  PRE-SANITIZATION ASSET ASSESSMENT" },
    { kind: "text", style: "title", text: "Serialized local assessment" },
    {
      kind: "text",
      style: "meta",
      text: `Document no. ${reportId}    Generated ${generatedLabel}    Host ${verification.hostname}`,
    },
    {
      kind: "text",
      style: "meta",
      text: "Classification: operator copy    Data erased: No    Cloud authentication: not enabled in this version",
    },
    { kind: "gap", size: 8 },
    {
      kind: "text",
      style: "notice",
      text: "This is a computer-generated report produced on the assessed Windows PC by CYVRA Erase, software of CYVORIQ Solutions Pvt. Ltd. It is issued as a serialized intake / pre-sanitization assessment (Report A). It is not a Certificate of Sanitization or Destruction, not NIST SP 800-88 Purge proof, and not a DPDP compliance certificate. File contents were not opened. No drive was erased. Device condition grading, cosmetic rating, and physical port confirmation are possible only after physical verification by a technician.",
    },
  ];

  addSection(
    items,
    "1. Device identity",
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
      { label: "Drives included in this scan", value: verification.scannedDrives },
      {
        label: "Hardware result",
        value:
          verification.hardwareResult === "pass"
            ? "Passed"
            : verification.hardwareResult === "fail"
              ? "Needs review"
              : "Not available on this PC",
      },
    ],
    "Identity was not available.",
  );

  addSection(
    items,
    "2. Hardware recorded in this scan",
    hardwareScanRows(verification.hardwareFields),
    "Hardware details were not available on this PC.",
  );

  addSection(
    items,
    "3. Observations pending collector update or physical inspection",
    peripheralHealthRows(verification),
    NOT_COLLECTED,
  );

  addSection(
    items,
    "4. Privacy exposure map (names and sizes only)",
    [
      { label: "Document locations", value: String(verification.personalLocationCount) },
      { label: "Mapped objects", value: String(verification.pdemObjectCount) },
      { label: "File contents opened", value: verification.contentInspected ? "Yes" : "No" },
      { label: "Data erased", value: "No" },
      ...verification.locationGroups,
    ],
    "No document categories were recorded on the selected drives.",
  );

  items.push({ kind: "gap", size: 10 });
  items.push({ kind: "text", style: "heading", text: "5. Method, limitations and issuing body" });
  items.push({ kind: "rule" });
  items.push({
    kind: "text",
    style: "body",
    text: verification.assessmentSummary || "Local assessment completed. No data was erased.",
  });
  items.push({
    kind: "text",
    style: "body",
    text: "Method: read-only Windows firmware, CIM and PnP inventory plus document-location metadata (names and sizes). Battery health is remaining full-charge capacity versus design capacity when both values are reported and greater than zero. Camera and microphone counts are Windows PnP enumerations. USB, HDMI, DisplayPort and jack counts are printed only from the firmware connector table, never guessed as zero. All-zero disk serials from Windows are treated as not reported.",
  });
  items.push({
    kind: "text",
    style: "body",
    text: "Keep this PDF off disks you may later erase. After a full-PC purge this application will not run on that computer.",
  });
  items.push({ kind: "gap", size: 8 });
  items.push({ kind: "text", style: "heading", text: "6. Issuing organisation" });
  items.push({ kind: "rule" });
  items.push({
    kind: "text",
    style: "body",
    text: "Issued by CYVORIQ Solutions Pvt. Ltd. as publisher of CYVRA Erase. This document is computer-generated on the assessed PC. It is not cloud-authenticated in this version. It does not certify sanitization, destruction, resale grade, or legal compliance. Device rating is possible only after physical verification.",
  });
  items.push({
    kind: "text",
    style: "body",
    text: "Operator / technician (physical verification): ________________________    Date: __________",
  });

  return items;
}

function emitText(x: number, y: number, font: "F1" | "F2", size: number, text: string): string {
  return `BT /${font} ${size} Tf 1 0 0 1 ${x.toFixed(2)} ${y.toFixed(2)} Tm (${pdfEscape(text)}) Tj ET`;
}

function paginate(items: PdfItem[]): string[] {
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

function assemblePdf(pageStreams: string[], footer: string): Uint8Array {
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

export function saveAssessmentPdf(verification: VerificationRecord): { filename: string; reportId: string } {
  const generatedAt = new Date();
  const reportId = makeReportId(verification, generatedAt);
  const filename = `CYVRA-Erase-assessment-${reportId}.pdf`;
  const bytes = buildAssessmentPdf(verification, generatedAt);
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
  return { filename, reportId };
}
