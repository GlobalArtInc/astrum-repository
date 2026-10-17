import { Text } from '@radix-ui/themes';
import { CircleAlert, CircleCheck, ExternalLink, Info } from 'lucide-react';
import { type ReactNode, useId } from 'react';
import styles from './Outbound.module.css';

export function AboutCard({
  title,
  children,
  accent,
}: {
  title: string;
  children: ReactNode;
  accent?: boolean;
}) {
  const id = useId();
  return (
    <section className={accent ? `${styles.about} ${styles.aboutAccent}` : styles.about} aria-labelledby={id}>
      <div className={styles.aboutHead}>
        <Info size={16} aria-hidden="true" className={styles.aboutIcon} />
        <span id={id} className={styles.aboutTitle}>
          {title}
        </span>
      </div>
      {children}
    </section>
  );
}

export function DocsLink({ href, children }: { href: string; children: ReactNode }) {
  return (
    <a href={href} target="_blank" rel="noopener noreferrer" className={styles.docs}>
      {children}
      <ExternalLink size={12} aria-hidden="true" />
    </a>
  );
}

export function Collapsible({
  title,
  open,
  onToggle,
  children,
}: {
  title: string;
  open: boolean;
  onToggle(): void;
  children: ReactNode;
}) {
  const id = useId();
  return (
    <div className={styles.collapsible}>
      <button
        type="button"
        className={styles.collapsibleHead}
        aria-expanded={open}
        aria-controls={id}
        onClick={onToggle}
      >
        <span className={open ? styles.chevronOpen : styles.chevron} aria-hidden="true">
          ▾
        </span>
        {title}
      </button>
      {open && (
        <div id={id} className={styles.collapsibleBody}>
          {children}
        </div>
      )}
    </div>
  );
}

export function ResultBox({ tone, children }: { tone: 'success' | 'error'; children: ReactNode }) {
  return (
    <div
      className={
        tone === 'success' ? `${styles.result} ${styles.success}` : `${styles.result} ${styles.failure}`
      }
      role={tone === 'success' ? 'status' : 'alert'}
    >
      {tone === 'success' ? (
        <CircleCheck size={16} aria-hidden="true" />
      ) : (
        <CircleAlert size={16} aria-hidden="true" />
      )}
      <Text size="2">{children}</Text>
    </div>
  );
}
