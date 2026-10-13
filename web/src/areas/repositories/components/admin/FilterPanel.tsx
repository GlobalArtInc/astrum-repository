import { Button, Checkbox, Select } from '@radix-ui/themes';
import { ChevronDown, ChevronUp, RefreshCw } from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { useT } from '../../../../i18n';
import styles from './admin.module.css';

export type Direction = 'asc' | 'desc';

export interface SortOption {
  value: string;
  label: string;
}

export interface CheckOption {
  value: string;
  label: string;
  count: number;
}

export function FilterPanel({ children }: { children: ReactNode }) {
  const t = useT();
  return (
    <aside className={styles.aside} aria-label={t('repositories.admin.filters')}>
      {children}
    </aside>
  );
}

export function ResetFilters({ onReset }: { onReset(): void }) {
  const t = useT();
  return (
    <Button variant="outline" color="gray" size="2" style={{ alignSelf: 'flex-start' }} onClick={onReset}>
      <RefreshCw size={12} aria-hidden="true" />
      {t('repositories.admin.reset')}
    </Button>
  );
}

export function SortGroup({
  options,
  value,
  direction,
  onChange,
}: {
  options: SortOption[];
  value: string;
  direction: Direction;
  onChange(value: string, direction: Direction): void;
}) {
  const t = useT();
  return (
    <div className={styles.group}>
      <span className={styles.sortTitle}>{t('repositories.admin.sorting')}</span>
      <Select.Root value={value} onValueChange={(next) => onChange(next, direction)}>
        <Select.Trigger aria-label={t('repositories.admin.sortField')} />
        <Select.Content position="popper">
          {options.map((option) => (
            <Select.Item key={option.value} value={option.value}>
              {option.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
      <Select.Root value={direction} onValueChange={(next) => onChange(value, next as Direction)}>
        <Select.Trigger aria-label={t('repositories.admin.sortDirection')} />
        <Select.Content position="popper">
          <Select.Item value="asc">{t('repositories.admin.ascending')}</Select.Item>
          <Select.Item value="desc">{t('repositories.admin.descending')}</Select.Item>
        </Select.Content>
      </Select.Root>
    </div>
  );
}

export function CheckGroup({
  title,
  options,
  selected,
  onToggle,
  limit,
}: {
  title: string;
  options: CheckOption[];
  selected: ReadonlySet<string>;
  onToggle(value: string): void;
  limit?: number;
}) {
  const t = useT();
  const [all, setAll] = useState(false);
  if (options.length === 0) {
    return null;
  }
  const cut = limit !== undefined && options.length > limit;
  const shown = cut && !all ? options.slice(0, limit) : options;
  return (
    <fieldset className={styles.group} style={{ border: 0, margin: 0, padding: 0 }}>
      <legend className={styles.groupTitle} style={{ padding: 0 }}>
        {title}
      </legend>
      <ul className={styles.options}>
        {shown.map((option) => (
          // biome-ignore lint/a11y/noLabelWithoutControl: the Radix checkbox inside is the control
          <label key={option.value} className={styles.option}>
            <Checkbox
              size="1"
              checked={selected.has(option.value)}
              onCheckedChange={() => onToggle(option.value)}
              aria-label={option.label}
            />
            <span>{option.label}</span>
            <span className={styles.count}>{option.count}</span>
          </label>
        ))}
      </ul>
      {cut && (
        <Button variant="ghost" size="1" className={styles.more} onClick={() => setAll(!all)}>
          {all ? <ChevronUp size={12} aria-hidden="true" /> : <ChevronDown size={12} aria-hidden="true" />}
          {all ? t('repositories.admin.showLess') : t('repositories.admin.showAll')}
        </Button>
      )}
    </fieldset>
  );
}

export function countBy<T>(
  rows: readonly T[],
  key: (row: T) => string,
  label: (value: string) => string,
  order: 'label' | 'value' = 'label',
): CheckOption[] {
  const counts = new Map<string, number>();
  for (const row of rows) {
    const value = key(row);
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return [...counts.entries()]
    .map(([value, count]) => ({ value, label: label(value), count }))
    .sort((left, right) =>
      order === 'value'
        ? left.value.localeCompare(right.value)
        : left.label.localeCompare(right.label, undefined, { sensitivity: 'base' }),
    );
}

export function toggled(set: ReadonlySet<string>, value: string): Set<string> {
  const next = new Set(set);
  if (next.has(value)) {
    next.delete(value);
  } else {
    next.add(value);
  }
  return next;
}
