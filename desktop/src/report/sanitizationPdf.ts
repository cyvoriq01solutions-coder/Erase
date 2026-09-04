import {
  addSection,
  assemblePdf,
  downloadPdf,
  paginate,
  type AssessmentSection,
  type PdfItem,
} from "./assessmentPdf";
import type { PurgeRecord, VerificationRecord } from "../types/shell";

export function makeSanitizationId(
  verification: VerificationRecord | null,
  generatedAt: Date,
): string {
  const day = generatedAt.toISOString().slice(0, 10).replace(/-/g, "");
  const host = (verification?.hostname ?? "").replace(/[^A-Za-z0-9]/g, "").slice(0, 18) || "PC";
  return `ERS-${day}-${host}`;
}

function identityRows(verification: VerificationRecord | null): { label: string; value: string }[] {
  if (!verification) {
    return [];
  }
  return [
    { label: "Computer name", value: verification.hostname },
    { label: "Manufacturer", value: verification.manufacturer },
    { label: "Model", value: verification.model },
    { label: "Operating system", value: verification.osCaption },
  ];
}

function mediaRows(record: PurgeRecord): { label: string; value: string }[] {
  return record.targets.map((target) => ({
    label: `${target.letter}: ${target.mediaLabel}`,
    value: `${target.methodLabel}. ${target.standard}. Serial ${target.serial}. ${target.verifyNote}`,
  }));
}

export function buildSanitizationSections(
  record: PurgeRecord,
  verification: VerificationRecord | null,
  generatedAt: Date,
): AssessmentSection[] {
  const documentId = makeSanitizationId(verification, generatedAt);
  const status = record.status === "VERIFIED" ? "VERIFIED" : "FAILED";
  return [
    {
      kind: "table",
      title: "1. Document Control",
      empty: "Not recorded.",
      rows: [
        { label: "Document", value: "Report S — Sanitization & Verification Record" },
        { label: "Family", value: "Report S / Report B" },
        { label: "Document identifier", value: documentId },
        { label: "Job identifier", value: record.jobId || "Not issued" },
        { label: "Generated", value: generatedAt.toISOString() },
        { label: "Status", value: status },
      ],
    },
    {
      kind: "prose",
      title: "2. Status",
      paragraphs: [
            status === "VERIFIED"
              ? "Independent verification on this PC passed. Status is VERIFIED. This is not a laboratory certification."
              : "This job is FAILED. No sanitization report is issued on FAIL, cancel, crash, or a missing helper.",
        record.message,
      ],
    },
    {
      kind: "table",
      title: "3. Device Identity",
      empty: "Run Report A first so this PC is identified.",
      rows: identityRows(verification),
    },
    {
      kind: "prose",
      title: "4. Operator consent",
      paragraphs: [
        "Mode S required a bound CYVRA Purge licence, this PC’s name typed to match, and ERASE in capital letters. Those consent tokens are not stored as keystrokes.",
        "The Windows system disk is never included in Mode S while CYVRA Erase is running on it.",
      ],
    },
    {
      kind: "table",
      title: "5. Media and method",
      empty: "No extra disk was recorded.",
      rows: mediaRows(record),
    },
    {
      kind: "prose",
      title: "6. Independent verification",
      paragraphs: record.targets.map(
        (target) =>
          `${target.letter}: helper ${target.helperOk ? "completed" : "did not complete"}. Sample ${target.samplePercent}%. ${target.verifyNote}`,
      ),
    },
    {
      kind: "table",
      title: "7. Evidence",
      empty: "None",
      rows: [
        { label: "Local evidence hash", value: record.evidenceHash },
        {
          label: "Data erased on selected extra disks",
          value: record.dataErased ? "Yes, extra disks selected for Mode S" : "No",
        },
        { label: "Report allowed", value: record.reportAllowed ? "Yes" : "No" },
      ],
    },
    {
      kind: "prose",
      title: "8. Limitations",
      paragraphs: [
        "This computer-generated record was produced by CYVRA Erase on this Windows PC. It is locally verified on this PC. It is not a laboratory certification, not a Certificate of Sanitization or Destruction, and not proof that CYVORIQ authenticated the job.",
        "Magnetic and USB hard disks use a single-pass overwrite labelled NIST Clear, not Purge. Firmware sanitize that is unavailable fails closed. Host overwrite is not called Purge on flash.",
      ],
    },
  ];
}

function buildSanitizationDocument(
  record: PurgeRecord,
  verification: VerificationRecord | null,
  generatedAt: Date,
): PdfItem[] {
  const items: PdfItem[] = [
    { kind: "text", style: "kicker", text: "CYVORIQ SOLUTIONS PVT. LTD." },
    { kind: "text", style: "title", text: "Report S — Sanitization & Verification Record" },
    {
      kind: "text",
      style: "body",
      text: "Locally verified on this PC. Not a laboratory certification.",
    },
  ];

  for (const section of buildSanitizationSections(record, verification, generatedAt)) {
    if (section.kind === "table") {
      addSection(items, section.title, section.rows, section.empty);
      continue;
    }
    items.push({ kind: "gap", size: 10 });
    items.push({ kind: "text", style: "heading", text: section.title });
    items.push({ kind: "rule" });
    for (const paragraph of section.paragraphs) {
      items.push({ kind: "text", style: "body", text: paragraph });
    }
  }

  items.push({
    kind: "text",
    style: "body",
    text: "END OF REPORT S. This record is locally verified on this PC. It is not a laboratory certification.",
  });
  return items;
}

export function buildSanitizationPdf(
  record: PurgeRecord,
  verification: VerificationRecord | null,
  generatedAt = new Date(),
): Uint8Array {
  const documentId = makeSanitizationId(verification, generatedAt);
  return assemblePdf(
    paginate(buildSanitizationDocument(record, verification, generatedAt)),
    `${documentId}  ·  CYVORIQ Solutions Pvt. Ltd.  ·  locally verified, not a laboratory certification`,
  );
}

export function saveSanitizationPdf(
  record: PurgeRecord,
  verification: VerificationRecord | null,
): { filename: string; documentId: string } {
  if (!record.reportAllowed || record.status !== "VERIFIED") {
    throw new Error("Report S is issued only after independent verify PASS.");
  }
  const generatedAt = new Date();
  const documentId = makeSanitizationId(verification, generatedAt);
  const filename = `CYVRA-Erase-sanitization-${documentId}.pdf`;
  downloadPdf(buildSanitizationPdf(record, verification, generatedAt), filename);
  return { filename, documentId };
}
