BEGIN;

ALTER TABLE organizations
  ADD COLUMN slug TEXT;

CREATE UNIQUE INDEX idx_organizations_slug_unique
  ON organizations (LOWER(slug))
  WHERE slug IS NOT NULL;

ALTER TABLE users
  ADD CONSTRAINT uq_users_id_organization
  UNIQUE (id, organization_id);

ALTER TABLE devices
  ADD CONSTRAINT uq_devices_id_organization
  UNIQUE (id, organization_id);

CREATE TABLE login_challenges (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  user_id UUID NOT NULL,
  challenge_hash TEXT NOT NULL UNIQUE,
  delivery_channel TEXT NOT NULL DEFAULT 'email',
  attempts INTEGER NOT NULL DEFAULT 0,
  max_attempts INTEGER NOT NULL DEFAULT 5,
  expires_at TIMESTAMPTZ NOT NULL,
  consumed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_login_challenges_user_org
    FOREIGN KEY (user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT chk_login_challenges_delivery
    CHECK (delivery_channel IN ('email')),
  CONSTRAINT chk_login_challenges_attempts
    CHECK (attempts >= 0 AND max_attempts > 0 AND attempts <= max_attempts),
  CONSTRAINT chk_login_challenges_expiry
    CHECK (expires_at > created_at)
);

CREATE TABLE customer_sessions (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  user_id UUID NOT NULL,
  token_hash TEXT NOT NULL UNIQUE,
  expires_at TIMESTAMPTZ NOT NULL,
  revoked_at TIMESTAMPTZ,
  last_seen_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_customer_sessions_user_org
    FOREIGN KEY (user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT chk_customer_sessions_expiry
    CHECK (expires_at > created_at)
);

CREATE TABLE licenses (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  issued_to_user_id UUID,
  product TEXT NOT NULL DEFAULT 'CYVORIQ_ERASE',
  key_prefix TEXT NOT NULL,
  key_hash TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL DEFAULT 'active',
  max_devices INTEGER NOT NULL DEFAULT 1,
  issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  CONSTRAINT fk_licenses_user_org
    FOREIGN KEY (issued_to_user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT uq_licenses_id_organization
    UNIQUE (id, organization_id),
  CONSTRAINT chk_licenses_product
    CHECK (product IN ('CYVORIQ_ERASE')),
  CONSTRAINT chk_licenses_status
    CHECK (status IN ('active', 'suspended', 'revoked', 'expired')),
  CONSTRAINT chk_licenses_max_devices
    CHECK (max_devices > 0),
  CONSTRAINT chk_licenses_expiry
    CHECK (expires_at IS NULL OR expires_at > issued_at)
);

CREATE TABLE device_activations (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  license_id UUID NOT NULL,
  device_id UUID NOT NULL,
  activated_by_user_id UUID,
  fingerprint_hash TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  activated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_seen_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  CONSTRAINT fk_device_activations_license_org
    FOREIGN KEY (license_id, organization_id)
    REFERENCES licenses (id, organization_id),
  CONSTRAINT fk_device_activations_device_org
    FOREIGN KEY (device_id, organization_id)
    REFERENCES devices (id, organization_id),
  CONSTRAINT fk_device_activations_user_org
    FOREIGN KEY (activated_by_user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT uq_device_activation_license_fingerprint
    UNIQUE (license_id, fingerprint_hash),
  CONSTRAINT uq_device_activation_license_device
    UNIQUE (license_id, device_id),
  CONSTRAINT chk_device_activations_status
    CHECK (status IN ('active', 'revoked'))
);

CREATE TABLE agent_tokens (
  id UUID PRIMARY KEY,
  activation_id UUID NOT NULL REFERENCES device_activations(id),
  token_hash TEXT NOT NULL UNIQUE,
  scope TEXT NOT NULL DEFAULT 'agent:submit',
  expires_at TIMESTAMPTZ NOT NULL,
  revoked_at TIMESTAMPTZ,
  last_seen_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT chk_agent_tokens_scope
    CHECK (scope IN ('agent:submit')),
  CONSTRAINT chk_agent_tokens_expiry
    CHECK (expires_at > created_at)
);

CREATE INDEX idx_login_challenges_user_id
  ON login_challenges (user_id);

CREATE INDEX idx_login_challenges_expires_at
  ON login_challenges (expires_at);

CREATE INDEX idx_customer_sessions_user_id
  ON customer_sessions (user_id);

CREATE INDEX idx_customer_sessions_expires_at
  ON customer_sessions (expires_at);

CREATE INDEX idx_licenses_organization_id
  ON licenses (organization_id);

CREATE INDEX idx_licenses_status
  ON licenses (status);

CREATE INDEX idx_device_activations_organization_id
  ON device_activations (organization_id);

CREATE INDEX idx_device_activations_device_id
  ON device_activations (device_id);

CREATE INDEX idx_device_activations_status
  ON device_activations (status);

CREATE INDEX idx_agent_tokens_activation_id
  ON agent_tokens (activation_id);

CREATE INDEX idx_agent_tokens_expires_at
  ON agent_tokens (expires_at);

COMMIT;
