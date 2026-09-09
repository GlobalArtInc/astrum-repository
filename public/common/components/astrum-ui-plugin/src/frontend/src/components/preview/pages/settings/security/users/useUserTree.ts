

import { User } from './types';
import { useUserTreePreview, type UseUserTreeResult } from './useUserTreePreview';

export type { UseUserTreeResult };

/**
 * Hook: Build a user-rooted security tree (User -> Roles -> Privileges -> Content selectors).
 * Thin wrapper over useUserTreePreview seeded with the user's persisted roles; keeps the
 * Profile Security Tree tab and the Edit User inline preview on one node-id/label convention.
 */
export function useUserTree(user: User | null): UseUserTreeResult {
  return useUserTreePreview(user?.roles ?? [], user);
}
