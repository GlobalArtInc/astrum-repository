import { Button, IconButton } from '@radix-ui/themes';
import { Check, Copy } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useT } from '../i18n';
import { copyText } from './copy';
import { useToast } from './Toaster';
import { Tooltip } from './Tooltip';

export interface CopyButtonProps {
  text: string;
  label?: string;
  size?: '1' | '2';
  variant?: 'icon' | 'text';
}

export function CopyButton({ text, label, size = '1', variant = 'icon' }: CopyButtonProps) {
  const t = useT();
  const toast = useToast();
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = window.setTimeout(() => setCopied(false), 1500);
    return () => window.clearTimeout(timer);
  }, [copied]);
  const name = label ?? t('ui.copy.label');
  const copy = async () => {
    try {
      await copyText(text);
      setCopied(true);
    } catch {
      toast.error(t('ui.copy.failed'));
    }
  };
  if (variant === 'text') {
    return (
      <Button
        type="button"
        size="2"
        variant="surface"
        color="gray"
        highContrast
        aria-label={name}
        onClick={copy}
      >
        {copied ? <Check size={16} aria-hidden="true" /> : <Copy size={16} aria-hidden="true" />}
        {copied ? t('ui.copy.copied') : t('ui.copy.button')}
      </Button>
    );
  }
  return (
    <Tooltip content={copied ? t('ui.copy.copied') : name}>
      <IconButton type="button" size={size} variant="ghost" color="gray" aria-label={name} onClick={copy}>
        {copied ? <Check size={14} aria-hidden="true" /> : <Copy size={14} aria-hidden="true" />}
        <span className="visually-hidden" aria-live="polite">
          {copied ? t('ui.copy.copied') : ''}
        </span>
      </IconButton>
    </Tooltip>
  );
}
