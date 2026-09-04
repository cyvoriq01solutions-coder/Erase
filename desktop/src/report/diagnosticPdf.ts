import {
  addSection,
  assemblePdf,
  downloadPdf,
  paginate,
  type AssessmentSection,
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
    {
      label: "Final device grade",
      value: advanceScan.gradeWithheld ? "WITHHELD" : advanceScan.gradeLabel,
    },
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
    rows.push({ label: "Grade-withholding reason", value: advanceScan.gradeWithheldReason });
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

function sealRows(advanceScan: AdvanceScanRecord): NamedValue[] {
  const seal = advanceScan.integritySeal;
  if (!seal) {
    return [];
  }
  return [
    { label: "Scheme", value: seal.scheme },
    { label: "SHA-256 digest", value: seal.digestHex },
    { label: "QR payload", value: seal.qrPayload },
    { label: "Ed25519 public key", value: seal.publicKeyHex },
    { label: "Ed25519 signature", value: seal.signatureHex },
    { label: "What this is", value: seal.notice },
    {
      label: "Authentication limitation",
      value: "Not cloud-authenticated; not Authenticode; not an organizational certificate.",
    },
  ];
}

export function buildDiagnosticSections(
  advanceScan: AdvanceScanRecord,
  verification: VerificationRecord | null,
  generatedAt: Date,
): AssessmentSection[] {
  const documentId = makeDiagnosticId(verification, generatedAt);
  const sections: AssessmentSection[] = [
    {
      kind: "table",
      title: "Executive Decision Snapshot",
      rows: [
        { label: "Report identifier", value: documentId },
        { label: "Assessed host", value: verification?.hostname ?? "Not recorded in this session" },
        { label: "Assessment scope", value: "Advance scan" },
        {
          label: "Assessed Health Index",
          value:
            advanceScan.indexPercent === null
              ? "Not assessable in this scan"
              : `${advanceScan.indexPercent} / 100`,
        },
        { label: "Diagnostic coverage", value: `${advanceScan.coveragePercent}%` },
        { label: "Final device grade", value: advanceScan.gradeWithheld ? "WITHHELD" : advanceScan.gradeLabel },
        {
          label: "Grade-withholding reason",
          value: advanceScan.gradeWithheldReason ?? "Not withheld",
        },
        { label: "Data erased", value: "No" },
        { label: "Bytes written to assessed drives", value: String(advanceScan.bytesWritten) },
      ],
      empty: "No snapshot was produced.",
    },
    {
      kind: "table",
      title: "1. Document Control & Evidence Status",
      rows: [
        { label: "Report identifier", value: documentId },
        { label: "Report type", value: "Report D — Technical Diagnostic & Condition Evidence Record" },
        { label: "Generated on assessed PC", value: generatedAt.toLocaleString() },
        { label: "Assessed host", value: verification?.hostname ?? "Not recorded in this session" },
        { label: "Issuing software", value: "CYVRA Erase" },
        { label: "Publisher", value: "CYVORIQ Solutions Pvt. Ltd." },
        { label: "Scan scope", value: "Advance scan" },
        { label: "Administrator approval", value: advanceScan.elevationLabel },
        { label: "Cloud authentication", value: "Not enabled in this version" },
        {
          label: "Current issuance state",
          value:
            "Computer-generated local diagnostic record; not an organization-authenticated certificate in this version.",
        },
      ],
      empty: "Document control was not available.",
    },
    {
      kind: "prose",
      title: "2. Evidence Status: What This Report Does and Does Not Prove",
      paragraphs: [
        "A final resale, refurbishment or condition grade should not be issued from this evidence set while mandatory evidence remains unavailable or physical verification remains incomplete. USB topology and battery/charger state come from the Advance scan pass, not from a live USB or live charger overlay.",
      ],
    },
    {
      kind: "table",
      title: "3. Device Identity",
      rows: identityRows(verification),
      empty:
        "No basic assessment has been run in this session, so device identity is not carried onto this report.",
    },
    {
      kind: "table",
      title: "4. Coverage statement",
      rows: [...advanceScan.coverageRows, ...gradeRows(advanceScan)],
      empty: "No coverage statement was produced.",
    },
    {
      kind: "table",
      title: "5. Coverage by diagnostic area",
      rows: coverageByArea(advanceScan),
      empty: "No diagnostic areas were evaluated.",
    },
    {
      kind: "table",
      title: "6. Key Findings & Recommended Actions",
      rows: [
        ...(advanceScan.gradeWithheldReason
          ? [{ label: "Evidence gap", value: advanceScan.gradeWithheldReason }]
          : []),
        ...advanceScan.notAssessable.map((entry, index) => ({
          label: `Recorded gap ${index + 1}`,
          value: entry,
        })),
        {
          label: "Physical verification",
          value: "Complete controlled technician checks before treating any grade as final.",
        },
        {
          label: "Erasure",
          value: "No data was erased. Temporary benchmark writes, if any, are not sanitization.",
        },
      ],
      empty: "No additional findings were recorded.",
    },
  ];

  let sectionNumber = 7;
  for (const group of advanceScan.telemetryGroups) {
    const rows = group.note ? [...group.rows, { label: "Note", value: group.note }] : group.rows;
    sections.push({
      kind: "table",
      title: `${sectionNumber}. ${group.title}`,
      rows,
      empty: "Not collected in this scan.",
    });
    sectionNumber += 1;
  }

  sections.push(
    {
      kind: "table",
      title: "15. Method and limitations",
      rows: advanceScan.methodRows,
      empty: "No method statement was produced.",
    },
    {
      kind: "table",
      title: "16. Grading rubric",
      rows: advanceScan.rubricRows,
      empty: "No rubric was recorded.",
    },
    {
      kind: "table",
      title: "17. Local Integrity Evidence",
      rows: sealRows(advanceScan),
      empty: "No local integrity seal was attached.",
    },
    {
      kind: "prose",
      title: "18. Audit-Ready Evidence Controls Recommended",
      paragraphs: [
        "Recommended for a later production issuance: evidence provenance, time integrity, authorization binding, technician attestation, immutability, version control, retention and supersession. These controls are not enabled in this version.",
      ],
    },
    {
      kind: "prose",
      title: "19. Final Recommended Next Action",
      paragraphs: [
        advanceScan.gradeWithheld
          ? "Resolve recorded evidence gaps, complete mandatory physical verification, then re-run grading against the completed evidence set. If the device proceeds to data disposal or reuse, create a separate Report S sanitization and verification record rather than modifying Report D to claim erasure."
          : "Complete physical verification before treating the provisional grade as final. If the device proceeds to data disposal or reuse, create a separate Report S sanitization and verification record rather than modifying Report D to claim erasure.",
      ],
    },
    {
      kind: "prose",
      title: "20. Controlled Issuance Statement",
      paragraphs: [
        "Issued by CYVORIQ Solutions Pvt. Ltd. as publisher of CYVRA Erase. This document is a computer-generated technical diagnostic and condition-evidence record. It is not cloud-authenticated in this version. A local integrity seal, when present, proves the JSON was not altered after the scan. It is not Authenticode and does not certify sanitization, destruction, resale grade, or legal compliance.",
        `Graded by ${advanceScan.gradingEngine} using rubric ${advanceScan.gradingRubric}. The engine applies fixed, published rules to the evidence collected on this PC. Areas that were not measured are neither credited nor penalised.`,
        "Physical verification. Technician name: recorded at sign-off on the assessed PC. Date of inspection: recorded at sign-off on the assessed PC. Result: see Technician checks. This block is not a handwritten signature line.",
      ],
    },
  );

  return sections;
}

export function buildDiagnosticDocument(
  advanceScan: AdvanceScanRecord,
  verification: VerificationRecord | null,
  generatedAt: Date,
): PdfItem[] {
  const documentId = makeDiagnosticId(verification, generatedAt);
  const items: PdfItem[] = [
    { kind: "text", style: "kicker", text: "CYVRA ERASE  |  REPORT D  |  TECHNICAL DIAGNOSTIC & CONDITION EVIDENCE" },
    { kind: "text", style: "kicker", text: "CYVORIQ SOLUTIONS PVT. LTD." },
    {
      kind: "text",
      style: "meta",
      text: "Controlled diagnostic evidence record  |  Final grade status shown independently from assessed index",
    },
    { kind: "text", style: "title", text: "Technical Diagnostic & Condition Evidence Record" },
    {
      kind: "text",
      style: "notice",
      text: advanceScan.gradeWithheld
        ? "DOCUMENT STATUS: DIAGNOSTIC EVIDENCE RECORD — FINAL DEVICE GRADE WITHHELD"
        : "DOCUMENT STATUS: DIAGNOSTIC EVIDENCE RECORD — PROVISIONAL GRADE, PHYSICAL VERIFICATION REQUIRED",
    },
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
      text: "This report documents diagnostic evidence collected from the assessed Windows PC and controlled technician/operator checks recorded during the diagnostic workflow. It is not a Certificate of Sanitization or Destruction, is not proof of NIST SP 800-88 purge completion, and does not independently certify compliance with the DPDP Act, ISO standards or any other legal/regulatory framework. A user must never mistake the Assessed Health Index for an issued final grade.",
    },
    {
      kind: "text",
      style: "notice",
      text: `${advanceScan.boundaryNote} ${advanceScan.temporaryFilesNote}`,
    },
  ];

  for (const section of buildDiagnosticSections(advanceScan, verification, generatedAt)) {
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
    text: "END OF REPORT D. Next document in the CYVRA evidence family: REPORT S — Sanitization & Verification Record (not generated in this version).",
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
