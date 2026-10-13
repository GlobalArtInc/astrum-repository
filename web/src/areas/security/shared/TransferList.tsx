import { TextField } from '@radix-ui/themes';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Search } from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { useT } from '../../../i18n';
import styles from './TransferList.module.css';

export interface TransferOption {
  value: string;
  label: string;
  detail?: ReactNode;
}

function Pane({
  title,
  options,
  selected,
  onToggle,
  onActivate,
  disabled,
  count,
}: {
  title: string;
  options: TransferOption[];
  selected: readonly string[];
  onToggle(value: string): void;
  onActivate(value: string): void;
  disabled?: boolean;
  count: string;
}) {
  const t = useT();
  const [filter, setFilter] = useState('');
  const words = filter.trim().toLowerCase();
  const shown = words ? options.filter((option) => option.label.toLowerCase().includes(words)) : options;
  return (
    <div className={styles.pane}>
      <div className={styles.head}>
        <span className={styles.title}>{title}</span>
        <span className={styles.count}>{count}</span>
      </div>
      <div className={styles.filter}>
        <TextField.Root
          size="1"
          value={filter}
          placeholder={t('security.transfer.filter')}
          aria-label={t('security.transfer.filterOf', { list: title })}
          onChange={(event) => setFilter(event.target.value)}
        >
          <TextField.Slot>
            <Search size={14} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
      </div>
      <div className={styles.list} role="listbox" aria-label={title} aria-multiselectable="true">
        {shown.map((option) => {
          const on = selected.includes(option.value);
          return (
            <div
              key={option.value}
              role="option"
              tabIndex={disabled ? -1 : 0}
              aria-selected={on}
              aria-disabled={disabled || undefined}
              className={on ? `${styles.option} ${styles.selected}` : styles.option}
              onClick={() => !disabled && onToggle(option.value)}
              onDoubleClick={() => !disabled && onActivate(option.value)}
              onKeyDown={(event) => {
                if (disabled) {
                  return;
                }
                if (event.key === ' ') {
                  event.preventDefault();
                  onToggle(option.value);
                } else if (event.key === 'Enter') {
                  event.preventDefault();
                  onActivate(option.value);
                }
              }}
            >
              <span>{option.label}</span>
              {option.detail && <span className={styles.detail}>{option.detail}</span>}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function TransferList({
  label,
  available,
  chosen,
  options,
  value,
  onChange,
  disabled,
  ordered,
  countLabel,
}: {
  label: string;
  available: string;
  chosen: string;
  options: TransferOption[];
  value: readonly string[];
  onChange(value: string[]): void;
  disabled?: boolean;
  ordered?: boolean;
  countLabel(count: number): string;
}) {
  const t = useT();
  const [left, setLeft] = useState<string[]>([]);
  const [right, setRight] = useState<string[]>([]);
  const byValue = new Map(options.map((option) => [option.value, option]));
  const rest = options.filter((option) => !value.includes(option.value));
  const picked = value.map((item) => byValue.get(item) ?? { value: item, label: item });
  const toggle = (list: string[], set: (next: string[]) => void, item: string) =>
    set(list.includes(item) ? list.filter((entry) => entry !== item) : [...list, item]);
  const add = (items: readonly string[]) => {
    onChange([...value, ...items.filter((item) => !value.includes(item))]);
    setLeft([]);
  };
  const remove = (items: readonly string[]) => {
    onChange(value.filter((item) => !items.includes(item)));
    setRight([]);
  };
  const move = (step: number) => {
    const next = [...value];
    const indexes = right
      .map((item) => next.indexOf(item))
      .filter((index) => index >= 0)
      .sort((a, b) => (step < 0 ? a - b : b - a));
    for (const index of indexes) {
      const target = index + step;
      if (target < 0 || target >= next.length || right.includes(next[target] ?? '')) {
        continue;
      }
      const [item] = next.splice(index, 1);
      next.splice(target, 0, item ?? '');
    }
    onChange(next);
  };
  return (
    <div className={styles.transfer} role="group" aria-label={label}>
      <Pane
        title={available}
        options={rest}
        selected={left}
        count={countLabel(rest.length)}
        disabled={disabled}
        onToggle={(item) => toggle(left, setLeft, item)}
        onActivate={(item) => add([item])}
      />
      <div className={styles.buttons}>
        <button
          type="button"
          className={styles.button}
          aria-label={t('security.transfer.addAll')}
          title={t('security.transfer.addAll')}
          disabled={disabled || rest.length === 0}
          onClick={() => add(rest.map((option) => option.value))}
        >
          <ChevronsRight size={16} aria-hidden="true" />
        </button>
        <button
          type="button"
          className={styles.button}
          aria-label={t('security.transfer.add')}
          title={t('security.transfer.add')}
          disabled={disabled || left.length === 0}
          onClick={() => add(left)}
        >
          <ChevronRight size={16} aria-hidden="true" />
        </button>
        <button
          type="button"
          className={styles.button}
          aria-label={t('security.transfer.remove')}
          title={t('security.transfer.remove')}
          disabled={disabled || right.length === 0}
          onClick={() => remove(right)}
        >
          <ChevronLeft size={16} aria-hidden="true" />
        </button>
        <button
          type="button"
          className={styles.button}
          aria-label={t('security.transfer.removeAll')}
          title={t('security.transfer.removeAll')}
          disabled={disabled || value.length === 0}
          onClick={() => remove(value)}
        >
          <ChevronsLeft size={16} aria-hidden="true" />
        </button>
      </div>
      <Pane
        title={chosen}
        options={picked}
        selected={right}
        count={countLabel(picked.length)}
        disabled={disabled}
        onToggle={(item) => toggle(right, setRight, item)}
        onActivate={(item) => remove([item])}
      />
      {ordered && (
        <div className={styles.buttons}>
          <button
            type="button"
            className={styles.button}
            aria-label={t('security.transfer.up')}
            title={t('security.transfer.up')}
            disabled={disabled || right.length === 0}
            onClick={() => move(-1)}
          >
            <ChevronRight size={16} aria-hidden="true" className={styles.up} />
          </button>
          <button
            type="button"
            className={styles.button}
            aria-label={t('security.transfer.down')}
            title={t('security.transfer.down')}
            disabled={disabled || right.length === 0}
            onClick={() => move(1)}
          >
            <ChevronRight size={16} aria-hidden="true" className={styles.down} />
          </button>
        </div>
      )}
    </div>
  );
}
