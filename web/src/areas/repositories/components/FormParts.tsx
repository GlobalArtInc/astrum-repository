import { IconButton, Select, TextField } from '@radix-ui/themes';
import {
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  ChevronUp,
  ExternalLink,
  Search,
  Trash2,
} from 'lucide-react';
import { type KeyboardEvent, type MouseEvent, useId, useState } from 'react';
import { Link } from 'react-router';
import { useT } from '../../../i18n';
import styles from './FormParts.module.css';
import { Field } from './fields';

export function MembersInput({
  value,
  onChange,
  candidates,
  error,
  disabled,
  label,
  hint,
  placeholder,
  empty,
}: {
  value: string[];
  onChange(value: string[]): void;
  candidates: Array<{ name: string; type: string }>;
  error?: string;
  disabled?: boolean;
  label?: string;
  hint?: string;
  placeholder?: string;
  empty?: string;
}) {
  const t = useT();
  const title = label ?? t('repositories.form.members');
  const id = useId();
  const free = candidates.filter((candidate) => !value.includes(candidate.name));
  const move = (index: number, by: number) => {
    const next = [...value];
    const [item] = next.splice(index, 1);
    if (item !== undefined) {
      next.splice(index + by, 0, item);
      onChange(next);
    }
  };
  return (
    <Field id={id} label={title} hint={hint ?? t('repositories.form.membersHint')} required error={error}>
      <Select.Root
        value=""
        disabled={disabled || free.length === 0}
        onValueChange={(name) => onChange([...value, name])}
      >
        <Select.Trigger
          id={id}
          className={styles.add}
          placeholder={placeholder ?? t('repositories.form.addMember')}
          aria-invalid={error ? true : undefined}
        />
        <Select.Content position="popper">
          {free.map((candidate) => (
            <Select.Item key={candidate.name} value={candidate.name}>
              {candidate.name} ({candidate.type})
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
      {value.length === 0 ? (
        <p className={styles.membersEmpty}>{empty ?? t('repositories.form.noMembers')}</p>
      ) : (
        <ol className={styles.members} aria-label={title}>
          {value.map((name, index) => (
            <li key={name} className={styles.member}>
              <span className={styles.position}>{index + 1}</span>
              <span className={styles.memberName}>{name}</span>
              <span className={styles.memberButtons}>
                <IconButton
                  type="button"
                  size="1"
                  variant="ghost"
                  color="gray"
                  disabled={disabled || index === 0}
                  aria-label={t('repositories.form.moveUp')}
                  onClick={() => move(index, -1)}
                >
                  <ChevronUp size={14} />
                </IconButton>
                <IconButton
                  type="button"
                  size="1"
                  variant="ghost"
                  color="gray"
                  disabled={disabled || index === value.length - 1}
                  aria-label={t('repositories.form.moveDown')}
                  onClick={() => move(index, 1)}
                >
                  <ChevronDown size={14} />
                </IconButton>
                <IconButton
                  type="button"
                  size="1"
                  variant="ghost"
                  color="gray"
                  disabled={disabled}
                  aria-label={t('repositories.form.removeMember')}
                  onClick={() => onChange(value.filter((member) => member !== name))}
                >
                  <Trash2 size={14} />
                </IconButton>
              </span>
            </li>
          ))}
        </ol>
      )}
    </Field>
  );
}

export function filterItems(items: readonly string[], filter: string): string[] {
  const needle = filter.trim().toLowerCase();
  return needle === '' ? [...items] : items.filter((item) => item.toLowerCase().includes(needle));
}

export function moveIn(value: readonly string[], items: readonly string[]): string[] {
  return [...value, ...items.filter((item) => !value.includes(item))];
}

export function moveOut(value: readonly string[], items: readonly string[]): string[] {
  const leaving = new Set(items);
  return value.filter((item) => !leaving.has(item));
}

function PolicyList({
  title,
  items,
  selection,
  onSelect,
  onTransfer,
  disabled,
  emptyText,
}: {
  title: string;
  items: string[];
  selection: ReadonlySet<string>;
  onSelect(next: Set<string>): void;
  onTransfer(items: string[]): void;
  disabled: boolean;
  emptyText: string;
}) {
  const t = useT();
  const [filter, setFilter] = useState('');
  const visible = filterItems(items, filter);

  const click = (event: MouseEvent, item: string) => {
    if (disabled) {
      return;
    }
    if (event.ctrlKey || event.metaKey) {
      const next = new Set(selection);
      if (next.has(item)) {
        next.delete(item);
      } else {
        next.add(item);
      }
      onSelect(next);
    } else {
      onSelect(new Set([item]));
    }
  };

  const key = (event: KeyboardEvent<HTMLDivElement>, item: string) => {
    if (disabled) {
      return;
    }
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      const target = event.currentTarget;
      const sibling = (target.nextElementSibling ?? target.previousElementSibling) as HTMLElement | null;
      onTransfer([item]);
      sibling?.focus();
    } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      const sibling = (
        event.key === 'ArrowDown'
          ? event.currentTarget.nextElementSibling
          : event.currentTarget.previousElementSibling
      ) as HTMLElement | null;
      sibling?.focus();
    }
  };

  return (
    <div className={styles.list}>
      <div className={styles.listHead}>
        <span className={styles.listTitle}>{title}</span>
        <span className={styles.listCount} aria-live="polite">
          {t('repositories.form.cleanupItems', { count: items.length })}
        </span>
      </div>
      <div className={styles.listFilter}>
        <TextField.Root
          size="1"
          value={filter}
          placeholder={t('repositories.form.cleanupFilter')}
          aria-label={`${title}: ${t('repositories.form.cleanupFilter')}`}
          spellCheck={false}
          disabled={disabled}
          onChange={(event) => setFilter(event.target.value)}
        >
          <TextField.Slot>
            <Search size={14} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
      </div>
      <div
        className={styles.listBody}
        role="listbox"
        aria-label={title}
        aria-multiselectable="true"
        aria-disabled={disabled || undefined}
      >
        {visible.map((item) => (
          <div
            key={item}
            role="option"
            aria-selected={selection.has(item)}
            tabIndex={disabled ? -1 : 0}
            className={styles.listItem}
            onClick={(event) => click(event, item)}
            onDoubleClick={() => !disabled && onTransfer([item])}
            onKeyDown={(event) => key(event, item)}
          >
            {item}
          </div>
        ))}
        {visible.length === 0 && (
          <p className={styles.listEmpty}>
            {items.length > 0 ? t('repositories.form.cleanupNoMatches') : emptyText}
          </p>
        )}
      </div>
    </div>
  );
}

export function CleanupPolicies({
  options,
  value,
  onChange,
  disabled = false,
  error,
  manage,
}: {
  options: readonly string[];
  value: readonly string[];
  onChange(value: string[]): void;
  disabled?: boolean;
  error?: string;
  manage?: string;
}) {
  const t = useT();
  const id = useId();
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [dropped, setDropped] = useState<Set<string>>(new Set());
  const available = options.filter((option) => !value.includes(option));
  const applied = [...value];
  const pickedNow = available.filter((item) => picked.has(item));
  const droppedNow = applied.filter((item) => dropped.has(item));

  const add = (items: string[]) => {
    if (items.length > 0) {
      onChange(moveIn(value, items));
      setPicked(new Set());
    }
  };
  const remove = (items: string[]) => {
    if (items.length > 0) {
      onChange(moveOut(value, items));
      setDropped(new Set());
    }
  };

  return (
    <div className={styles.cleanup}>
      <p className={styles.cleanupText}>
        {t('repositories.form.cleanupText')}
        {manage && (
          <>
            {' '}
            <Link to={manage} className={styles.manage}>
              {t('repositories.form.cleanupManage')}
              <ExternalLink size={12} aria-hidden="true" />
            </Link>
          </>
        )}
      </p>
      <span id={`${id}-label`} className={styles.cleanupLabel}>
        {t('repositories.form.cleanupPolicies')}
      </span>
      <div className={styles.transfer} role="group" aria-labelledby={`${id}-label`}>
        <PolicyList
          title={t('repositories.form.cleanupAvailable')}
          items={available}
          selection={picked}
          onSelect={setPicked}
          onTransfer={add}
          disabled={disabled}
          emptyText={t('repositories.form.cleanupNoneAvailable')}
        />
        <div className={styles.arrows}>
          <IconButton
            type="button"
            size="1"
            variant="ghost"
            color="gray"
            aria-label={t('repositories.form.cleanupAddAllLabel')}
            title={t('repositories.form.cleanupAddAll')}
            disabled={disabled || available.length === 0}
            onClick={() => add(available)}
          >
            <ChevronsRight size={16} aria-hidden="true" />
          </IconButton>
          <IconButton
            type="button"
            size="1"
            variant="ghost"
            color="gray"
            aria-label={t('repositories.form.cleanupAddLabel')}
            title={t('repositories.form.cleanupAdd')}
            disabled={disabled || pickedNow.length === 0}
            onClick={() => add(pickedNow)}
          >
            <ChevronRight size={16} aria-hidden="true" />
          </IconButton>
          <IconButton
            type="button"
            size="1"
            variant="ghost"
            color="gray"
            aria-label={t('repositories.form.cleanupRemoveLabel')}
            title={t('repositories.form.cleanupRemove')}
            disabled={disabled || droppedNow.length === 0}
            onClick={() => remove(droppedNow)}
          >
            <ChevronLeft size={16} aria-hidden="true" />
          </IconButton>
          <IconButton
            type="button"
            size="1"
            variant="ghost"
            color="gray"
            aria-label={t('repositories.form.cleanupRemoveAllLabel')}
            title={t('repositories.form.cleanupRemoveAll')}
            disabled={disabled || applied.length === 0}
            onClick={() => remove(applied)}
          >
            <ChevronsLeft size={16} aria-hidden="true" />
          </IconButton>
        </div>
        <PolicyList
          title={t('repositories.form.cleanupApplied')}
          items={applied}
          selection={dropped}
          onSelect={setDropped}
          onTransfer={remove}
          disabled={disabled}
          emptyText={t('repositories.form.cleanupNothing')}
        />
      </div>
      <p className={styles.cleanupHint}>{t('repositories.form.cleanupHint')}</p>
      {error && (
        <p className={styles.cleanupError} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
