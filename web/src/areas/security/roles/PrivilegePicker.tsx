import { Check, ChevronDown, X } from 'lucide-react';
import { Fragment, type KeyboardEvent, type ReactNode, useId, useMemo, useRef, useState } from 'react';
import { useT } from '../../../i18n';
import type { Privilege } from '../api';
import styles from './PrivilegePicker.module.css';

export type PrivilegeGroup = 'other' | 'blobstores' | 'admin' | 'view';

const GROUP_ORDER: PrivilegeGroup[] = ['other', 'blobstores', 'admin', 'view'];

const GROUP_LABELS: Record<PrivilegeGroup, string> = {
  other: 'security.role.groupOther',
  blobstores: 'security.role.groupBlobstores',
  admin: 'security.role.groupAdmin',
  view: 'security.role.groupView',
};

export function groupOf(name: string): PrivilegeGroup {
  if (name.startsWith('as-repository-admin-')) {
    return 'admin';
  }
  if (name.startsWith('as-repository-view-')) {
    return 'view';
  }
  if (name.startsWith('as-blobstores-')) {
    return 'blobstores';
  }
  return 'other';
}

function highlight(text: string, words: string): ReactNode {
  if (!words) {
    return text;
  }
  const lower = text.toLowerCase();
  const parts: ReactNode[] = [];
  let index = 0;
  while (index < text.length) {
    const found = lower.indexOf(words, index);
    if (found < 0) {
      parts.push(text.slice(index));
      break;
    }
    if (found > index) {
      parts.push(text.slice(index, found));
    }
    parts.push(<mark key={found}>{text.slice(found, found + words.length)}</mark>);
    index = found + words.length;
  }
  return parts;
}

export function PrivilegePicker({
  privileges,
  value,
  onChange,
  disabled,
}: {
  privileges: readonly Privilege[];
  value: readonly string[];
  onChange(value: string[]): void;
  disabled?: boolean;
}) {
  const t = useT();
  const id = useId();
  const [query, setQuery] = useState('');
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const words = query.trim().toLowerCase();
  const found = useMemo(
    () =>
      privileges.filter(
        (privilege) =>
          !words ||
          privilege.name.toLowerCase().includes(words) ||
          privilege.description.toLowerCase().includes(words),
      ),
    [privileges, words],
  );
  const groups = GROUP_ORDER.map((group) => ({
    group,
    items: found.filter((privilege) => groupOf(privilege.name) === group),
  })).filter((entry) => entry.items.length > 0);
  const flat = groups.flatMap((entry) => entry.items);
  const toggle = (name: string) =>
    onChange(value.includes(name) ? value.filter((item) => item !== name) : [...value, name]);

  const keys = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setOpen(true);
      setActive((current) => Math.min(current + 1, Math.max(flat.length - 1, 0)));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive((current) => Math.max(current - 1, 0));
    } else if (event.key === 'Enter' && open && flat[active]) {
      event.preventDefault();
      toggle(flat[active].name);
    } else if (event.key === 'Escape') {
      setOpen(false);
    }
  };

  return (
    <div className={styles.picker}>
      <label htmlFor={id} className={styles.count}>
        {t('security.role.privilegesGranted', { count: value.length })}
      </label>
      <div className={styles.field}>
        <input
          id={id}
          ref={input}
          role="combobox"
          aria-expanded={open}
          aria-controls={`${id}-list`}
          aria-autocomplete="list"
          className={styles.input}
          value={query}
          disabled={disabled}
          placeholder={t('security.role.privilegesSearch')}
          onChange={(event) => {
            setQuery(event.target.value);
            setOpen(true);
            setActive(0);
          }}
          onFocus={() => setOpen(true)}
          onBlur={() => setOpen(false)}
          onKeyDown={keys}
        />
        <ChevronDown size={16} aria-hidden="true" className={styles.chevron} />
      </div>
      {open && !disabled && (
        <div id={`${id}-list`} role="listbox" aria-multiselectable="true" className={styles.list}>
          <div role="status" className={styles.status}>
            {t('security.role.privilegesFound', {
              found: found.length,
              total: privileges.length,
              chosen: value.length,
            })}
          </div>
          {groups.map((entry) => (
            <Fragment key={entry.group}>
              <div role="presentation" className={styles.group}>
                <ChevronDown size={12} aria-hidden="true" />
                {t(GROUP_LABELS[entry.group])} ({entry.items.length})
              </div>
              {entry.items.map((privilege) => {
                const on = value.includes(privilege.name);
                const index = flat.indexOf(privilege);
                return (
                  <div
                    key={privilege.name}
                    role="option"
                    tabIndex={-1}
                    aria-selected={on}
                    className={index === active ? `${styles.option} ${styles.active}` : styles.option}
                    onMouseDown={(event) => {
                      event.preventDefault();
                      toggle(privilege.name);
                    }}
                    onMouseEnter={() => setActive(index)}
                  >
                    <span className={on ? `${styles.box} ${styles.boxOn}` : styles.box} aria-hidden="true">
                      {on && <Check size={12} />}
                    </span>
                    <span className={styles.texts}>
                      <span className={styles.name}>{highlight(privilege.name, words)}</span>
                      {privilege.description && (
                        <span className={styles.description}>{highlight(privilege.description, words)}</span>
                      )}
                    </span>
                  </div>
                );
              })}
            </Fragment>
          ))}
        </div>
      )}
      {value.length > 0 && (
        <ul className={styles.chips} aria-label={t('security.role.privileges')}>
          {value.map((name) => (
            <li key={name} className={styles.chip}>
              {name}
              {!disabled && (
                <button
                  type="button"
                  className={styles.remove}
                  aria-label={t('security.role.removePrivilege', { name })}
                  onClick={() => toggle(name)}
                >
                  <X size={10} aria-hidden="true" />
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
