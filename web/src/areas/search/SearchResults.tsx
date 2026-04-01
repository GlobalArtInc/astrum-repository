import { Button, IconButton, Spinner, Table, Text } from '@radix-ui/themes';
import { History, type LucideIcon } from 'lucide-react';
import type { KeyboardEvent, ReactNode } from 'react';
import { useNavigate } from 'react-router';
import { useT } from '../../i18n';
import { DateTime, FormatBadge, PHONE, useMediaQuery } from '../../ui';
import { componentPath, displayName, groupLine, lastModified, openPath, type SearchComponent } from './model';
import styles from './Search.module.css';
import type { PagedSearch } from './usePagedSearch';

export function SearchMessage({
  icon: Icon,
  title,
  children,
  action,
  role,
}: {
  icon?: LucideIcon;
  title: string;
  children?: ReactNode;
  action?: ReactNode;
  role?: 'alert';
}) {
  return (
    <div className={styles.message} role={role}>
      {Icon && (
        <div className={styles.messageIcon}>
          <Icon size={28} aria-hidden="true" />
        </div>
      )}
      <Text size="3" weight="bold">
        {title}
      </Text>
      {children && (
        <Text size="2" align="center" className={styles.messageText}>
          {children}
        </Text>
      )}
      {action}
    </div>
  );
}

export function activate(open: () => void) {
  return (event: KeyboardEvent) => {
    if (event.target !== event.currentTarget) {
      return;
    }
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      open();
    }
  };
}

export function LoadMore({
  results,
}: {
  results: Pick<PagedSearch<unknown>, 'hasMore' | 'loadingMore' | 'loadMore'>;
}) {
  const t = useT();
  if (!results.hasMore) {
    return null;
  }
  return (
    <div className={styles.more}>
      <Button
        type="button"
        size="2"
        variant="surface"
        color="gray"
        highContrast
        loading={results.loadingMore}
        onClick={results.loadMore}
      >
        {t('search.results.more')}
      </Button>
    </div>
  );
}

function VersionsButton({ component, size }: { component: SearchComponent; size: '1' | '3' }) {
  const t = useT();
  const navigate = useNavigate();
  return (
    <IconButton
      type="button"
      size={size}
      variant="ghost"
      color="gray"
      className={size === '3' ? styles.itemVersions : undefined}
      aria-label={t('search.results.versionsOf', { name: displayName(component) })}
      title={t('search.results.versions')}
      onClick={(event) => {
        event.stopPropagation();
        navigate(componentPath(component));
      }}
    >
      <History size={size === '1' ? 16 : 18} aria-hidden="true" />
    </IconButton>
  );
}

function PhoneList({ items }: { items: SearchComponent[] }) {
  const t = useT();
  const navigate = useNavigate();
  return (
    <ul className={styles.list}>
      {items.map((component) => {
        const group = groupLine(component);
        const version = component.version || '—';
        return (
          <li key={component.id} className={styles.item}>
            <button
              type="button"
              className={styles.open}
              aria-label={t('search.results.open', { name: displayName(component) })}
              onClick={() => navigate(openPath(component))}
            >
              <Text size="2" weight="medium" className={styles.itemName}>
                {displayName(component)}
              </Text>
              <Text size="1" className={`${styles.mono} ${styles.gray}`}>
                {group ? `${group} · ${version}` : version}
              </Text>
              <span className={styles.listMeta}>
                <FormatBadge format={component.format} />
                <Text size="1" className={styles.gray}>
                  {component.repository}
                </Text>
              </span>
            </button>
            <VersionsButton component={component} size="3" />
          </li>
        );
      })}
    </ul>
  );
}

function DesktopTable({ items }: { items: SearchComponent[] }) {
  const t = useT();
  const navigate = useNavigate();
  return (
    <Table.Root variant="surface" size="2" className={styles.table}>
      <Table.Header>
        <Table.Row>
          <Table.ColumnHeaderCell>{t('search.results.component')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{t('search.results.version')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{t('search.results.format')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell>{t('search.results.repository')}</Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell className={styles.updated}>
            {t('search.results.updated')}
          </Table.ColumnHeaderCell>
          <Table.ColumnHeaderCell width="48px" aria-label={t('search.results.versions')} />
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {items.map((component) => {
          const group = groupLine(component);
          const when = lastModified(component);
          const open = () => navigate(openPath(component));
          return (
            <Table.Row
              key={component.id}
              align="center"
              className={styles.row}
              role="button"
              tabIndex={0}
              aria-label={t('search.results.open', { name: displayName(component) })}
              onClick={open}
              onKeyDown={activate(open)}
            >
              <Table.Cell>
                <div className={styles.nameCell}>
                  <Text size="2" weight="medium">
                    {displayName(component)}
                  </Text>
                  {group && (
                    <Text size="1" className={`${styles.mono} ${styles.gray}`}>
                      {group}
                    </Text>
                  )}
                </div>
              </Table.Cell>
              <Table.Cell>
                <Text size="2" className={styles.mono}>
                  {component.version || '—'}
                </Text>
              </Table.Cell>
              <Table.Cell>
                <FormatBadge format={component.format} />
              </Table.Cell>
              <Table.Cell>
                <Text size="2" className={styles.gray}>
                  {component.repository}
                </Text>
              </Table.Cell>
              <Table.Cell className={styles.updated}>
                <Text size="2" className={styles.gray}>
                  {when ? <DateTime value={when} /> : '—'}
                </Text>
              </Table.Cell>
              <Table.Cell justify="end">
                <VersionsButton component={component} size="1" />
              </Table.Cell>
            </Table.Row>
          );
        })}
      </Table.Body>
    </Table.Root>
  );
}

export function ComponentResults({
  results,
  empty,
}: {
  results: PagedSearch<SearchComponent>;
  empty: ReactNode;
}) {
  const t = useT();
  const phone = useMediaQuery(PHONE);
  if (results.error && !results.items?.length) {
    return (
      <SearchMessage
        role="alert"
        title={t('search.results.errorTitle')}
        action={
          <Button type="button" variant="soft" color="gray" onClick={results.retry}>
            {t('search.results.retry')}
          </Button>
        }
      >
        {results.error.message}
      </SearchMessage>
    );
  }
  if (results.loading || results.items === undefined) {
    return (
      <div className={styles.loading} role="status">
        <Spinner size="3" />
        <Text size="2" className={styles.gray}>
          {t('search.results.loading')}
        </Text>
      </div>
    );
  }
  if (results.items.length === 0) {
    return <>{empty}</>;
  }
  return (
    <div className={styles.resultStack}>
      <div className={styles.status}>
        <Text size="2" className={styles.gray} role="status">
          {t('search.results.shown', { count: results.items.length })}
        </Text>
      </div>
      {phone ? <PhoneList items={results.items} /> : <DesktopTable items={results.items} />}
      <LoadMore results={results} />
    </div>
  );
}
