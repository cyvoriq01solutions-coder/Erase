BEGIN;

CREATE TABLE admin_auth_rate_limits (
  scope TEXT NOT NULL,
  key_hash TEXT NOT NULL,
  request_timestamps TIMESTAMPTZ[] NOT NULL
    DEFAULT ARRAY[]::TIMESTAMPTZ[],
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

  PRIMARY KEY (scope, key_hash),

  CONSTRAINT chk_admin_auth_rate_limits_scope
    CHECK (scope IN ('source', 'identity')),

  CONSTRAINT chk_admin_auth_rate_limits_key_hash
    CHECK (key_hash ~ '^[0-9a-f]{64}$'),

  CONSTRAINT chk_admin_auth_rate_limits_timestamp_count
    CHECK (
      (scope = 'source' AND cardinality(request_timestamps) <= 5)
      OR
      (scope = 'identity' AND cardinality(request_timestamps) <= 3)
    ),

  CONSTRAINT chk_admin_auth_rate_limits_no_null_timestamps
    CHECK (array_position(request_timestamps, NULL) IS NULL),

  CONSTRAINT chk_admin_auth_rate_limits_updated_at
    CHECK (updated_at >= created_at)
);

CREATE INDEX idx_admin_auth_rate_limits_updated_at
  ON admin_auth_rate_limits (updated_at);

COMMIT;
