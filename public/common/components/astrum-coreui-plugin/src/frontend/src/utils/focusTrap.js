const FOCUSABLE = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])'
].join(',');

/**
 * Keeps Tab and Shift+Tab inside a layer drawn over the page (the menu drawer, the full-screen search), so that
 * keyboard focus cannot wander to the page hidden behind it. Call it from a keydown listener.
 */
export function trapTabKey(event, container) {
  if (event.key !== 'Tab' || event.defaultPrevented || !container) {
    return;
  }
  const focusable = Array.from(container.querySelectorAll(FOCUSABLE))
      .filter((element) => element.getClientRects().length > 0);
  if (focusable.length === 0) {
    return;
  }
  const first = focusable[0];
  const last = focusable[focusable.length - 1];
  const active = document.activeElement;

  if (!container.contains(active)) {
    event.preventDefault();
    first.focus();
  }
  else if (event.shiftKey && active === first) {
    event.preventDefault();
    last.focus();
  }
  else if (!event.shiftKey && active === last) {
    event.preventDefault();
    first.focus();
  }
}
