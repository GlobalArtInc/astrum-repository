import { Heading, Text } from '@radix-ui/themes';
import { ChevronRight } from 'lucide-react';
import { Fragment, type ReactNode } from 'react';
import { Link } from 'react-router';
import { useT } from '../i18n';
import { useDocumentTitle } from './hooks';
import styles from './Page.module.css';

export interface Crumb {
  label: string;
  to?: string;
}

export interface PageProps {
  title: string;
  description?: ReactNode;
  breadcrumbs?: Crumb[];
  actions?: ReactNode;
  documentTitle?: string;
  children?: ReactNode;
}

export function Breadcrumbs({ items }: { items: Crumb[] }) {
  const t = useT();
  return (
    <nav aria-label={t('ui.page.breadcrumbs')} className={styles.crumbs}>
      <ol>
        {items.map((item, index) => {
          const last = index === items.length - 1;
          return (
            <Fragment key={`${item.label}|${item.to ?? ''}`}>
              <li aria-current={last ? 'page' : undefined}>
                {item.to && !last ? <Link to={item.to}>{item.label}</Link> : item.label}
              </li>
              {!last && (
                <li aria-hidden="true">
                  <ChevronRight size={14} />
                </li>
              )}
            </Fragment>
          );
        })}
      </ol>
    </nav>
  );
}

export function Page({ title, description, breadcrumbs, actions, documentTitle, children }: PageProps) {
  useDocumentTitle(documentTitle ?? title);
  return (
    <div className={styles.page}>
      {breadcrumbs && breadcrumbs.length > 0 && <Breadcrumbs items={breadcrumbs} />}
      <header className={styles.header}>
        <div className={styles.heading}>
          <Heading as="h1" size="7" weight="bold" className={styles.title}>
            {title}
          </Heading>
          {description && (
            <Text as="p" size="2" color="gray">
              {description}
            </Text>
          )}
        </div>
        {actions && <div className={styles.actions}>{actions}</div>}
      </header>
      {children}
    </div>
  );
}
