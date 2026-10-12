import { act, renderHook } from '@testing-library/react';

const mockIsCompact = jest.fn(() => true);
let transitionSuccess: (() => void) | undefined;

jest.mock('@globalart/astrum-ui-plugin', () => ({
  useIsCompactViewport: () => mockIsCompact(),
}));

jest.mock('@uirouter/react', () => ({
  useTransitionHook: (_hook: string, _criteria: unknown, callback: () => void) => {
    transitionSuccess = callback;
  },
}));

import { setNavigationDrawerOpen, useNavigationDrawer } from './navigationDrawer';

describe('useNavigationDrawer', () => {
  beforeEach(() => {
    mockIsCompact.mockReturnValue(true);
  });

  function open() {
    act(() => setNavigationDrawerOpen(true));
  }

  it('is a drawer narrower than a desktop only', () => {
    expect(renderHook(() => useNavigationDrawer()).result.current.isDrawer).toBe(true);

    mockIsCompact.mockReturnValue(false);
    expect(renderHook(() => useNavigationDrawer()).result.current.isDrawer).toBe(false);
  });

  it('starts closed and opens on request', () => {
    const { result } = renderHook(() => useNavigationDrawer());
    expect(result.current.isDrawerOpen).toBe(false);

    open();
    expect(result.current.isDrawerOpen).toBe(true);
  });

  it('closes on Esc', () => {
    const { result } = renderHook(() => useNavigationDrawer());
    open();

    act(() => {
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    });

    expect(result.current.isDrawerOpen).toBe(false);
  });

  it('closes after a navigation', () => {
    const { result } = renderHook(() => useNavigationDrawer());
    open();

    act(() => transitionSuccess?.());

    expect(result.current.isDrawerOpen).toBe(false);
  });

  it('closes when one of its links is picked, not on other clicks', () => {
    const { result } = renderHook(() => useNavigationDrawer());
    open();
    const link = document.createElement('a');
    link.href = '#browse/browse';
    const label = document.createElement('span');
    link.appendChild(label);

    act(() => result.current.onDrawerClick({ target: document.createElement('div') } as never));
    expect(result.current.isDrawerOpen).toBe(true);

    act(() => result.current.onDrawerClick({ target: label } as never));
    expect(result.current.isDrawerOpen).toBe(false);
  });

  it('closes when the viewport grows to a desktop', () => {
    const { result, rerender } = renderHook(() => useNavigationDrawer());
    open();

    mockIsCompact.mockReturnValue(false);
    rerender();
    mockIsCompact.mockReturnValue(true);
    rerender();

    expect(result.current.isDrawerOpen).toBe(false);
  });
});
