

import { renderHook, act } from '@testing-library/react';
import { useIsPreviewUI } from './useIsPreviewUI';

describe('useIsPreviewUI', () => {
  beforeEach(() => {
    // Reset hash before each test
    window.location.hash = '';
  });

  afterEach(() => {
    window.location.hash = '';
  });

  it('returns false when hash does not start with #preview', () => {
    window.location.hash = '#browse/welcome';
    const { result } = renderHook(() => useIsPreviewUI());
    expect(result.current).toBe(false);
  });

  it('returns true when hash starts with #preview', () => {
    window.location.hash = '#preview/browse/welcome';
    const { result } = renderHook(() => useIsPreviewUI());
    expect(result.current).toBe(true);
  });

  it('returns false for empty hash', () => {
    window.location.hash = '';
    const { result } = renderHook(() => useIsPreviewUI());
    expect(result.current).toBe(false);
  });

  it('updates when hash changes to preview', () => {
    window.location.hash = '#browse/welcome';
    const { result } = renderHook(() => useIsPreviewUI());
    expect(result.current).toBe(false);

    act(() => {
      window.location.hash = '#preview/browse/welcome';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
    });

    expect(result.current).toBe(true);
  });

  it('updates when hash changes away from preview', () => {
    window.location.hash = '#preview/browse/welcome';
    const { result } = renderHook(() => useIsPreviewUI());
    expect(result.current).toBe(true);

    act(() => {
      window.location.hash = '#browse/welcome';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
    });

    expect(result.current).toBe(false);
  });

  it('cleans up event listener on unmount', () => {
    const addEventListenerSpy = jest.spyOn(window, 'addEventListener');
    const removeEventListenerSpy = jest.spyOn(window, 'removeEventListener');

    const { unmount } = renderHook(() => useIsPreviewUI());

    expect(addEventListenerSpy).toHaveBeenCalledWith('hashchange', expect.any(Function));

    unmount();

    expect(removeEventListenerSpy).toHaveBeenCalledWith('hashchange', expect.any(Function));

    addEventListenerSpy.mockRestore();
    removeEventListenerSpy.mockRestore();
  });
});