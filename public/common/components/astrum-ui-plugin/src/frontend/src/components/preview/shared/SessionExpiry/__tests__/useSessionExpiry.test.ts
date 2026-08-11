
import {renderHook, act} from '@testing-library/react';
import {useSessionExpiry} from '../useSessionExpiry';
import {ASTRUM_SESSION_EXPIRED_EVENT} from '../../../../../interface/api/sessionExpiryBroadcast';

const DEFAULT_MESSAGE = 'Your session has expired. Please sign in again to continue.';

describe('useSessionExpiry', () => {
  it('starts with isExpired=false and the default message', () => {
    const {result} = renderHook(() => useSessionExpiry());
    expect(result.current.isExpired).toBe(false);
    expect(result.current.message).toBe(DEFAULT_MESSAGE);
  });

  it('showExpiryModal sets isExpired=true', () => {
    const {result} = renderHook(() => useSessionExpiry());
    act(() => result.current.showExpiryModal());
    expect(result.current.isExpired).toBe(true);
  });

  it('showExpiryModal uses a custom message when provided', () => {
    const {result} = renderHook(() => useSessionExpiry());
    act(() => result.current.showExpiryModal('Token expired'));
    expect(result.current.message).toBe('Token expired');
  });

  it('showExpiryModal is idempotent — second call is ignored', () => {
    const {result} = renderHook(() => useSessionExpiry());
    act(() => result.current.showExpiryModal('first'));
    act(() => result.current.showExpiryModal('second'));
    expect(result.current.message).toBe('first');
  });

  it('hideExpiryModal sets isExpired=false', () => {
    const {result} = renderHook(() => useSessionExpiry());
    act(() => result.current.showExpiryModal());
    act(() => result.current.hideExpiryModal());
    expect(result.current.isExpired).toBe(false);
  });

  it('hideExpiryModal resets the dedup guard so modal can show again', () => {
    const {result} = renderHook(() => useSessionExpiry());
    act(() => result.current.showExpiryModal('first'));
    act(() => result.current.hideExpiryModal());
    act(() => result.current.showExpiryModal('second'));
    expect(result.current.isExpired).toBe(true);
    expect(result.current.message).toBe('second');
  });

  describe('checkSessionError', () => {
    it('returns false for a non-401 status', () => {
      const {result} = renderHook(() => useSessionExpiry());
      let out: boolean;
      act(() => {
        out = result.current.checkSessionError({status: 500});
      });
      expect(out!).toBe(false);
      expect(result.current.isExpired).toBe(false);
    });

    it('returns true and shows modal for status=401', () => {
      const {result} = renderHook(() => useSessionExpiry());
      let out: boolean;
      act(() => {
        out = result.current.checkSessionError({status: 401});
      });
      expect(out!).toBe(true);
      expect(result.current.isExpired).toBe(true);
    });

    it('returns true for a nested response.status=401', () => {
      const {result} = renderHook(() => useSessionExpiry());
      let out: boolean;
      act(() => {
        out = result.current.checkSessionError({response: {status: 401}});
      });
      expect(out!).toBe(true);
    });

    it('returns false for null', () => {
      const {result} = renderHook(() => useSessionExpiry());
      let out: boolean;
      act(() => {
        out = result.current.checkSessionError(null);
      });
      expect(out!).toBe(false);
    });
  });

  describe('event listeners', () => {
    it('shows modal when the ASTRUM_SESSION_EXPIRED_EVENT is dispatched', () => {
      const {result} = renderHook(() => useSessionExpiry());
      act(() => {
        window.dispatchEvent(new CustomEvent(ASTRUM_SESSION_EXPIRED_EVENT));
      });
      expect(result.current.isExpired).toBe(true);
    });

    it('removes the session-expired listener on unmount', () => {
      const {result, unmount} = renderHook(() => useSessionExpiry());
      unmount();
      act(() => {
        window.dispatchEvent(new CustomEvent(ASTRUM_SESSION_EXPIRED_EVENT));
      });
      expect(result.current.isExpired).toBe(false);
    });
  });
});
