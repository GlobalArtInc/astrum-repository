

import { useState, useEffect } from 'react';

/**
 * Hook to detect if we're in Preview UI mode.
 * Checks if the URL hash starts with '#preview'.
 *
 * @returns {boolean} True if the current route is in Preview UI mode
 */
export function useIsPreviewUI(): boolean {
  const [isPreview, setIsPreview] = useState(() => window.location.hash.startsWith('#preview'));

  useEffect(() => {
    function checkPreview() {
      setIsPreview(window.location.hash.startsWith('#preview'));
    }

    window.addEventListener('hashchange', checkPreview);
    return () => window.removeEventListener('hashchange', checkPreview);
  }, []);

  return isPreview;
}
