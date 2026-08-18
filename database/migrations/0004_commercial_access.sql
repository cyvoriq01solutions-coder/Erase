BEGIN;

ALTER TABLE organizations
  ADD COLUMN account_type TEXT NOT NULL DEFAULT 'individual';

ALTER TABLE organizations
  ADD CONSTRAINT chk_organizations_account_type
  CHECK (account_type IN ('individual', 'enterprise', 'internal'));

ALTER TABLE users
  ADD COLUMN email_verified_at TIMESTAMPTZ,
  ADD COLUMN account_status TEXT NOT NULL DEFAULT 'pending_email_verification';

ALTER TABLE users
  ADD CONSTRAINT chk_users_account_status
  CHECK (account_status IN ('pending_email_verification', 'active', 'suspended', 'closed'));

CREATE UNIQUE INDEX idx_users_email_ci_unique
  ON users (LOWER(email));

CREATE TABLE user_roles (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  user_id UUID NOT NULL,
  role TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  approved_by_user_id UUID,
  approved_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_user_roles_user_org
    FOREIGN KEY (user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT fk_user_roles_approver_org
    FOREIGN KEY (approved_by_user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT uq_user_roles_user_role
    UNIQUE (user_id, role),
  CONSTRAINT chk_user_roles_role
    CHECK (role IN ('customer', 'accounts_admin', 'super_admin')),
  CONSTRAINT chk_user_roles_status
    CHECK (status IN ('pending', 'active', 'revoked'))
);

CREATE TABLE customer_orders (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  user_id UUID NOT NULL,
  product TEXT NOT NULL DEFAULT 'CYVORIQ_ERASE',
  status TEXT NOT NULL DEFAULT 'payment_pending',
  amount_minor BIGINT NOT NULL,
  currency TEXT NOT NULL,
  payment_received_at TIMESTAMPTZ,
  approved_by_user_id UUID,
  approved_at TIMESTAMPTZ,
  rejected_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_customer_orders_user_org
    FOREIGN KEY (user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT fk_customer_orders_approver_org
    FOREIGN KEY (approved_by_user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT uq_customer_orders_id_organization
    UNIQUE (id, organization_id),
  CONSTRAINT chk_customer_orders_product
    CHECK (product IN ('CYVORIQ_ERASE')),
  CONSTRAINT chk_customer_orders_status
    CHECK (status IN (
      'payment_pending',
      'payment_received',
      'approval_pending',
      'approved',
      'rejected',
      'cancelled',
      'fulfilled'
    )),
  CONSTRAINT chk_customer_orders_amount
    CHECK (amount_minor >= 0),
  CONSTRAINT chk_customer_orders_currency
    CHECK (currency ~ '^[A-Z]{3}$')
);

CREATE TABLE payment_records (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  order_id UUID NOT NULL,
  provider TEXT NOT NULL,
  provider_reference TEXT NOT NULL,
  amount_minor BIGINT NOT NULL,
  currency TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  verified_by_user_id UUID,
  verified_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_payment_records_order_org
    FOREIGN KEY (order_id, organization_id)
    REFERENCES customer_orders (id, organization_id),
  CONSTRAINT fk_payment_records_verifier_org
    FOREIGN KEY (verified_by_user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT uq_payment_provider_reference
    UNIQUE (provider, provider_reference),
  CONSTRAINT chk_payment_records_status
    CHECK (status IN ('pending', 'received', 'failed', 'refunded')),
  CONSTRAINT chk_payment_records_amount
    CHECK (amount_minor >= 0),
  CONSTRAINT chk_payment_records_currency
    CHECK (currency ~ '^[A-Z]{3}$')
);

CREATE TABLE download_entitlements (
  id UUID PRIMARY KEY,
  organization_id UUID NOT NULL REFERENCES organizations(id),
  user_id UUID NOT NULL,
  order_id UUID NOT NULL,
  license_id UUID,
  product TEXT NOT NULL DEFAULT 'CYVORIQ_ERASE',
  status TEXT NOT NULL DEFAULT 'pending',
  enabled_by_user_id UUID,
  enabled_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT fk_download_entitlements_user_org
    FOREIGN KEY (user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT fk_download_entitlements_order_org
    FOREIGN KEY (order_id, organization_id)
    REFERENCES customer_orders (id, organization_id),
  CONSTRAINT fk_download_entitlements_license_org
    FOREIGN KEY (license_id, organization_id)
    REFERENCES licenses (id, organization_id),
  CONSTRAINT fk_download_entitlements_enabler_org
    FOREIGN KEY (enabled_by_user_id, organization_id)
    REFERENCES users (id, organization_id),
  CONSTRAINT uq_download_entitlements_order
    UNIQUE (order_id),
  CONSTRAINT chk_download_entitlements_product
    CHECK (product IN ('CYVORIQ_ERASE')),
  CONSTRAINT chk_download_entitlements_status
    CHECK (status IN ('pending', 'enabled', 'revoked')),
  CONSTRAINT chk_download_entitlements_expiry
    CHECK (expires_at IS NULL OR enabled_at IS NULL OR expires_at > enabled_at)
);

CREATE INDEX idx_user_roles_user_id
  ON user_roles (user_id);

CREATE INDEX idx_user_roles_role_status
  ON user_roles (role, status);

CREATE INDEX idx_customer_orders_user_id
  ON customer_orders (user_id);

CREATE INDEX idx_customer_orders_status
  ON customer_orders (status);

CREATE INDEX idx_payment_records_order_id
  ON payment_records (order_id);

CREATE INDEX idx_payment_records_status
  ON payment_records (status);

CREATE INDEX idx_download_entitlements_user_id
  ON download_entitlements (user_id);

CREATE INDEX idx_download_entitlements_status
  ON download_entitlements (status);

COMMIT;
