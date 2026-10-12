import { Badge, Checkbox, TextField as RadixTextField, Text } from '@radix-ui/themes';
import { Search, X } from 'lucide-react';
import { type KeyboardEvent, useId, useMemo, useState } from 'react';
import { useT } from '../../../i18n';
import { FieldShell } from '../../../ui';
import styles from './ChipPicker.module.css';

export interface PickerOption {
  value: string;
  label: string;
  detail?: string;
}

export interface ChipPickerProps {
  label: string;
  value: string[];
  onChange(value: string[]): void;
  options: PickerOption[];
  name?: string;
  hint?: string;
  error?: string;
  disabled?: boolean;
  loading?: boolean;
  limit?: number;
  searchLabel?: string;
}

export function matchOptions(options: readonly PickerOption[], query: string): PickerOption[] {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) {
    return [...options];
  }
  return options.filter((option) => {
    const text = `${option.value} ${option.label} ${option.detail ?? ''}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

export function ChipPicker({
  label,
  value,
  onChange,
  options,
  name,
  hint,
  error,
  disabled = false,
  loading = false,
  limit = 50,
  searchLabel,
}: ChipPickerProps) {
  const t = useT();
  const id = useId();
  const listId = `${id}-list`;
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(0);
  const chosen = useMemo(() => new Set(value), [value]);
  const labels = useMemo(() => new Map(options.map((option) => [option.value, option])), [options]);
  const matching = useMemo(() => matchOptions(options, query), [options, query]);
  const shown = matching.slice(0, limit);
  const hidden = matching.length - shown.length;
  const current = shown[Math.min(active, shown.length - 1)];

  const toggle = (item: string) => {
    if (disabled) {
      return;
    }
    onChange(chosen.has(item) ? value.filter((entry) => entry !== item) : [...value, item]);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActive((index) => Math.min(shown.length - 1, index + 1));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive((index) => Math.max(0, index - 1));
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (current) {
        toggle(current.value);
      }
    }
  };

  return (
    <FieldShell id={`${id}-search`} label={label} hint={hint} error={error}>
      <div className={styles.picker} data-disabled={disabled || undefined}>
        {value.length === 0 ? (
          <Text size="2" color="gray" className={styles.none}>
            {t('security.picker.none')}
          </Text>
        ) : (
          <ul className={styles.chips} aria-label={t('security.picker.chosen', { label })}>
            {value.map((item) => {
              const text = labels.get(item)?.label ?? item;
              return (
                <li key={item}>
                  <Badge size="2" variant="soft" className={styles.chip}>
                    <span className={styles.chipText}>{text}</span>
                    {!disabled && (
                      <button
                        type="button"
                        className={styles.remove}
                        aria-label={t('security.picker.remove', { name: text })}
                        onClick={() => toggle(item)}
                      >
                        <X size={12} aria-hidden="true" />
                      </button>
                    )}
                  </Badge>
                </li>
              );
            })}
          </ul>
        )}
        {!disabled && (
          <>
            <RadixTextField.Root
              id={`${id}-search`}
              name={name}
              type="search"
              role="combobox"
              aria-label={searchLabel ?? t('security.picker.search', { label })}
              aria-controls={listId}
              aria-expanded={true}
              aria-autocomplete="list"
              aria-activedescendant={current ? `${listId}-${shown.indexOf(current)}` : undefined}
              aria-invalid={error ? true : undefined}
              placeholder={t('security.picker.placeholder')}
              value={query}
              onChange={(event) => {
                setQuery(event.target.value);
                setActive(0);
              }}
              onKeyDown={onKeyDown}
            >
              <RadixTextField.Slot>
                <Search size={14} aria-hidden="true" />
              </RadixTextField.Slot>
            </RadixTextField.Root>
            <div
              id={listId}
              role="listbox"
              aria-multiselectable="true"
              aria-label={label}
              aria-busy={loading || undefined}
              className={styles.list}
            >
              {shown.length === 0 ? (
                <Text as="p" size="2" color="gray" className={styles.empty}>
                  {loading ? t('security.picker.loading') : t('security.picker.noMatches')}
                </Text>
              ) : (
                shown.map((option, index) => {
                  const selected = chosen.has(option.value);
                  return (
                    <div
                      key={option.value}
                      id={`${listId}-${index}`}
                      role="option"
                      tabIndex={-1}
                      aria-selected={selected}
                      data-active={option === current || undefined}
                      className={styles.option}
                      onMouseEnter={() => setActive(index)}
                      onMouseDown={(event) => event.preventDefault()}
                      onClick={() => toggle(option.value)}
                      onKeyDown={(event) => {
                        if (event.key === 'Enter' || event.key === ' ') {
                          event.preventDefault();
                          toggle(option.value);
                        }
                      }}
                    >
                      <Checkbox checked={selected} tabIndex={-1} aria-hidden="true" />
                      <span className={styles.optionText}>
                        <Text size="2">{option.label}</Text>
                        {option.detail && (
                          <Text size="1" color="gray" className={styles.detail}>
                            {option.detail}
                          </Text>
                        )}
                      </span>
                    </div>
                  );
                })
              )}
            </div>
            {hidden > 0 && (
              <Text size="1" color="gray" role="status">
                {t('security.picker.more', { count: hidden })}
              </Text>
            )}
          </>
        )}
      </div>
    </FieldShell>
  );
}
