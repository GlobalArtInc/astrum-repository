

import { useState, useCallback, useEffect, useRef } from 'react';
import { ASTRUM_SESSION_EXPIRED_EVENT } from '../../../../interface/api/sessionExpiryBroadcast';
import SharedStrings from '../../constants/SharedStrings';

interface SessionExpiryState {
  isExpired: boolean;
  message: string;
}

export interface UseSessionExpiryReturn {
  /** Whether the session has expired */
  isExpired: boolean;
  /** Message to display in the modal */
  message: string;
  /** Show the session expiry modal */
  showExpiryModal: (message?: string) => void;
  /** Hide the session expiry modal */
  hideExpiryModal: () => void;
  /** Check if an error is a session expiry error */
  checkSessionError: (error: unknown) => boolean;
}

/**
 * Hook for managing session expiry state.
 *
 * Usage:
 * ```tsx
 * const { isExpired, message, showExpiryModal, hideExpiryModal } = useSessionExpiry();
 *
 * return (
 *   <>
 *     <YourApp />
 *     <SessionExpiryModal isOpen={isExpired} onClose={hideExpiryModal} message={message} />
 *   </>
 * );
 * ```
 */
export function useSessionExpiry(): UseSessionExpiryReturn {
  const [state, setState] = useState<SessionExpiryState>({
    isExpired: false,
    message: SharedStrings.SESSION_EXPIRY.SESSION_HAS_EXPIRED_PLEASE_SIGN,
  });

  // Ref to track if we've already shown the modal (to prevent multiple popups)
  const hasShownRef = useRef(false);

  const showExpiryModal = useCallback((message?: string) => {
    if (!hasShownRef.current) {
      hasShownRef.current = true;
      setState({
        isExpired: true,
        message: message || SharedStrings.SESSION_EXPIRY.SESSION_HAS_EXPIRED_PLEASE_SIGN,
      });
    }
  }, []);

  const hideExpiryModal = useCallback(() => {
    setState((prev) => ({ ...prev, isExpired: false }));
    // Reset the ref when modal is closed so we can show it again if needed
    hasShownRef.current = false;
  }, []);

  const checkSessionError = useCallback((error: unknown): boolean => {
    // Check if error is a 401 Unauthorized
    if (error && typeof error === 'object') {
      const err = error as { status?: number; response?: { status?: number } };
      const status = err.status || err.response?.status;

      if (status === 401) {
        showExpiryModal();
        return true;
      }
    }
    return false;
  }, [showExpiryModal]);

  // Axios 401 → CustomEvent from rest-client interceptors
  useEffect(() => {
    const handleRestUnauthorized = () => {
      showExpiryModal();
    };
    window.addEventListener(ASTRUM_SESSION_EXPIRED_EVENT, handleRestUnauthorized);
    return () => window.removeEventListener(ASTRUM_SESSION_EXPIRED_EVENT, handleRestUnauthorized);
  }, [showExpiryModal]);

  // Optional: uncaught errors that carry 401-shaped payloads
  useEffect(() => {
    const handleUnauthorized = (event: ErrorEvent) => {
      checkSessionError(event.error);
    };

    window.addEventListener('error', handleUnauthorized);
    return () => window.removeEventListener('error', handleUnauthorized);
  }, [checkSessionError]);

  return {
    isExpired: state.isExpired,
    message: state.message,
    showExpiryModal,
    hideExpiryModal,
    checkSessionError,
  };
}

export default useSessionExpiry;
