

import { useMemo } from 'react';
import type { BreadcrumbItem } from '../search/details/Breadcrumbs';

/**
 * Builds breadcrumb items for the Browse tree view.
 *
 * Returns: Browse (clickable) > repository name (non-clickable).
 */
export function useBrowseBreadcrumbs(
  selectedRepository: string | null,
  onBackToList: () => void
): BreadcrumbItem[] {
  return useMemo(() => {
    if (!selectedRepository) {
      return [];
    }
    return [
      { label: 'Browse', onClick: onBackToList },
      { label: selectedRepository },
    ];
  }, [selectedRepository, onBackToList]);
}
