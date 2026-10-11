
/**
 * Hook to get context-aware route name.
 * Prefixes with 'preview.': Astrum One is the only UI, and its states are the only ones registered.
 *
 * @param name - Base route name (e.g., 'browse.welcome')
 * @returns Context-aware route name (e.g., 'preview.browse.welcome')
 */
export function useContextAwareRouteName(name: string | undefined): string | undefined {
  if (!name) return name;

  return name.startsWith('preview.') ? name : `preview.${name}`;
}
