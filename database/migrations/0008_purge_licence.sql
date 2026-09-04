BEGIN;

-- P-LICENCE: allow a second SKU on licenses without rewriting existing
-- CYVORIQ_ERASE rows. Assessment keys stay as they are. Purge keys use
-- product CYVORIQ_PURGE. Wipe execution stays off.

ALTER TABLE licenses DROP CONSTRAINT chk_licenses_product;

ALTER TABLE licenses
  ADD CONSTRAINT chk_licenses_product
  CHECK (product IN ('CYVORIQ_ERASE', 'CYVORIQ_PURGE'));

CREATE UNIQUE INDEX IF NOT EXISTS uq_licenses_user_product_active
  ON licenses (issued_to_user_id, product)
  WHERE status = 'active' AND issued_to_user_id IS NOT NULL;

COMMIT;
