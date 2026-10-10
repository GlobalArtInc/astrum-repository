
import { act, renderHook } from '@testing-library/react';
import { SIDEBAR_OPEN_STORAGE_KEY, useSideNavbarOpenState } from './useSideNavbarOpenState';

describe('useSideNavbarOpenState', () => {
  const setup = (initialOpen = true) => renderHook(() => useSideNavbarOpenState(initialOpen));

  beforeEach(() => {
    window.localStorage.clear();
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  describe('remembering the choice of the user', () => {
    it('restores the stored state instead of the initial one', () => {
      window.localStorage.setItem(SIDEBAR_OPEN_STORAGE_KEY, 'true');

      const { result } = setup(false);

      expect(result.current[0]).toBe(true);
    });

    it('stores the state chosen with onToggleClick', () => {
      const { result } = setup(false);

      act(() => {
        result.current[1]();
      });

      expect(window.localStorage.getItem(SIDEBAR_OPEN_STORAGE_KEY)).toBe('true');
      expect(setup(false).result.current[0]).toBe(true);
    });

    it('stores the state chosen with the toggle event', () => {
      const header = setup(false);
      const menu = setup(false);

      act(() => {
        window.dispatchEvent(new CustomEvent('as-sidebar-toggle'));
      });

      expect(header.result.current[0]).toBe(true);
      expect(menu.result.current[0]).toBe(true);
      expect(window.localStorage.getItem(SIDEBAR_OPEN_STORAGE_KEY)).toBe('true');
    });

    it('does not store a state forced by a page', () => {
      const { result } = setup(true);

      act(() => {
        window.dispatchEvent(new CustomEvent('as-sidebar-toggle', { detail: { open: false } }));
      });

      expect(result.current[0]).toBe(false);
      expect(window.localStorage.getItem(SIDEBAR_OPEN_STORAGE_KEY)).toBeNull();
    });

    it('works without storage', () => {
      jest.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
        throw new Error('denied');
      });
      jest.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
        throw new Error('denied');
      });
      const { result } = setup(true);

      act(() => {
        result.current[1]();
      });

      expect(result.current[0]).toBe(false);
    });
  });

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

  describe('as-sidebar-toggle window event', () => {
    it('should toggle isOpen when as-sidebar-toggle event is dispatched without detail', () => {
      const { result } = setup(true);

      act(() => {
        window.dispatchEvent(new CustomEvent('as-sidebar-toggle'));
      });

      const [isOpen] = result.current;
      expect(isOpen).toBe(false);
    });

    it('should set isOpen to true when as-sidebar-toggle event has detail.open=true', () => {
      const { result } = setup(false);

      act(() => {
        window.dispatchEvent(new CustomEvent('as-sidebar-toggle', { detail: { open: true } }));
      });

      const [isOpen] = result.current;
      expect(isOpen).toBe(true);
    });

    it('should set isOpen to false when as-sidebar-toggle event has detail.open=false', () => {
      const { result } = setup(true);

      act(() => {
        window.dispatchEvent(new CustomEvent('as-sidebar-toggle', { detail: { open: false } }));
      });

      const [isOpen] = result.current;
      expect(isOpen).toBe(false);
    });

    it('should remove the event listener on unmount', () => {
      const { result, unmount } = setup(true);

      unmount();

      expect(() => {
        window.dispatchEvent(new CustomEvent('as-sidebar-toggle'));
      }).not.toThrow();

      const [isOpen] = result.current;
      expect(isOpen).toBe(true);
    });
  });
});
