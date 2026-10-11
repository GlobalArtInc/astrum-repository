

import React from 'react';
import { Dialog, Flex, Text, Button, Box, Heading } from '@radix-ui/themes';
import { AlertCircle, LogIn, RefreshCw } from 'lucide-react';
import { useRouter } from '@uirouter/react';
import SharedStrings from '../../constants/SharedStrings';

export interface SessionExpiryModalProps {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Callback when modal is closed */
  onClose: () => void;
  /** Optional message to display */
  message?: string;
}

/**
 * SessionExpiryModal - User-friendly modal for session expiration handling.
 *
 * Features:
 * - Clear messaging about session expiration
 * - Options to re-authenticate or refresh
 * - Modern Radix UI design matching Preview UI
 */
export function SessionExpiryModal({
  isOpen,
  onClose,
  message = SharedStrings.SESSION_EXPIRY.SESSION_HAS_EXPIRED_PLEASE_SIGN,
}: SessionExpiryModalProps): JSX.Element {
  const router = useRouter();

  const handleSignIn = () => {
    // Get current URL
    const currentUrl = window.location.hash;

    // Don't return to test hub or other internal routes after login
    // These routes require debug mode and special localStorage flags
    const isInternalRoute = /^#\/?(?:preview\/)?test(?:[-/?]|$)/.test(currentUrl || '');

    // Store URL for redirect after login (but not for internal routes)
    if (currentUrl && !isInternalRoute) {
      sessionStorage.setItem('astrum:returnTo', currentUrl);
    } else {
      // Clear any previous return path for internal routes
      sessionStorage.removeItem('astrum:returnTo');
    }

    // Redirect to login page
    // For internal routes, return to welcome page after login
    const returnTo = isInternalRoute ? btoa('#browse/welcome') : btoa(currentUrl || '#/');
    router.stateService.go('login', { returnTo });
    onClose();
  };

  const handleRefresh = () => {
    // Attempt to refresh the page - this may redirect to login if session is truly expired
    window.location.reload();
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Content
        maxWidth="450px"
        data-testid="session-expiry-modal"
      >
        <Flex direction="column" gap="4">
          {/* Header */}
          <Flex align="center" gap="3">
            <Box
              style={{
                background: 'var(--orange-3)',
                borderRadius: 'var(--radius-3)',
                padding: 'var(--space-3)',
              }}
            >
              <AlertCircle size={28} color="var(--orange-9)" />
            </Box>
            <Box>
              <Heading size="4">{SharedStrings.SESSION_EXPIRY.SESSION_EXPIRED}</Heading>
              <Text size="2" color="gray">
                {SharedStrings.SESSION_EXPIRY.AUTHENTICATION_REQUIRED}
              </Text>
            </Box>
          </Flex>

          {/* Message */}
          <Text size="2" color="gray">
            {message}
          </Text>

          {/* Actions */}
          <Flex gap="3" justify="end" mt="2">
            <Button variant="soft" color="gray" onClick={handleRefresh}>
              <RefreshCw size={16} />
              {SharedStrings.SESSION_EXPIRY.REFRESH_PAGE}
            </Button>
            <Button variant="solid" onClick={handleSignIn}>
              <LogIn size={16} />
              {SharedStrings.SESSION_EXPIRY.SIGN_IN}
            </Button>
          </Flex>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

export default SessionExpiryModal;
