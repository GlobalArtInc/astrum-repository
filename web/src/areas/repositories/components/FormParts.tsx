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
import { useId } from 'react';
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

function PolicyList({ title, items }: { title: string; items: string[] }) {
  const t = useT();
  return (
    <div className={styles.list}>
      <div className={styles.listHead}>
        <span className={styles.listTitle}>{title}</span>
        <span className={styles.listCount}>
          {t('repositories.form.cleanupItems', { count: items.length })}
        </span>
      </div>
      <div className={styles.listFilter}>
        <TextField.Root
          size="1"
          placeholder={t('repositories.form.cleanupFilter')}
          aria-label={title}
          disabled
        >
          <TextField.Slot>
            <Search size={14} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
      </div>
      <ul className={styles.listBody} aria-label={title}>
        {items.length === 0 ? (
          <li className={styles.listEmpty}>{t('repositories.form.cleanupNothing')}</li>
        ) : (
          items.map((item) => (
            <li key={item} className={styles.listItem}>
              {item}
            </li>
          ))
        )}
      </ul>
    </div>
  );
}

export function CleanupPolicies({ applied }: { applied: string[] }) {
  const t = useT();
  return (
    <div className={styles.cleanup}>
      <p className={styles.cleanupText}>
        {t('repositories.form.cleanupText')}{' '}
        <span className={styles.manage}>
          {t('repositories.form.cleanupManage')}
          <ExternalLink size={12} aria-hidden="true" />
        </span>
      </p>
      <span className={styles.cleanupLabel}>{t('repositories.form.cleanupPolicies')}</span>
      <div className={styles.transfer}>
        <PolicyList title={t('repositories.form.cleanupAvailable')} items={[]} />
        <div className={styles.arrows} aria-hidden="true">
          <ChevronsRight size={16} />
          <ChevronRight size={16} />
          <ChevronLeft size={16} />
          <ChevronsLeft size={16} />
        </div>
        <PolicyList title={t('repositories.form.cleanupApplied')} items={applied} />
      </div>
      <p className={styles.cleanupHint}>{t('repositories.form.cleanupHint')}</p>
    </div>
  );
}
