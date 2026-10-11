
/**
 * Hook to detect if we're in Preview UI (Astrum One) mode.
 *
 * The Classic UI is retired and Astrum One owns every URL, so the answer is always yes. The hook stays so the
 * components that ask keep working until their Classic branches are removed.
 *
 * @returns {boolean} Always true
 */
export function useIsPreviewUI(): boolean {
  return true;
}
