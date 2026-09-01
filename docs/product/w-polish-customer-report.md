# W-polish: customer report and verification journey

Customer-facing polish on top of W-collect. Purge, grading issuance and cloud report authentication stay off.

## What changed

1. Verification is a real journey: choose drives, start, watch progress, review results, generate the report.
2. USB, optical and extra letters are listed. The Windows system drive is selected by default. Extra disks stay off until the user checks them.
3. Progress events run off the UI thread so Windows does not mark the window as Not Responding. The footer and verification screen show the current step and a percent bar.
4. The Report screen is a structured assessment. Raw Assessment JSON and validator key=value dumps are not shown.
5. Generate report is the last customer action on Results. The same report can be emailed through the user’s mail application.
6. Exit is available in the title bar, sidebar and Help.

## Still off

- Destructive operations
- Grading issuance
- Cloud report authentication
- Authenticode signing and B2 unsigned upload
