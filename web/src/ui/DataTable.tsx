import { Button, DropdownMenu, Flex, IconButton, Skeleton, Table, Text } from '@radix-ui/themes';
import { ArrowDown, ArrowUp, ArrowUpDown, type LucideIcon, MoreHorizontal } from 'lucide-react';
import { type ReactNode, useMemo, useState } from 'react';
import { Link } from 'react-router';
import { useI18n } from '../i18n';
import styles from './DataTable.module.css';
import { EmptyState } from './EmptyState';
import { ErrorState } from './ErrorState';
import { PHONE, useMediaQuery } from './hooks';

export type SortValue = string | number | boolean | Date | null | undefined;
export type Direction = 'asc' | 'desc';

export interface Column<T> {
  id: string;
  header: string;
  cell: (row: T) => ReactNode;
  sortValue?: (row: T) => SortValue;
  align?: 'left' | 'center' | 'right';
  width?: string;
  hideOnPhone?: boolean;
  primary?: boolean;
}

export interface RowAction<T> {
  id: string;
  label: string;
  onSelect: (row: T) => void;
  icon?: LucideIcon;
  danger?: boolean;
  disabled?: boolean;
}

export interface DataTableProps<T> {
  label: string;
  columns: Column<T>[];
  rows: readonly T[] | undefined;
  getRowId: (row: T) => string;
  loading?: boolean;
  error?: unknown;
  onRetry?: () => void;
  empty?: ReactNode;
  rowActions?: (row: T) => RowAction<T>[];
  rowHref?: (row: T) => string | undefined;
  initialSort?: { column: string; direction: Direction };
  pageSize?: number;
  paging?: 'pages' | 'more';
  hasMore?: boolean;
  onLoadMore?: () => void;
  loadingMore?: boolean;
}

export function compareValues(left: SortValue, right: SortValue, collator: Intl.Collator): number {
  const missing = (value: SortValue) => value === null || value === undefined || value === '';
  if (missing(left) || missing(right)) {
    return Number(missing(left)) - Number(missing(right));
  }
  if (left instanceof Date && right instanceof Date) {
    return left.getTime() - right.getTime();
  }
  if (typeof left === 'number' && typeof right === 'number') {
    return left - right;
  }
  return collator.compare(String(left), String(right));
}

export function sortRows<T>(
  rows: readonly T[],
  column: Column<T> | undefined,
  direction: Direction,
  collator: Intl.Collator,
): T[] {
  const sorted = [...rows];
  const value = column?.sortValue;
  if (!value) {
    return sorted;
  }
  const sign = direction === 'asc' ? 1 : -1;
  return sorted.sort((left, right) => {
    const a = value(left);
    const b = value(right);
    const missing = (item: SortValue) => item === null || item === undefined || item === '';
    if (missing(a) || missing(b)) {
      return compareValues(a, b, collator);
    }
    return sign * compareValues(a, b, collator);
  });
}

function ActionsMenu<T>({ row, actions, label }: { row: T; actions: RowAction<T>[]; label: string }) {
  if (actions.length === 0) {
    return null;
  }
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <IconButton variant="ghost" color="gray" size="1" aria-label={label}>
          <MoreHorizontal size={16} aria-hidden="true" />
        </IconButton>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end">
        {actions.map((action) => {
          const Icon = action.icon;
          return (
            <DropdownMenu.Item
              key={action.id}
              color={action.danger ? 'red' : undefined}
              disabled={action.disabled}
              onSelect={() => action.onSelect(row)}
            >
              {Icon && <Icon size={14} aria-hidden="true" />}
              {action.label}
            </DropdownMenu.Item>
          );
        })}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}

export function DataTable<T>(props: DataTableProps<T>) {
  const {
    label,
    columns,
    rows,
    getRowId,
    loading = false,
    error,
    onRetry,
    empty,
    rowActions,
    rowHref,
    initialSort,
    pageSize,
    paging = 'pages',
    hasMore = false,
    onLoadMore,
    loadingMore = false,
  } = props;
  const { language, t } = useI18n();
  const phone = useMediaQuery(PHONE);
  const [sort, setSort] = useState(initialSort);
  const [shown, setShown] = useState(pageSize ?? Number.POSITIVE_INFINITY);
  const [page, setPage] = useState(0);
  const collator = useMemo(
    () => new Intl.Collator(language, { numeric: true, sensitivity: 'base' }),
    [language],
  );

  const sorted = useMemo(() => {
    const column = columns.find((candidate) => candidate.id === sort?.column);
    return sortRows(rows ?? [], column, sort?.direction ?? 'asc', collator);
  }, [rows, columns, sort, collator]);

  const total = sorted.length;
  const lastPage = pageSize ? Math.max(0, Math.ceil(total / pageSize) - 1) : 0;
  const currentPage = Math.min(page, lastPage);
  const visible = !pageSize
    ? sorted
    : paging === 'more'
      ? sorted.slice(0, shown)
      : sorted.slice(currentPage * pageSize, (currentPage + 1) * pageSize);

  const toggleSort = (column: Column<T>) => {
    setSort((current) =>
      current?.column === column.id
        ? { column: column.id, direction: current.direction === 'asc' ? 'desc' : 'asc' }
        : { column: column.id, direction: 'asc' },
    );
    setPage(0);
  };

  const primary = columns.find((column) => column.primary) ?? columns[0];
  const rowLabel = t('ui.table.rowActions');

  const renderPrimary = (row: T) => {
    const href = rowHref?.(row);
    const content = primary?.cell(row);
    return href ? (
      <Link to={href} className={styles.rowLink}>
        {content}
      </Link>
    ) : (
      content
    );
  };

  if (error && !loading && (rows === undefined || rows.length === 0)) {
    return (
      <div className={styles.wrap}>
        <ErrorState error={error} onRetry={onRetry} />
      </div>
    );
  }

  const footer = (() => {
    const parts: ReactNode[] = [];
    if (pageSize && paging === 'pages' && total > pageSize) {
      const from = currentPage * pageSize + 1;
      const to = Math.min(total, (currentPage + 1) * pageSize);
      parts.push(
        <Text key="range" size="2" color="gray">
          {t('ui.table.range', { from, to, total })}
        </Text>,
        <Flex key="nav" gap="2">
          <Button
            variant="soft"
            color="gray"
            size="1"
            disabled={currentPage === 0}
            onClick={() => setPage(currentPage - 1)}
          >
            {t('ui.table.previous')}
          </Button>
          <Button
            variant="soft"
            color="gray"
            size="1"
            disabled={currentPage >= lastPage}
            onClick={() => setPage(currentPage + 1)}
          >
            {t('ui.table.next')}
          </Button>
        </Flex>,
      );
    }
    const more = (pageSize && paging === 'more' && shown < total) || (hasMore && onLoadMore);
    if (more) {
      parts.push(
        <Text key="count" size="2" color="gray">
          {t('ui.table.shown', { count: visible.length })}
        </Text>,
        <Button
          key="more"
          variant="soft"
          size="1"
          loading={loadingMore}
          onClick={() => {
            if (pageSize && paging === 'more' && shown < total) {
              setShown(shown + pageSize);
            } else {
              onLoadMore?.();
            }
          }}
        >
          {t('ui.table.more')}
        </Button>,
      );
    }
    return parts.length > 0 ? parts : null;
  })();

  const isEmpty = !loading && total === 0;
  const emptyContent = empty ?? <EmptyState title={t('ui.table.empty')} />;

  if (phone) {
    return (
      <section aria-label={label} aria-busy={loading}>
        {loading && total === 0 ? (
          <ul className={styles.cards}>
            {[0, 1, 2].map((index) => (
              <li key={index} className={styles.card}>
                <Skeleton height="48px" />
              </li>
            ))}
          </ul>
        ) : isEmpty ? (
          <div className={styles.wrap}>{emptyContent}</div>
        ) : (
          <ul className={styles.cards}>
            {visible.map((row) => (
              <li key={getRowId(row)} className={styles.card}>
                <div className={styles.cardHead}>
                  <div className={styles.cardTitle}>{renderPrimary(row)}</div>
                  {rowActions && <ActionsMenu row={row} actions={rowActions(row)} label={rowLabel} />}
                </div>
                <dl className={styles.fields}>
                  {columns
                    .filter((column) => column !== primary && !column.hideOnPhone)
                    .map((column) => (
                      <div key={column.id} style={{ display: 'contents' }}>
                        <dt>{column.header}</dt>
                        <dd>{column.cell(row)}</dd>
                      </div>
                    ))}
                </dl>
              </li>
            ))}
          </ul>
        )}
        {footer && <div className={styles.cardsFooter}>{footer}</div>}
      </section>
    );
  }

  return (
    <div className={styles.wrap} aria-busy={loading}>
      <Table.Root variant="ghost" size="2">
        <caption className="visually-hidden">{label}</caption>
        <Table.Header>
          <Table.Row>
            {columns.map((column) => {
              const sorted = sort?.column === column.id ? sort.direction : undefined;
              const Arrow = sorted === 'asc' ? ArrowUp : sorted === 'desc' ? ArrowDown : ArrowUpDown;
              return (
                <Table.ColumnHeaderCell
                  key={column.id}
                  align={column.align}
                  width={column.width}
                  aria-sort={sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : undefined}
                >
                  {column.sortValue ? (
                    <button type="button" className={styles.sort} onClick={() => toggleSort(column)}>
                      {column.header}
                      <Arrow size={14} aria-hidden="true" opacity={sorted ? 1 : 0.4} />
                    </button>
                  ) : (
                    column.header
                  )}
                </Table.ColumnHeaderCell>
              );
            })}
            {rowActions && (
              <Table.ColumnHeaderCell width="48px">
                <span className="visually-hidden">{rowLabel}</span>
              </Table.ColumnHeaderCell>
            )}
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {loading && total === 0
            ? [0, 1, 2, 3, 4].map((index) => (
                <Table.Row key={index}>
                  {columns.map((column) => (
                    <Table.Cell key={column.id}>
                      <Skeleton height="16px" />
                    </Table.Cell>
                  ))}
                  {rowActions && <Table.Cell />}
                </Table.Row>
              ))
            : visible.map((row) => (
                <Table.Row key={getRowId(row)} align="center">
                  {columns.map((column) =>
                    column === primary ? (
                      <Table.RowHeaderCell key={column.id} align={column.align}>
                        {renderPrimary(row)}
                      </Table.RowHeaderCell>
                    ) : (
                      <Table.Cell key={column.id} align={column.align}>
                        {column.cell(row)}
                      </Table.Cell>
                    ),
                  )}
                  {rowActions && (
                    <Table.Cell align="right">
                      <ActionsMenu row={row} actions={rowActions(row)} label={rowLabel} />
                    </Table.Cell>
                  )}
                </Table.Row>
              ))}
        </Table.Body>
      </Table.Root>
      {isEmpty && emptyContent}
      {footer && <div className={styles.footer}>{footer}</div>}
    </div>
  );
}
