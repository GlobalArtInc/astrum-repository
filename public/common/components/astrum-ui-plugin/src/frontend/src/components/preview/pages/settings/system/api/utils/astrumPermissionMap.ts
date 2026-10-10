

/**
 * Effective permission flags from classic UI / Ext bootstrap ({@code AS.Permissions.permissions}).
 */
export function readAstrumPermissionMap(): Record<string, boolean> {
  try {
    const w = window as unknown as { AS?: { Permissions?: { permissions?: Record<string, boolean> } } };
    const p = w.AS?.Permissions?.permissions;
    if (p && typeof p === 'object') {
      return p;
    }
  } catch {
    // ignore
  }
  return {};
}
