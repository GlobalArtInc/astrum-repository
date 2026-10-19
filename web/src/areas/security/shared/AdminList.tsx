import { Checkbox, Dialog, IconButton, Table, Text, TextField, VisuallyHidden } from '@radix-ui/themes';
import {
  ArrowDown,
  ArrowUp,
  ChevronDown,
  ChevronRight,
  CircleHelp,
  ExternalLink,
  Filter,
  type LucideIcon,
  Search,
  SlidersHorizontal,
  X,
} from 'lucide-react';
import { type ReactNode, useMemo, useState } from 'react';
import { useI18n, useT } from '../../../i18n';
import { Tooltip } from '../../../ui';
import { PHONE, useMediaQuery } from '../../../ui/hooks';
import styles from './AdminList.module.css';

export interface FilterOption {
  value: string;
  label: string;
  count: number;
}

export interface FilterSection {
  id: string;
  title: string;
  options: FilterOption[];
}

export type FilterState = Record<string, readonly string[]>;

function Sections({
  sections,
  selected,
  onChange,
}: {
  sections: FilterSection[];
  selected: FilterState;
  onChange(section: string, values: string[]): void;
}) {
  const [closed, setClosed] = useState<readonly string[]>([]);
  return (
    <div className={styles.sections}>
      {sections.map((section) => {
        const open = !closed.includes(section.id);
        const chosen = selected[section.id] ?? [];
        return (
          <div key={section.id} className={styles.section}>
            <button
              type="button"
              className={styles.sectionHeader}
              aria-expanded={open}
              onClick={() =>
                setClosed(open ? [...closed, section.id] : closed.filter((id) => id !== section.id))
              }
            >
              {open ? (
                <ChevronDown size={16} aria-hidden="true" className={styles.chevron} />
              ) : (
                <ChevronRight size={16} aria-hidden="true" className={styles.chevron} />
              )}
              <span className={styles.sectionTitle}>{section.title}</span>
            </button>
            {open && (
              <div className={styles.options}>
                {section.options.map((option) => {
                  const checked = chosen.includes(option.value);
                  return (
                    // biome-ignore lint/a11y/noLabelWithoutControl: the Radix checkbox inside is the control
                    <label key={option.value} className={styles.option}>
                      <Checkbox
                        size="2"
                        checked={checked}
                        onCheckedChange={(value) =>
                          onChange(
                            section.id,
                            value === true
                              ? [...chosen, option.value]
                              : chosen.filter((item) => item !== option.value),
                          )
                        }
                      />
                      <span className={styles.optionLabel}>{option.label}</span>
                      <span className={styles.optionCount}>{option.count}</span>
                    </label>
                  );
                })}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

export function FilterPanel({
  sections,
  selected,
  onChange,
}: {
  sections: FilterSection[];
  selected: FilterState;
  onChange(section: string, values: string[]): void;
}) {
  const t = useT();
  const phone = useMediaQuery(PHONE);
  if (phone) {
    return (
      <Dialog.Root>
        <Dialog.Trigger>
          <button type="button" className={styles.filtersButton}>
            <SlidersHorizontal size={18} aria-hidden="true" />
            {t('security.filters.title')}
          </button>
        </Dialog.Trigger>
        <Dialog.Content className={styles.sheet} aria-describedby={undefined}>
          <div className={styles.sheetHeader}>
            <Dialog.Title className={styles.sheetTitle}>{t('security.filters.title')}</Dialog.Title>
            <Dialog.Close>
              <IconButton variant="ghost" size="2" aria-label={t('security.filters.close')}>
                <X size={18} aria-hidden="true" />
              </IconButton>
            </Dialog.Close>
          </div>
          <div className={styles.sheetBody}>
            <Sections sections={sections} selected={selected} onChange={onChange} />
          </div>
        </Dialog.Content>
      </Dialog.Root>
    );
  }
  return (
    <aside className={styles.aside} aria-label={t('security.filters.title')}>
      <div className={styles.asideHeader}>
        <div className={styles.asideTitle}>
          <Filter size={16} aria-hidden="true" />
          <span>{t('security.filters.title')}</span>
        </div>
      </div>
      <Sections sections={sections} selected={selected} onChange={onChange} />
      <div className={styles.asideFooter}>
        <Text size="1" className={styles.footerText}>
          {t('security.filters.automatic')}
        </Text>
      </div>
    </aside>
  );
}

export function ListLayout({
  filters,
  tight,
  children,
}: {
  filters?: ReactNode;
  tight?: boolean;
  children: ReactNode;
}) {
  return (
    <div className={tight ? `${styles.layout} ${styles.tight}` : styles.layout}>
      {filters}
      <div className={styles.main}>{children}</div>
    </div>
  );
}

export function SearchBox({
  value,
  onChange,
  placeholder,
}: {
  value: string;
  onChange(value: string): void;
  placeholder: string;
}) {
  return (
    <TextField.Root
      size="2"
      value={value}
      placeholder={placeholder}
      aria-label={placeholder}
      onChange={(event) => onChange(event.target.value)}
      className={styles.search}
    >
      <TextField.Slot>
        <Search size={16} aria-hidden="true" />
      </TextField.Slot>
    </TextField.Root>
  );
}

export interface ListColumn<T> {
  id: string;
  header: string;
  cell: (row: T) => ReactNode;
  sortValue?: (row: T) => string | number;
  width?: string;
}

export interface RowButton {
  label: string;
  icon: LucideIcon;
  onClick(): void;
  danger?: boolean;
}

export function sortList<T>(
  rows: readonly T[],
  sortValue: ((row: T) => string | number) | undefined,
  direction: 'asc' | 'desc',
  collator: Intl.Collator,
): T[] {
  const sorted = [...rows];
  if (!sortValue) {
    return sorted;
  }
  const sign = direction === 'asc' ? 1 : -1;
  return sorted.sort((left, right) => {
    const a = sortValue(left);
    const b = sortValue(right);
    if (typeof a === 'number' && typeof b === 'number') {
      return sign * (a - b);
    }
    return sign * collator.compare(String(a), String(b));
  });
}

export function ListTable<T>({
  label,
  columns,
  rows,
  getRowId,
  rowLabel,
  onOpen,
  buttons,
  sort,
  onSort,
  empty,
  loading,
  chevron,
  plain,
}: {
  chevron?: boolean;
  plain?: boolean;
  label: string;
  columns: ListColumn<T>[];
  rows: readonly T[];
  getRowId(row: T): string;
  rowLabel?(row: T): string;
  onOpen?(row: T): void;
  buttons?(row: T): RowButton[];
  sort?: { column: string; direction: 'asc' | 'desc' };
  onSort?(column: string): void;
  empty?: ReactNode;
  loading?: boolean;
}) {
  return (
    <div
      className={plain ? `${styles.tableCard} ${styles.plainCard}` : styles.tableCard}
      role="region"
      aria-label={label}
    >
      <Table.Root variant="ghost" size="2" className={styles.table}>
        <Table.Header>
          <Table.Row>
            {columns.map((column) => {
              const sorted = sort?.column === column.id ? sort.direction : undefined;
              return (
                <Table.ColumnHeaderCell
                  key={column.id}
                  className={styles.th}
                  style={column.width ? { width: column.width } : undefined}
                  aria-sort={sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : undefined}
                >
                  {column.sortValue && onSort ? (
                    <button type="button" className={styles.sort} onClick={() => onSort(column.id)}>
                      <span>{column.header}</span>
                      {sorted === 'asc' && <ArrowUp size={14} aria-hidden="true" />}
                      {sorted === 'desc' && <ArrowDown size={14} aria-hidden="true" />}
                    </button>
                  ) : (
                    <span className={styles.thLabel}>{column.header}</span>
                  )}
                </Table.ColumnHeaderCell>
              );
            })}
            {(buttons || chevron) && <Table.ColumnHeaderCell className={styles.th} />}
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {rows.map((row) => {
            const open = onOpen ? () => onOpen(row) : undefined;
            const extra = buttons?.(row) ?? [];
            return (
              <Table.Row
                key={getRowId(row)}
                className={open ? styles.clickable : undefined}
                role={open ? 'button' : undefined}
                tabIndex={open ? 0 : undefined}
                aria-label={open && rowLabel ? rowLabel(row) : undefined}
                onClick={open}
                onKeyDown={
                  open
                    ? (event) => {
                        if (event.key === 'Enter' || event.key === ' ') {
                          event.preventDefault();
                          open();
                        }
                      }
                    : undefined
                }
              >
                {columns.map((column) => (
                  <Table.Cell key={column.id} className={styles.td}>
                    {column.cell(row)}
                  </Table.Cell>
                ))}
                {chevron && !buttons && (
                  <Table.Cell className={`${styles.td} ${styles.chevronCell}`} align="right">
                    <ChevronRight size={16} aria-hidden="true" />
                  </Table.Cell>
                )}
                {buttons && (
                  <Table.Cell className={styles.td} align="right">
                    <span className={styles.buttons}>
                      {extra.map((button) => {
                        const Icon = button.icon;
                        return (
                          <Tooltip key={button.label} content={button.label}>
                            <button
                              type="button"
                              className={
                                button.danger ? `${styles.rowButton} ${styles.danger}` : styles.rowButton
                              }
                              aria-label={button.label}
                              onClick={(event) => {
                                event.stopPropagation();
                                button.onClick();
                              }}
                              onKeyDown={(event) => event.stopPropagation()}
                            >
                              <Icon size={14} aria-hidden="true" />
                            </button>
                          </Tooltip>
                        );
                      })}
                    </span>
                  </Table.Cell>
                )}
              </Table.Row>
            );
          })}
        </Table.Body>
      </Table.Root>
      {rows.length === 0 && !loading && empty}
      {loading && <div className={styles.loading} aria-busy="true" />}
    </div>
  );
}

export function usePaging<T>(rows: readonly T[], size: number) {
  const [page, setPage] = useState(0);
  const pages = Math.max(1, Math.ceil(rows.length / size));
  const current = Math.min(page, pages - 1);
  const visible = useMemo(() => rows.slice(current * size, (current + 1) * size), [rows, current, size]);
  return {
    visible,
    page: current,
    pages,
    from: rows.length === 0 ? 0 : current * size + 1,
    to: Math.min(rows.length, (current + 1) * size),
    total: rows.length,
    setPage,
  };
}

export function ListPager({
  range,
  page,
  pages,
  onPage,
}: {
  range: string;
  page: number;
  pages: number;
  onPage(page: number): void;
}) {
  const t = useT();
  return (
    <div className={styles.pager}>
      <Text size="2" className={styles.pagerText}>
        {range}
      </Text>
      {pages > 1 && (
        <div className={styles.pagerNav}>
          <IconButton
            size="1"
            variant="soft"
            disabled={page === 0}
            aria-label={t('security.pager.previous')}
            onClick={() => onPage(page - 1)}
          >
            <span className={styles.arrow} aria-hidden="true">
              ←
            </span>
          </IconButton>
          <Text size="2" className={styles.pagerPage}>
            {t('security.pager.page', { page: page + 1, pages })}
          </Text>
          <IconButton
            size="1"
            variant="soft"
            disabled={page >= pages - 1}
            aria-label={t('security.pager.next')}
            onClick={() => onPage(page + 1)}
          >
            <span className={styles.arrow} aria-hidden="true">
              →
            </span>
          </IconButton>
        </div>
      )}
    </div>
  );
}

export function HelpCard({
  title,
  children,
  link,
  linkLabel,
  flush,
}: {
  title: string;
  children: ReactNode;
  link: string;
  linkLabel?: string;
  flush?: boolean;
}) {
  const t = useT();
  return (
    <section className={flush ? `${styles.help} ${styles.flush}` : styles.help} aria-label={title}>
      <h2 className={styles.helpTitle}>
        <CircleHelp size={14} aria-hidden="true" className={styles.helpIcon} />
        {title}
      </h2>
      <p className={styles.helpText}>{children}</p>
      <a className={styles.helpLink} href={link} target="_blank" rel="noreferrer">
        {linkLabel ?? t('security.help.docs')}
        <ExternalLink size={12} aria-hidden="true" />
        <VisuallyHidden>{t('security.help.newTab')}</VisuallyHidden>
      </a>
    </section>
  );
}

export function useCollator() {
  const { language } = useI18n();
  return useMemo(() => new Intl.Collator(language, { numeric: true, sensitivity: 'base' }), [language]);
}

export function countBy<T>(rows: readonly T[], key: (row: T) => string): Map<string, number> {
  const counts = new Map<string, number>();
  for (const row of rows) {
    const value = key(row);
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return counts;
}

export const DOCS = 'https://links.globalart.dev/products/astrum/docs';
