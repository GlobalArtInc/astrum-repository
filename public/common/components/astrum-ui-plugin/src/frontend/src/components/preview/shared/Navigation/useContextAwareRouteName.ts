
import { useContext } from 'react';
import { PreviewUIContext } from './PreviewUIContext';

/**
 * Hook to get context-aware route name.
 * Prefixes with 'preview.' when in Preview UI mode.
 *
 * @param name - Base route name (e.g., 'browse.welcome')
 * @returns Context-aware route name (e.g., 'preview.browse.welcome' in Preview UI)
 */
export function useContextAwareRouteName(name: string | undefined): string | undefined {
  const isPreviewUI = useContext(PreviewUIContext);

  if (!name) return name;

  // If already has preview prefix, return as is
  if (name.startsWith('preview.')) {
    return isPreviewUI ? name : name.replace('preview.', '');
  }

  // Add preview prefix if in preview mode
  return isPreviewUI ? `preview.${name}` : name;
}
