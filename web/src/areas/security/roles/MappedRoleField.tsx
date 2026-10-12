import { TextField as RadixTextField, Spinner, Text } from '@radix-ui/themes';
import { Search } from 'lucide-react';
import { type KeyboardEvent, useId, useState } from 'react';
import { useApi, withQuery } from '../../../api';
import { useT } from '../../../i18n';
import { FieldShell } from '../../../ui';
import { type Role, SOURCE_ROLES } from '../api';
import { useDebounced } from '../shared/useDebounced';
import styles from './MappedRoleField.module.css';

export const MIN_SEARCH = 3;

export function MappedRoleField({
  source,
  sourceName,
  value,
  onPick,
  disabled,
}: {
  source: string;
  sourceName: string;
  value: string;
  onPick(id: string): void;
  disabled?: boolean;
}) {
  const t = useT();
  const id = useId();
  const listId = `${id}-list`;
  const [query, setQuery] = useState(value);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const settled = useDebounced(query.trim(), 300);
  const searching = settled.length >= MIN_SEARCH && open;
  const found = useApi<Role[]>(searching ? withQuery(SOURCE_ROLES, { source, search: settled }) : null);
  const results = searching ? (found.data ?? []) : [];
  const shown = open && settled.length >= MIN_SEARCH && (found.data !== undefined || !found.isFetching);

  const pick = (role: Role) => {
    setQuery(role.id);
    setOpen(false);
    onPick(role.id);
  };

  const keys = (event: KeyboardEvent) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setOpen(true);
      setActive((current) => Math.min(current + 1, Math.max(results.length - 1, 0)));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive((current) => Math.max(current - 1, 0));
    } else if (event.key === 'Enter' && open && results[active]) {
      event.preventDefault();
      pick(results[active]);
    } else if (event.key === 'Escape') {
      setOpen(false);
    }
  };

  return (
    <FieldShell
      id={id}
      label={t('security.role.mappedRole')}
      hint={t('security.role.mappedRoleHint', { source: sourceName })}
    >
      <div className={styles.anchor}>
        <RadixTextField.Root
          id={id}
          value={query}
          disabled={disabled}
          placeholder={t('security.role.mappedRoleSearch', { source: sourceName })}
          role="combobox"
          aria-expanded={shown}
          aria-controls={listId}
          aria-autocomplete="list"
          autoComplete="off"
          onChange={(event) => {
            setQuery(event.target.value);
            setActive(0);
            setOpen(true);
            onPick(event.target.value.trim());
          }}
          onFocus={() => setOpen(true)}
          onBlur={() => setOpen(false)}
          onKeyDown={keys}
        >
          <RadixTextField.Slot>
            <Search size={14} aria-hidden="true" />
          </RadixTextField.Slot>
          {found.isFetching && (
            <RadixTextField.Slot side="right">
              <Spinner size="1" />
            </RadixTextField.Slot>
          )}
        </RadixTextField.Root>
        {shown && (
          <div className={styles.popover}>
            {results.length > 0 ? (
              <div id={listId} role="listbox" className={styles.list}>
                {results.map((role, index) => (
                  <div
                    key={role.id}
                    role="option"
                    tabIndex={-1}
                    aria-selected={index === active}
                    data-active={index === active || undefined}
                    className={styles.option}
                    onMouseDown={(event) => event.preventDefault()}
                    onClick={() => pick(role)}
                    onKeyDown={() => undefined}
                  >
                    {role.id}
                  </div>
                ))}
              </div>
            ) : (
              !found.isFetching && (
                <Text as="p" size="1" color="gray" className={styles.none}>
                  {t('security.role.mappedRoleNone', { source: sourceName })}
                </Text>
              )
            )}
          </div>
        )}
      </div>
    </FieldShell>
  );
}
