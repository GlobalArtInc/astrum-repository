

/**
 * Effective permission flags from classic UI / Ext bootstrap ({@code NX.Permissions.permissions}).
 */
export function readAstrumPermissionMap(): Record<string, boolean> {
  try {
    const w = window as unknown as { NX?: { Permissions?: { permissions?: Record<string, boolean> } } };
    const p = w.NX?.Permissions?.permissions;
    if (p && typeof p === 'object') {
      return p;
    }
  } catch {
    // ignore
  }
  return {};
}
