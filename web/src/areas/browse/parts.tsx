import { Text } from '@radix-ui/themes';
import type { ReactNode } from 'react';
import { useT } from '../../i18n';
import { CopyButton } from '../../ui';
import styles from './parts.module.css';

export function OnlineStatus({ online }: { online: boolean }) {
  const t = useT();
  return (
    <span className={styles.status} data-online={online}>
      <span className={styles.dot} aria-hidden="true" />
      {online ? t('browse.status.online') : t('browse.status.offline')}
    </span>
  );
}

export function UrlLine({ url, label, shown }: { url: string; label: string; shown?: string }) {
  return (
    <span className={styles.url}>
      <code className={styles.urlText} title={url}>
        {shown ?? url}
      </code>
      <CopyButton text={url} label={label} />
    </span>
  );
}

export interface DetailItem {
  id: string;
  label: string;
  value: ReactNode;
  mono?: boolean;
}

export function DetailList({ items, label }: { items: DetailItem[]; label?: string }) {
  return (
    <dl className={styles.details} aria-label={label}>
      {items.map((item) => (
        <div key={item.id} className={styles.detailRow}>
          <dt>{item.label}</dt>
          <dd className={item.mono ? styles.mono : undefined}>{item.value}</dd>
        </div>
      ))}
    </dl>
  );
}

export function Section({
  title,
  actions,
  children,
  id,
}: {
  title: string;
  actions?: ReactNode;
  children: ReactNode;
  id?: string;
}) {
  return (
    <section className={styles.section} aria-labelledby={id}>
      <div className={styles.sectionHead}>
        <Text as="p" size="3" weight="bold" id={id}>
          {title}
        </Text>
        {actions}
      </div>
      {children}
    </section>
  );
}

function plain(value: unknown): string {
  if (value === null || value === undefined) {
    return '';
  }
  if (typeof value === 'object') {
    return JSON.stringify(value, null, 2);
  }
  return String(value);
}

export function AttributeList({ attributes, label }: { attributes: Record<string, unknown>; label: string }) {
  const entries = Object.entries(attributes).filter(([, value]) => plain(value) !== '');
  if (entries.length === 0) {
    return null;
  }
  return (
    <DetailList
      label={label}
      items={entries.map(([key, value]) => ({
        id: key,
        label: key,
        mono: true,
        value: typeof value === 'object' ? <pre className={styles.pre}>{plain(value)}</pre> : plain(value),
      }))}
    />
  );
}
