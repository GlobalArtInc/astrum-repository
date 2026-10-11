import { useEffect, useState } from 'react';

/** Viewports narrower than a tablet: one column, lists instead of tables, a header of icons. */
export const PHONE_MEDIA_QUERY = '(max-width: 639px)';

/** Viewports narrower than a desktop (phones and tablets): the menu is a drawer and the search opens full screen. */
export const COMPACT_MEDIA_QUERY = '(max-width: 1023px)';

function matches(query: string): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return false;
  }
  return window.matchMedia(query).matches;
}

/**
 * Whether the viewport matches a media query, updated when it starts or stops matching.
 * False where the browser cannot evaluate media queries (server rendering, tests).
 */
export function useMediaQuery(query: string): boolean {
  const [isMatching, setIsMatching] = useState(() => matches(query));

  useEffect(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
      return undefined;
    }
    const mediaQueryList = window.matchMedia(query);
    const onChange = () => setIsMatching(mediaQueryList.matches);
    onChange();
    mediaQueryList.addEventListener('change', onChange);
    return () => mediaQueryList.removeEventListener('change', onChange);
  }, [query]);

  return isMatching;
}

/** Whether the viewport is phone sized, see {@link PHONE_MEDIA_QUERY}. */
export function useIsPhone(): boolean {
  return useMediaQuery(PHONE_MEDIA_QUERY);
}

/** Whether the viewport is narrower than a desktop, see {@link COMPACT_MEDIA_QUERY}. */
export function useIsCompactViewport(): boolean {
  return useMediaQuery(COMPACT_MEDIA_QUERY);
}
