import { useEffect, useMemo } from 'react';
import { useSession } from '../auth/state';
import { replaceTitleSuffix } from '../ui/hooks';
import styles from './Shell.module.css';
import { sanitizeHtml } from './sanitize';

export function BrandingSnippet({
  html,
  place,
}: {
  html: string | null | undefined;
  place: 'header' | 'footer';
}) {
  const safe = useMemo(() => (html ? sanitizeHtml(html) : ''), [html]);
  if (!safe) {
    return null;
  }
  return (
    <div
      className={place === 'header' ? styles.brandingHeader : styles.brandingFooter}
      data-testid={`branding-${place}`}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: the snippet is sanitized against an allow list first
      dangerouslySetInnerHTML={{ __html: safe }}
    />
  );
}

export function useConfiguredTitle(): void {
  const { state } = useSession();
  const title = state.uiSettings?.title;
  useEffect(() => {
    replaceTitleSuffix(title);
  }, [title]);
}
