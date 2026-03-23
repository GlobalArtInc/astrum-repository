import { CircleHelp, ExternalLink } from 'lucide-react';
import type { ReactNode } from 'react';
import { useT } from '../../../../i18n';
import { DOCUMENTATION_URL } from '../../../../shell/HeaderMenus';
import styles from './admin.module.css';

export const DOCS_URL = DOCUMENTATION_URL;

export function HelpCard({
  title,
  children,
  href = DOCS_URL,
}: {
  title: string;
  children: ReactNode;
  href?: string;
}) {
  const t = useT();
  return (
    <section className={styles.help} aria-label={title}>
      <span className={styles.helpTitle}>
        <CircleHelp size={14} aria-hidden="true" />
        {title}
      </span>
      <p className={styles.helpText}>{children}</p>
      <a className={styles.helpLink} href={href} target="_blank" rel="noreferrer">
        {t('repositories.admin.documentation')}
        <ExternalLink size={12} aria-hidden="true" />
      </a>
    </section>
  );
}
