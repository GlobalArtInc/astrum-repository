

import { useMemo, useState } from 'react';
import { useCombinedRoleTree, type SecurityTreeNode } from '../roles/useRoleTree';
import { DEFAULT_SOURCE, User, getSourceLabel } from './types';

export interface UseUserTreeResult {
  tree: SecurityTreeNode[];
  loading: boolean;
  error: string | null;
  toggleExpand: (nodeId: string) => void;
  expandAll: () => void;
  collapseAll: () => void;
  setSearchTerm: (term: string) => void;
}

/** Builds a user-rooted security tree from an explicit set of role IDs. Consumed
 *  by useUserTree (persisted roles) and by the Edit User inline preview (pending
 *  roles) so both paths render identical node ids and labels. */
export function useUserTreePreview(
  pendingRoleIds: string[],
  user: User | null = null,
): UseUserTreeResult {
  const [searchTerm, setSearchTerm] = useState('');

  const {
    tree: roleTrees,
    loading,
    error,
    toggleExpand,
    expandAll,
    collapseAll,
  } = useCombinedRoleTree(pendingRoleIds, { searchTerm });

  const tree = useMemo<SecurityTreeNode[]>(() => {
    if (roleTrees.length === 0) return [];
    if (!user) return roleTrees;
    const source = user.source || DEFAULT_SOURCE;
    const userRoot: SecurityTreeNode = {
      id: `user:${user.userId}:${source}`,
      entityId: user.userId,
      name: `User: ${user.userId} (${getSourceLabel(source)})`,
      type: 'role',
      inherited: false,
      expanded: true,
      isVisible: true,
      children: roleTrees,
    };
    return [userRoot];
  }, [user, roleTrees]);

  return {
    tree,
    loading,
    error,
    toggleExpand,
    expandAll,
    collapseAll,
    setSearchTerm,
  };
}
