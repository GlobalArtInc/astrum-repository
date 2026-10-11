import { renderHook, act } from '@testing-library/react';
import { useIsPreviewUI } from './useIsPreviewUI';

describe('useIsPreviewUI', () => {
  afterEach(() => {
    window.location.hash = '';
  });

  it.each([
    ['#browse/welcome'],
    ['#admin/security/users'],
    ['#settings'],
    ['#browse/welcome'],
    [''],
  ])('returns true on %p: Astrum One is the only UI', (hash) => {
    window.location.hash = hash;
    const { result } = renderHook(() => useIsPreviewUI());
    expect(result.current).toBe(true);
  });

  it('stays true when the hash changes', () => {
    window.location.hash = '#browse/welcome';
    const { result } = renderHook(() => useIsPreviewUI());

    act(() => {
      window.location.hash = '#admin/system/tasks';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
    });

    expect(result.current).toBe(true);
  });
});
