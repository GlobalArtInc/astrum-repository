import { Badge, Button, Checkbox, Popover, TextField as RadixTextField, Text } from '@radix-ui/themes';
import { ChevronDown, X } from 'lucide-react';
import { type KeyboardEvent, useId, useMemo, useState } from 'react';
import { useT } from '../../i18n';
import styles from './Field.module.css';
import { describedBy, type FieldProps, FieldShell, type Option, useFieldId } from './fields';

export interface MultiSelectFieldProps extends FieldProps {
  value: string[];
  onChange(value: string[]): void;
  options: Option[];
  placeholder?: string;
}

export function filterOptions(options: Option[], query: string): Option[] {
  const wanted = query.trim().toLowerCase();
  if (wanted === '') {
    return options;
  }
  return options.filter(
    (option) => option.label.toLowerCase().includes(wanted) || option.value.toLowerCase().includes(wanted),
  );
}

export function MultiSelectField(props: MultiSelectFieldProps) {
  const t = useT();
  const id = useFieldId(props.id);
  const listId = useId();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(0);
  const filtered = useMemo(() => filterOptions(props.options, query), [props.options, query]);
  const labels = useMemo(
    () => new Map(props.options.map((option) => [option.value, option.label])),
    [props.options],
  );

  const toggle = (value: string) => {
    props.onChange(
      props.value.includes(value) ? props.value.filter((item) => item !== value) : [...props.value, value],
    );
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActive((current) => Math.min(filtered.length - 1, current + 1));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive((current) => Math.max(0, current - 1));
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const option = filtered[active];
      if (option) {
        toggle(option.value);
      }
    }
  };

  const activeOption = filtered[active];

  return (
    <FieldShell {...props} id={id}>
      {props.value.length > 0 && (
        <ul
          className={styles.chips}
          aria-label={t('ui.form.chosen')}
          style={{ margin: 0, padding: 0, listStyle: 'none' }}
        >
          {props.value.map((value) => (
            <li key={value}>
              <Badge variant="soft" className={styles.chip}>
                {labels.get(value) ?? value}
                {!props.disabled && (
                  <button
                    type="button"
                    className={styles.chipRemove}
                    aria-label={t('ui.form.remove', { name: labels.get(value) ?? value })}
                    onClick={() => toggle(value)}
                  >
                    <X size={12} aria-hidden="true" />
                  </button>
                )}
              </Badge>
            </li>
          ))}
        </ul>
      )}
      <Popover.Root
        open={open}
        onOpenChange={(next) => {
          setOpen(next);
          setQuery('');
          setActive(0);
        }}
      >
        <Popover.Trigger>
          <Button
            id={id}
            type="button"
            variant="surface"
            color="gray"
            className={styles.trigger}
            disabled={props.disabled}
            aria-haspopup="listbox"
            aria-expanded={open}
            aria-invalid={props.error ? true : undefined}
            aria-describedby={describedBy(id, props.hint, props.error)}
          >
            <Text color={props.value.length === 0 ? 'gray' : undefined}>
              {props.value.length === 0
                ? (props.placeholder ?? t('ui.form.choose'))
                : t('ui.form.chosenCount', { count: props.value.length })}
            </Text>
            <ChevronDown size={14} aria-hidden="true" />
          </Button>
        </Popover.Trigger>
        <Popover.Content
          width="var(--radix-popover-trigger-width)"
          minWidth="240px"
          maxWidth="calc(100vw - 32px)"
        >
          <RadixTextField.Root
            role="combobox"
            aria-label={t('ui.form.filter')}
            aria-controls={listId}
            aria-expanded={true}
            aria-activedescendant={activeOption ? `${listId}-${activeOption.value}` : undefined}
            placeholder={t('ui.form.filter')}
            value={query}
            autoFocus
            onChange={(event) => {
              setQuery(event.target.value);
              setActive(0);
            }}
            onKeyDown={onKeyDown}
          />
          {filtered.length === 0 ? (
            <Text as="p" size="2" color="gray" mt="2">
              {t('ui.form.noOptions')}
            </Text>
          ) : (
            <div
              id={listId}
              role="listbox"
              aria-multiselectable="true"
              aria-label={props.label}
              className={styles.list}
            >
              {filtered.map((option, index) => {
                const selected = props.value.includes(option.value);
                return (
                  <div
                    key={option.value}
                    tabIndex={-1}
                    id={`${listId}-${option.value}`}
                    role="option"
                    aria-selected={selected}
                    data-active={index === active}
                    className={styles.option}
                    onMouseEnter={() => setActive(index)}
                    onClick={() => toggle(option.value)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter' || event.key === ' ') {
                        toggle(option.value);
                      }
                    }}
                  >
                    <Checkbox checked={selected} tabIndex={-1} aria-hidden="true" />
                    {option.label}
                  </div>
                );
              })}
            </div>
          )}
        </Popover.Content>
      </Popover.Root>
    </FieldShell>
  );
}
