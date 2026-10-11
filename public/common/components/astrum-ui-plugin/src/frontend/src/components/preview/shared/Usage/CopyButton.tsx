import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Button, IconButton, Tooltip } from '@radix-ui/themes';
import { Check, Copy } from 'lucide-react';

import UIStrings from '../../constants/UIStrings';
import { copyToClipboard } from './clipboard';

/** How long the button says "Copied" after a successful copy. */
export const COPIED_FEEDBACK_MS = 1500;

export interface CopyButtonProps {
  /** The text to copy. */
  text: string;
  /** Accessible name; defaults to the visible label. */
  label?: string;
  /** Show only the icon (with a tooltip) instead of icon and label. */
  iconOnly?: boolean;
  /** Radix button size. */
  size?: '1' | '2';
  /** Called after the text was copied. */
  onCopied?: () => void;
  /** Called when the text could not be copied. */
  onCopyFailed?: () => void;
  className?: string;
}

/**
 * A button that copies a text and says "Copied" for a moment afterwards.
 */
export function CopyButton({
  text,
  label,
  iconOnly = false,
  size = '2',
  onCopied,
  onCopyFailed,
  className,
}: CopyButtonProps): JSX.Element {
  const strings = UIStrings.BROWSE.USAGE;
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => () => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }
  }, []);

  useEffect(() => {
    setCopied(false);
  }, [text]);

  const handleClick = useCallback(async (event: React.MouseEvent) => {
    event.stopPropagation();
    const ok = await copyToClipboard(text);
    if (!ok) {
      onCopyFailed?.();
      return;
    }
    setCopied(true);
    onCopied?.();
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }
    timerRef.current = setTimeout(() => setCopied(false), COPIED_FEEDBACK_MS);
  }, [text, onCopied, onCopyFailed]);

  const visible = copied ? strings.COPIED : strings.COPY;
  const icon = copied ? <Check size={14} aria-hidden /> : <Copy size={14} aria-hidden />;

  if (iconOnly) {
    return (
      <Tooltip content={visible}>
        <IconButton
          type="button"
          variant="ghost"
          color="gray"
          size={size}
          onClick={handleClick}
          aria-label={label ?? visible}
          data-copied={copied ? 'true' : 'false'}
          className={className}
        >
          {icon}
        </IconButton>
      </Tooltip>
    );
  }

  return (
    <Button
      type="button"
      variant="surface"
      color="gray"
      highContrast
      size={size}
      onClick={handleClick}
      aria-label={label}
      data-copied={copied ? 'true' : 'false'}
      className={className}
    >
      {icon}
      <span aria-live="polite">{visible}</span>
    </Button>
  );
}

export default CopyButton;
