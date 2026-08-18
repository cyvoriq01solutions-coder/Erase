BEGIN;

CREATE UNIQUE INDEX idx_users_org_email_ci_unique
  ON users (organization_id, LOWER(email));

CREATE INDEX idx_login_challenges_user_created_at
  ON login_challenges (organization_id, user_id, created_at DESC);

COMMIT;
