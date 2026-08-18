export const CEO_SUPER_USER_EMAIL = "ceo@cyvra.co.in";
export const ACCOUNTS_APPROVER_EMAIL = "accounts@cyvra.co.in";

export type ControlPanelRole = "super_admin" | "accounts_admin";
export type UserRole = "customer" | ControlPanelRole;

export interface ActiveRole {
  role: UserRole;
  status: "active";
}

export function normalizeAuthorizationEmail(email: string): string {
  return email.trim().toLowerCase();
}

export function isBootstrapSuperUser(email: string): boolean {
  return normalizeAuthorizationEmail(email) === CEO_SUPER_USER_EMAIL;
}

export function isAccountsAuthorityIdentity(email: string): boolean {
  return normalizeAuthorizationEmail(email) === ACCOUNTS_APPROVER_EMAIL;
}

export function canAccessControlPanel(roles: readonly ActiveRole[]): boolean {
  return roles.some(
    ({ role }) => role === "super_admin" || role === "accounts_admin",
  );
}

export function canManageAdminRoles(roles: readonly ActiveRole[]): boolean {
  return roles.some(({ role }) => role === "super_admin");
}

export function canApproveCommercialPurchase(
  roles: readonly ActiveRole[],
): boolean {
  return roles.some(
    ({ role }) => role === "super_admin" || role === "accounts_admin",
  );
}

export function canConfirmPayment(roles: readonly ActiveRole[]): boolean {
  return canApproveCommercialPurchase(roles);
}

export function canEnableDownloadEntitlement(
  roles: readonly ActiveRole[],
): boolean {
  return canApproveCommercialPurchase(roles);
}

export function canExerciseSuperUserAuthority(
  email: string,
  roles: readonly ActiveRole[],
): boolean {
  return (
    isBootstrapSuperUser(email) &&
    roles.some(({ role }) => role === "super_admin")
  );
}
