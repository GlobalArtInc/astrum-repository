

import React from 'react';
import { Flex, Spinner, Text } from '@radix-ui/themes';

import './LoadingState.scss';
import UIStrings from '../../../constants/UIStrings';

export interface LoadingStateProps {
  /** Optional message to display */
  message?: string;
  /** Size variant */
  size?: 'small' | 'medium' | 'large';
  /** Whether to display inline (no centering) */
  inline?: boolean;
  /** Custom class name */
  className?: string;
}

/**
 * LoadingState displays a spinner with optional message.
 *
 * Features:
 * - Three size variants
 * - Optional descriptive message
 * - Inline or centered display
 * - Accessible loading indication
 *
 * @example
 * ```tsx
 * <LoadingState message="Loading repositories..." />
 * <LoadingState size="small" inline />
 * <LoadingState size="large" message="Please wait..." />
 * ```
 */
export function LoadingState({
  message,
  size = 'medium',
  inline = false,
  className = '',
}: LoadingStateProps): JSX.Element {
  const spinnerSize = size === 'large' ? '3' : size === 'small' ? '1' : '2';
  const textSize = size === 'large' ? '3' : size === 'small' ? '1' : '2';
  const sizeClass = `loading-state--${size}`;
  const inlineClass = inline ? 'loading-state--inline' : '';

  return (
    <Flex
      direction="column"
      align="center"
      justify="center"
      gap="3"
      className={`loading-state ${sizeClass} ${inlineClass} ${className}`}
      data-testid="loading-state"
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <Spinner size={spinnerSize} />
      {message && (
        <Text size={textSize} color="gray" className="loading-state__message">
          {message}
        </Text>
      )}
      {/* Screen reader text */}
      <span className="sr-only">{UIStrings.SHELL.LOADING}{message ? `: ${message}` : ''}</span>
    </Flex>
  );
}

export default LoadingState;


