

/** Hash target for Preview UI API hub ({@code preview.browse.api}). */
export const PREVIEW_BROWSE_API_HUB = '#browse/api';

export function apiHubHref(params: { user?: string; role?: string; permission?: string; endpoint?: string }): string {
  const sp = new URLSearchParams();
  if (params.user) {
    sp.set('user', params.user);
  }
  if (params.role) {
    sp.set('role', params.role);
  }
  if (params.permission) {
    sp.set('permission', params.permission);
  }
  if (params.endpoint) {
    sp.set('endpoint', params.endpoint);
  }
  const q = sp.toString();
  return q ? `${PREVIEW_BROWSE_API_HUB}?${q}` : PREVIEW_BROWSE_API_HUB;
}
