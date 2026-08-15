BEGIN;

CREATE TABLE organizations (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  email TEXT NOT NULL,
  display_name TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (organization_id, email)
);

CREATE TABLE assets (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  asset_tag TEXT,
  serial_number TEXT,
  status TEXT NOT NULL DEFAULT 'registered',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE devices (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  asset_id UUID REFERENCES assets(id),
  platform TEXT NOT NULL,
  hostname TEXT,
  manufacturer TEXT,
  model TEXT,
  serial_number TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE assessments (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  device_id UUID NOT NULL REFERENCES devices(id),
  status TEXT NOT NULL DEFAULT 'pending',
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE evidence (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  device_id UUID NOT NULL REFERENCES devices(id),
  assessment_id UUID REFERENCES assessments(id),
  evidence_type TEXT NOT NULL,
  source TEXT NOT NULL,
  sha256 TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE verification_results (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  device_id UUID NOT NULL REFERENCES devices(id),
  assessment_id UUID REFERENCES assessments(id),
  result TEXT NOT NULL,
  confidence NUMERIC(5,4),
  rationale JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE audit_events (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  actor_id UUID,
  event_type TEXT NOT NULL,
  entity_type TEXT,
  entity_id UUID,
  details JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_organization_id
  ON users (organization_id);

CREATE INDEX idx_assets_organization_id
  ON assets (organization_id);

CREATE INDEX idx_devices_organization_id
  ON devices (organization_id);

CREATE INDEX idx_devices_asset_id
  ON devices (asset_id);

CREATE INDEX idx_assessments_device_id
  ON assessments (device_id);

CREATE INDEX idx_evidence_device_id
  ON evidence (device_id);

CREATE INDEX idx_verification_results_device_id
  ON verification_results (device_id);

CREATE INDEX idx_audit_events_organization_id
  ON audit_events (organization_id);

COMMIT;
