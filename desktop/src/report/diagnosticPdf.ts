import {
  addSection,
  assemblePdf,
  downloadPdf,
  paginate,
  type PdfItem,
} from "./assessmentPdf";
import type { AdvanceScanRecord, NamedValue, VerificationRecord } from "../types/shell";
import { SOFTWARE_OBSERVED_LABEL } from "../types/shell";

/** Report D document number, matching the ERA- convention used by Report A. */
export function makeDiagnosticId(
  verification: VerificationRecord | null,
  generatedAt: Date,
): string {
  const day = generatedAt.toISOString().slice(0, 10).replace(/-/g, "");
  const host = (verification?.hostname ?? "").replace(/[^A-Za-z0-9]/g, "").slice(0, 18) || "PC";
  return `ERD-${day}-${host}`;
}

function gradeRows(advanceScan: AdvanceScanRecord): NamedValue[] {
  const condition = advanceScan.gradeObservation
    ? `${advanceScan.gradeCondition} — ${SOFTWARE_OBSERVED_LABEL}`
    : advanceScan.gradeCondition;
  const rows: NamedValue[] = [
    { label: "Provisional grade", value: advanceScan.gradeLabel },
    { label: "Condition", value: condition },
    {
      label: "Assessed Health Index",
      value:
        advanceScan.indexPercent === null
          ? "Not assessable in this scan"
          : `${advanceScan.indexPercent} / 100`,
    },
    { label: "Coverage", value: `${advanceScan.coveragePercent}%` },
    { label: "Graded by", value: `${advanceScan.gradingEngine} · rubric ${advanceScan.gradingRubric}` },
    {
      label: "Issuance",
      value: advanceScan.issuanceNotice,
    },
    {
      label: "Physical verification",
      value: "Required for a final grade",
    },
  ];
  if (advanceScan.gradeWithheldReason) {
    rows.push({ label: "Why no grade", value: advanceScan.gradeWithheldReason });
  }
  return rows;
}

function identityRows(verification: VerificationRecord | null): NamedValue[] {
  if (!verification) {
    return [];
  }
  return [
    { label: "Computer name", value: verification.hostname },
    { label: "Manufacturer", value: verification.manufacturer },
    { label: "Model", value: verification.model },
    { label: "Operating system", value: verification.osCaption },
    { label: "Drives in the basic assessment", value: verification.scannedDrives },
  ];
}

function coverageByArea(advanceScan: AdvanceScanRecord): NamedValue[] {
  return advanceScan.coverageDomains.map((domain) => ({
    label: domain.domain,
    value: `${domain.state} (${domain.confidence}) — awarded ${domain.awarded}, assessed ${domain.assessed}, not assessable ${domain.notAssessable} of ${domain.weight} points. ${domain.note}.`,
  }));
}

export function buildDiagnosticDocument(
  advanceScan: AdvanceScanRecord,
  verification: VerificationRecord | null,
  generatedAt: Date,
): PdfItem[] {
  const documentId = makeDiagnosticId(verification, generatedAt);
  const items: PdfItem[] = [
    { kind: "text", style: "kicker", text: "CYVORIQ SOLUTIONS PVT. LTD.  ·  CYVRA ERASE  ·  REPORT D" },
    { kind: "text", style: "title", text: "In-depth hardware diagnostic evaluation" },
    {
      kind: "text",
      style: "meta",
      text: `Document no.  ${documentId}    Generated on this PC  ${generatedAt.toLocaleString()}`,
    },
    {
      kind: "text",
      style: "meta",
      text: `Scan scope: Advance scan  ·  ${advanceScan.elevationLabel}  ·  Bytes written to assessed drives: ${advanceScan.bytesWritten}`,
    },
    { kind: "gap", size: 8 },
    {
      kind: "text",
      style: "notice",
      text: "This is a computer-generated diagnostic evaluation produced on the assessed Windows PC by CYVRA Erase, software of CYVORIQ Solutions Pvt. Ltd. It is not a Certificate of Sanitization or Destruction, not NIST SP 800-88 Purge proof, and not a DPDP compliance certificate. Any grade shown is provisional; a final device grade is possible only after physical verification by a technician. This document is not cloud-authenticated in this version.",
    },
    {
      kind: "text",
      style: "notice",
      text: `${advanceScan.boundaryNote} ${advanceScan.temporaryFilesNote}`,
    },
  ];

  const identity = identityRows(verification);
  addSection(
    items,
    "1. Device identity",
    identity,
    "No basic assessment has been run in this session, so device identity is not carried onto this report.",
  );

  addSection(
    items,
    "2. Coverage statement",
    advanceScan.coverageRows,
    "No coverage statement was produced.",
  );

  addSection(
    items,
    "3. Coverage by diagnostic area",
    coverageByArea(advanceScan),
    "No diagnostic areas were evaluated.",
  );

  addSection(items, "4. Grading summary", gradeRows(advanceScan), "No grade was produced.");

  let sectionNumber = 5;
  for (const group of advanceScan.telemetryGroups) {
    const rows = group.note
      ? [...group.rows, { label: "Note", value: group.note }]
      : group.rows;
    addSection(items, `${sectionNumber}. ${group.title}`, rows, "Not collected in this scan.");
    sectionNumber += 1;
  }

  addSection(
    items,
    `${sectionNumber}. Not assessable in this scan`,
    advanceScan.notAssessable.map((entry, index) => ({
      label: `Gap ${index + 1}`,
      value: entry,
    })),
    "Every diagnostic area was assessed.",
  );
  sectionNumber += 1;

  addSection(
    items,
    `${sectionNumber}. Method and limitations`,
    advanceScan.methodRows,
    "No method statement was produced.",
  );
  sectionNumber += 1;

  addSection(
    items,
    `${sectionNumber}. Grading rubric`,
    advanceScan.rubricRows,
    "No rubric was recorded.",
  );
  sectionNumber += 1;

  const seal = advanceScan.integritySeal;
  if (seal) {
    addSection(
      items,
      `${sectionNumber}. Local integrity seal`,
      [
        { label: "Scheme", value: seal.scheme },
        { label: "SHA-256 digest", value: seal.digestHex },
        { label: "QR payload", value: seal.qrPayload },
        { label: "Ed25519 public key", value: seal.publicKeyHex },
        { label: "Ed25519 signature", value: seal.signatureHex },
        { label: "What this is", value: seal.notice },
      ],
      "No local integrity seal was attached.",
    );
    sectionNumber += 1;
  }

  items.push({ kind: "gap", size: 10 });
  items.push({ kind: "text", style: "heading", text: `${sectionNumber}. Issuing organisation` });
  items.push({ kind: "rule" });
  items.push({
    kind: "text",
    style: "body",
    text: "Issued by CYVORIQ Solutions Pvt. Ltd. as publisher of CYVRA Erase. This document is computer-generated on the assessed PC. It is not cloud-authenticated in this version. A local integrity seal, when present, proves the JSON was not altered after the scan. It is not Authenticode and does not certify sanitization, destruction, resale grade, or legal compliance.",
  });
  items.push({
    kind: "text",
    style: "body",
    text: `Graded by ${advanceScan.gradingEngine} using rubric ${advanceScan.gradingRubric}. The engine applies fixed, published rules to the evidence collected on this PC. Areas that were not measured are neither credited nor penalised.`,
  });
  items.push({
    kind: "text",
    style: "body",
    text: "Physical verification. Technician name: recorded at sign-off on the assessed PC. Date of inspection: recorded at sign-off on the assessed PC. Result: see Technician checks. USB topology and battery/charger state come from the Advance scan pass, not from a live USB or live charger overlay. This block is not a handwritten signature line.",
  });

  return items;
}

export function buildDiagnosticPdf(
  advanceScan: AdvanceScanRecord,
  verification: VerificationRecord | null,
  generatedAt = new Date(),
): Uint8Array {
  const documentId = makeDiagnosticId(verification, generatedAt);
  const items = buildDiagnosticDocument(advanceScan, verification, generatedAt);
  return assemblePdf(
    paginate(items),
    `${documentId}  ·  CYVORIQ Solutions Pvt. Ltd.  ·  provisional, physical verification required`,
  );
}

export function saveDiagnosticPdf(
  advanceScan: AdvanceScanRecord,
  verification: VerificationRecord | null,
): { filename: string; documentId: string } {
  const generatedAt = new Date();
  const documentId = makeDiagnosticId(verification, generatedAt);
  const filename = `CYVRA-Erase-diagnostic-${documentId}.pdf`;
  downloadPdf(buildDiagnosticPdf(advanceScan, verification, generatedAt), filename);
  return { filename, documentId };
}
