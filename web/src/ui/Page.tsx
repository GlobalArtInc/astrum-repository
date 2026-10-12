import { Button, Heading, Text } from '@radix-ui/themes';
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
  titleExtra?: ReactNode;
  documentTitle?: string;
  size?: 'medium' | 'large';
  gutter?: 'small' | 'medium' | 'large';
  className?: string;
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
              <li aria-current={last ? 'page' : undefined} className={last ? styles.current : undefined}>
                {item.to && !last ? (
                  <Button asChild variant="ghost" size="2" className={styles.crumbLink}>
                    <Link to={item.to}>{item.label}</Link>
                  </Button>
                ) : (
                  item.label
                )}
              </li>
              {!last && (
                <li aria-hidden="true" className={styles.separator}>
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

export function Page({
  title,
  description,
  breadcrumbs,
  actions,
  titleExtra,
  documentTitle,
  size = 'medium',
  gutter = 'medium',
  className,
  children,
}: PageProps) {
  useDocumentTitle(documentTitle ?? title);
  return (
    <div className={className ? `${styles.page} ${className}` : styles.page} data-gutter={gutter}>
      <div className={styles.top}>
        {breadcrumbs && breadcrumbs.length > 0 && <Breadcrumbs items={breadcrumbs} />}
        <header className={styles.header}>
          <div className={styles.heading}>
            <div className={styles.titleRow}>
              <Heading as="h1" size={size === 'large' ? '7' : '6'} weight="bold" className={styles.title}>
                {title}
              </Heading>
              {titleExtra}
            </div>
            {description && (
              <Text as="p" size="2" className={styles.description}>
                {description}
              </Text>
            )}
          </div>
          {actions && <div className={styles.actions}>{actions}</div>}
        </header>
      </div>
      {children}
    </div>
  );
}
