
import { useEffect, useState } from 'react';

/**
 * Shared open/collapsed state for the left navigation sidebar.
 *
 * Returns `[isOpen, onToggleClick]`. The state is also synced to the
 * `as-sidebar-toggle` window event so any component can open/collapse the
 * sidebar without prop drilling: dispatch `new CustomEvent('as-sidebar-toggle')`
 * to toggle, or pass `{ detail: { open: boolean } }` to force a state.
 *
 * @param initialOpen - Initial open state (`true` = expanded, `false` = collapsed).
 */
export function useSideNavbarOpenState(initialOpen: boolean): [boolean, () => void] {
  const [isOpen, setIsOpen] = useState(initialOpen);

  const onToggleClick = () => {
    setIsOpen(!isOpen);
  };

  useEffect(() => {
    const handleGlobalToggle = (e: Event) => {
      const shouldOpen = (e as CustomEvent<{ open?: boolean }>).detail?.open;
      if (shouldOpen !== undefined) {
        setIsOpen(shouldOpen);
      } else {
        setIsOpen((prev) => !prev);
      }
    };

    window.addEventListener('as-sidebar-toggle', handleGlobalToggle);
    return () => window.removeEventListener('as-sidebar-toggle', handleGlobalToggle);
  }, []);

  return [isOpen, onToggleClick];
}
