BEGIN;

CREATE TABLE IF NOT EXISTS customer_access_decisions (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  user_id UUID NOT NULL,
  status TEXT NOT NULL DEFAULT 'waiting',
  reject_reason TEXT,
  decided_by_user_id UUID,
  decided_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_customer_access_user_org
    FOREIGN KEY (user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT uq_customer_access_user
    UNIQUE (user_id),
  CONSTRAINT chk_customer_access_status
    CHECK (status IN ('waiting', 'approved', 'rejected')),
  CONSTRAINT chk_customer_access_reject_reason
    CHECK (
      (status <> 'rejected' AND reject_reason IS NULL)
      OR (status = 'rejected' AND reject_reason IS NOT NULL AND char_length(reject_reason) BETWEEN 8 AND 500)
    )
);

CREATE INDEX IF NOT EXISTS idx_customer_access_status
  ON customer_access_decisions (status);

COMMIT;
