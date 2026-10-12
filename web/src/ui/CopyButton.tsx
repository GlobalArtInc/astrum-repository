import { IconButton, Tooltip } from '@radix-ui/themes';
import { Check, Copy } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useT } from '../i18n';
import { copyText } from './copy';
import { useToast } from './Toaster';

export interface CopyButtonProps {
  text: string;
  label?: string;
  size?: '1' | '2';
}

export function CopyButton({ text, label, size = '1' }: CopyButtonProps) {
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
