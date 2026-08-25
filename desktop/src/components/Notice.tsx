import type { ReactNode } from "react";

type NoticeKind = "information" | "success" | "warning" | "error";

interface NoticeProps {
  kind: NoticeKind;
  title: string;
  children: ReactNode;
}

const symbols: Record<NoticeKind, string> = {
  information: "i",
  success: "✓",
  warning: "!",
  error: "×",
};

export function Notice({ kind, title, children }: NoticeProps) {
  return (
    <section className={`notice notice-${kind}`} aria-label={title}>
      <span className="notice-symbol" aria-hidden="true">
        {symbols[kind]}
      </span>
      <div>
        <strong>{title}</strong>
        <div className="notice-copy">{children}</div>
      </div>
    </section>
  );
}
