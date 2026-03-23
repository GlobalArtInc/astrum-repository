import { CircleAlert, X } from 'lucide-react';
import { useEffect, useRef } from 'react';
import { useT } from '../../../i18n';
import styles from './SaveError.module.css';

export function SaveError({ message, onDismiss }: { message: string | null; onDismiss(): void }) {
  const t = useT();
  const wrap = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (message) {
      wrap.current?.scrollIntoView?.({ block: 'nearest' });
    }
  }, [message]);
  if (!message) {
    return null;
  }
  return (
    <div className={styles.wrap} ref={wrap}>
      <div className={styles.alert} role="alert">
        <CircleAlert size={18} className={styles.icon} aria-hidden="true" />
        <div className={styles.content}>{message}</div>
        <button
          type="button"
          className={styles.close}
          aria-label={t('repositories.create.dismissError')}
          onClick={onDismiss}
        >
          <X size={16} aria-hidden="true" />
        </button>
      </div>
    </div>
  );
}

export function FooterError({ message }: { message: string | null }) {
  if (!message) {
    return null;
  }
  return (
    <div className={styles.footer} role="alert">
      <CircleAlert size={14} aria-hidden="true" />
      <span>{message}</span>
    </div>
  );
}
