import type { MouseEvent, RefObject } from 'react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useTransitionHook } from '@uirouter/react';
import { useIsCompactViewport } from '@globalart/astrum-ui-plugin';
import { trapTabKey } from '../../utils/focusTrap';

/**
 * On phones and tablets the menu of Astrum One is a drawer over the page, opened by the menu button of the header.
 * The header and the menu are separate components, so they share its state through this window event.
 */
export const NAVIGATION_DRAWER_EVENT = 'as-navigation-drawer';

/** Opens (`true`) or closes (`false`) the menu drawer. */
export function setNavigationDrawerOpen(open: boolean): void {
  window.dispatchEvent(new CustomEvent(NAVIGATION_DRAWER_EVENT, { detail: { open } }));
}

/** Whether the menu drawer is open. It starts closed on every page load. */
export function useNavigationDrawerOpen(): boolean {
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    const onChange = (event: Event) => {
      setIsOpen(Boolean((event as CustomEvent<{ open?: boolean }>).detail?.open));
    };
    window.addEventListener(NAVIGATION_DRAWER_EVENT, onChange);
    return () => window.removeEventListener(NAVIGATION_DRAWER_EVENT, onChange);
  }, []);

  return isOpen;
}

/** Return value of {@link useNavigationDrawer}. */
export interface NavigationDrawer {
  /** Whether the menu is a drawer (Astrum One narrower than a desktop) instead of the side rail. */
  isDrawer: boolean;
  /** Whether the drawer is open. */
  isDrawerOpen: boolean;
  /** Closes the drawer. */
  closeDrawer: () => void;
  /** Closes the drawer when one of its links was picked. */
  onDrawerClick: (event: MouseEvent<HTMLElement>) => void;
  /** The drawer element: focus moves into it when it opens and stays there while it is open. */
  drawerRef: RefObject<HTMLElement>;
}

/**
 * The menu drawer of Astrum One on phones and tablets. It closes after a pick, on Esc, on a tap on the scrim, after
 * any navigation and when the viewport grows to a desktop, and gives the focus back to where it was opened from.
 */
export function useNavigationDrawer(isPreviewUI: boolean): NavigationDrawer {
  const isCompact = useIsCompactViewport();
  const isDrawer = isPreviewUI && isCompact;
  const isOpen = useNavigationDrawerOpen();
  const isDrawerOpen = isDrawer && isOpen;
  const drawerRef = useRef<HTMLElement>(null);

  const closeDrawer = useCallback(() => setNavigationDrawerOpen(false), []);

  const onDrawerClick = useCallback(
    (event: MouseEvent<HTMLElement>) => {
      if ((event.target as Element).closest?.('a[href]')) {
        closeDrawer();
      }
    },
    [closeDrawer]
  );

  useTransitionHook('onSuccess', {}, () => {
    closeDrawer();
  });

  useEffect(() => {
    if (isOpen && !isDrawer) {
      closeDrawer();
    }
  }, [isOpen, isDrawer, closeDrawer]);

  useEffect(() => {
    if (!isDrawerOpen) {
      return undefined;
    }
    const openedFrom = document.activeElement as HTMLElement | null;
    drawerRef.current?.querySelector<HTMLElement>('[data-drawer-autofocus]')?.focus();

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !event.defaultPrevented) {
        closeDrawer();
        return;
      }
      trapTabKey(event, drawerRef.current);
    };
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
      openedFrom?.focus?.();
    };
  }, [isDrawerOpen, closeDrawer]);

  return { isDrawer, isDrawerOpen, closeDrawer, onDrawerClick, drawerRef };
}
