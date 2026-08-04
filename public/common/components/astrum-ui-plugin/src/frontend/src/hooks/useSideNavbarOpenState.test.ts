
import { act, renderHook } from '@testing-library/react';
import { useSideNavbarOpenState } from './useSideNavbarOpenState';

describe('useSideNavbarOpenState', () => {
  const setup = (initialOpen = true) => renderHook(() => useSideNavbarOpenState(initialOpen));

  it('should initialize correctly', () => {
    const { result } = setup(true);
    const [isOpen] = result.current;
    expect(isOpen).toBe(true);
  });

  it('should toggle open state with onToggleClick', () => {
    const { result } = setup(true);
    const [, onToggleClick] = result.current;

    act(() => {
      onToggleClick();
    });

    const [isOpen] = result.current;
    expect(isOpen).toBe(false);
  });

  describe('nx-sidebar-toggle window event', () => {
    it('should toggle isOpen when nx-sidebar-toggle event is dispatched without detail', () => {
      const { result } = setup(true);

      act(() => {
        window.dispatchEvent(new CustomEvent('nx-sidebar-toggle'));
      });

      const [isOpen] = result.current;
      expect(isOpen).toBe(false);
    });

    it('should set isOpen to true when nx-sidebar-toggle event has detail.open=true', () => {
      const { result } = setup(false);

      act(() => {
        window.dispatchEvent(new CustomEvent('nx-sidebar-toggle', { detail: { open: true } }));
      });

      const [isOpen] = result.current;
      expect(isOpen).toBe(true);
    });

    it('should set isOpen to false when nx-sidebar-toggle event has detail.open=false', () => {
      const { result } = setup(true);

      act(() => {
        window.dispatchEvent(new CustomEvent('nx-sidebar-toggle', { detail: { open: false } }));
      });

      const [isOpen] = result.current;
      expect(isOpen).toBe(false);
    });

    it('should remove the event listener on unmount', () => {
      const { result, unmount } = setup(true);

      unmount();

      expect(() => {
        window.dispatchEvent(new CustomEvent('nx-sidebar-toggle'));
      }).not.toThrow();

      const [isOpen] = result.current;
      expect(isOpen).toBe(true);
    });
  });
});
