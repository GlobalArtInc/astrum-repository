

import React from 'react';
import { Box, Button, Callout, Flex, Text } from '@radix-ui/themes';
import { AlertCircle, RefreshCw } from 'lucide-react';

import './ErrorState.scss';

export interface ErrorStateProps {
  /** Error title (defaults to "Something went wrong") */
  title?: string;
  /** Error message */
  message: string;
  /** Optional retry callback */
  onRetry?: () => void;
  /** Retry button text */
  retryText?: string;
  /** Size variant */
  size?: 'small' | 'medium' | 'large';
  /** Use inline callout style */
  variant?: 'inline' | 'centered';
  /** Custom class name */
  className?: string;
}

/**
 * ErrorState displays an error message with optional retry button.
 *
 * Features:
 * - Clear error messaging
 * - Optional retry functionality
 * - Two display variants: inline callout or centered block
 * - Accessible error indication
 *
 * @example
 * ```tsx
 * <ErrorState
 *   title="Failed to load repositories"
 *   message="Network connection lost. Please check your connection."
 *   onRetry={() => refetch()}
 * />
 *
 * <ErrorState
 *   variant="inline"
 *   message="Invalid form data"
 * />
 * ```
 */
export function ErrorState({
  title = 'Something went wrong',
  message,
  onRetry,
  retryText = 'Try again',
  size = 'medium',
  variant = 'centered',
  className = '',
}: ErrorStateProps): JSX.Element {
  const sizeClass = `error-state--${size}`;

  // Inline variant uses Radix Callout
  if (variant === 'inline') {
    return (
      <Box className={`error-state error-state--inline ${className}`} data-testid="error-state">
        <Callout.Root color="red" role="alert">
          <Callout.Icon>
            <AlertCircle size={16} />
          </Callout.Icon>
          <Callout.Text>
            {title !== 'Something went wrong' && <strong>{title}: </strong>}
            {message}
            {onRetry && (
              <Button
                variant="ghost"
                size="1"
                onClick={onRetry}
                className="error-state__inline-retry"
              >
                <RefreshCw size={14} />
                {retryText}
              </Button>
            )}
          </Callout.Text>
        </Callout.Root>
      </Box>
    );
  }

  // Centered variant
  return (
    <Flex
      direction="column"
      align="center"
      justify="center"
      gap="4"
      className={`error-state ${sizeClass} ${className}`}
      data-testid="error-state"
      role="alert"
    >
      <Box className="error-state__icon">
        <AlertCircle size={size === 'large' ? 48 : size === 'small' ? 24 : 32} />
      </Box>

      <Text
        as="h3"
        size={size === 'large' ? '5' : size === 'small' ? '2' : '4'}
        weight="medium"
        className="error-state__title"
      >
        {title}
      </Text>

      <Text
        size={size === 'small' ? '1' : '2'}
        color="gray"
        align="center"
        className="error-state__message"
      >
        {message}
      </Text>

      {onRetry && (
        <Button
          variant="outline"
          size={size === 'small' ? '1' : '2'}
          onClick={onRetry}
          className="error-state__retry"
        >
          <RefreshCw size={14} />
          {retryText}
        </Button>
      )}
    </Flex>
  );
}

export default ErrorState;


