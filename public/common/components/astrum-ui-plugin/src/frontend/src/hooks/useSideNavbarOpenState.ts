import { useEffect, useRef, useState } from 'react';

export const SIDEBAR_OPEN_STORAGE_KEY = 'astrum.sidebar.open';

function storedOpen(): boolean | undefined {
  try {
    const value = window.localStorage.getItem(SIDEBAR_OPEN_STORAGE_KEY);
    return value === null ? undefined : value === 'true';
  } catch {
    return undefined;
  }
}

function storeOpen(open: boolean): void {
  try {
    window.localStorage.setItem(SIDEBAR_OPEN_STORAGE_KEY, String(open));
  } catch {
    // The choice is simply not remembered when storage is unavailable.
  }
}

/**
 * Shared open/collapsed state for the left navigation sidebar.
 *
 * Returns `[isOpen, onToggleClick]`. The state is also synced to the
 * `as-sidebar-toggle` window event so any component can open/collapse the
 * sidebar without prop drilling: dispatch `new CustomEvent('as-sidebar-toggle')`
 * to toggle, or pass `{ detail: { open: boolean } }` to force a state.
 *
 * A toggle by the user (`onToggleClick` or the event without detail) is remembered in local storage and restored
 * on the next page load; a forced state is not, so pages that open or close the sidebar for their own layout do not
 * change the user's choice.
 *
 * @param initialOpen - Open state used until the user has made a choice (`true` = expanded, `false` = collapsed).
 */
export function useSideNavbarOpenState(initialOpen: boolean): [boolean, () => void] {
  const [isOpen, setIsOpen] = useState(() => storedOpen() ?? initialOpen);
  const isOpenRef = useRef(isOpen);
  isOpenRef.current = isOpen;

  const onToggleClick = () => {
    const next = !isOpen;
    storeOpen(next);
    setIsOpen(next);
  };

  useEffect(() => {
    const handleGlobalToggle = (e: Event) => {
      const shouldOpen = (e as CustomEvent<{ open?: boolean }>).detail?.open;
      if (shouldOpen !== undefined) {
        setIsOpen(shouldOpen);
      } else {
        const next = !isOpenRef.current;
        isOpenRef.current = next;
        storeOpen(next);
        setIsOpen(next);
      }
    };

    window.addEventListener('as-sidebar-toggle', handleGlobalToggle);
    return () => window.removeEventListener('as-sidebar-toggle', handleGlobalToggle);
  }, []);

  return [isOpen, onToggleClick];
}
