BEGIN;

CREATE TABLE admin_login_challenges (
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
  CONSTRAINT fk_admin_login_challenges_user_org
    FOREIGN KEY (user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT chk_admin_login_challenges_delivery
    CHECK (delivery_channel IN ('email')),
  CONSTRAINT chk_admin_login_challenges_attempts
    CHECK (attempts >= 0 AND max_attempts > 0 AND attempts <= max_attempts),
  CONSTRAINT chk_admin_login_challenges_expiry
    CHECK (expires_at > created_at)
);

CREATE TABLE admin_sessions (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  user_id UUID NOT NULL,
  token_hash TEXT NOT NULL UNIQUE,
  expires_at TIMESTAMPTZ NOT NULL,
  revoked_at TIMESTAMPTZ,
  last_seen_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_admin_sessions_user_org
    FOREIGN KEY (user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT chk_admin_sessions_expiry
    CHECK (expires_at > created_at)
);

CREATE INDEX idx_admin_login_challenges_user_id
  ON admin_login_challenges (user_id);

CREATE INDEX idx_admin_login_challenges_expires_at
  ON admin_login_challenges (expires_at);

CREATE INDEX idx_admin_sessions_user_id
  ON admin_sessions (user_id);

CREATE INDEX idx_admin_sessions_expires_at
  ON admin_sessions (expires_at);

COMMIT;
