import { Button, IconButton, TextField as RadixTextField, Text } from '@radix-ui/themes';
import { Plus, X } from 'lucide-react';
import { type KeyboardEvent, useId, useState } from 'react';
import { useT } from '../../../i18n';
import { FieldShell } from '../../../ui';
import formStyles from '../shared/Form.module.css';
import styles from './Ssrf.module.css';

export interface ListEditorProps {
  label: string;
  hint?: string;
  values: string[];
  onChange(values: string[]): void;
  check(value: string): string | undefined;
  normalize?: (value: string) => string;
  placeholder?: string;
  error?: string;
  disabled?: boolean;
  emptyText: string;
}

export function ListEditor({
  label,
  hint,
  values,
  onChange,
  check,
  normalize = (value) => value.trim(),
  placeholder,
  error,
  disabled = false,
  emptyText,
}: ListEditorProps) {
  const t = useT();
  const id = useId();
  const [draft, setDraft] = useState('');
  const [problem, setProblem] = useState<string | undefined>();

  const add = () => {
    const value = normalize(draft);
    if (value === '') {
      return;
    }
    const found = check(value);
    if (found) {
      setProblem(found);
      return;
    }
    if (values.includes(value)) {
      setProblem(t('security.ssrf.duplicate', { value }));
      return;
    }
    onChange([...values, value]);
    setDraft('');
    setProblem(undefined);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      add();
    }
  };

  return (
    <FieldShell id={`${id}-input`} label={label} hint={hint} error={problem ?? error}>
      {!disabled && (
        <div className={styles.add}>
          <RadixTextField.Root
            id={`${id}-input`}
            className={styles.input}
            value={draft}
            placeholder={placeholder}
            autoComplete="off"
            spellCheck={false}
            aria-invalid={problem || error ? true : undefined}
            aria-describedby={problem || error ? `${id}-input-error` : hint ? `${id}-input-hint` : undefined}
            onChange={(event) => {
              setDraft(event.target.value);
              setProblem(undefined);
            }}
            onKeyDown={onKeyDown}
          />
          <Button type="button" variant="soft" onClick={add} disabled={draft.trim() === ''}>
            <Plus size={16} aria-hidden="true" />
            {t('security.ssrf.add')}
          </Button>
        </div>
      )}
      {values.length === 0 ? (
        <Text size="2" color="gray">
          {emptyText}
        </Text>
      ) : (
        <ul className={styles.entries} aria-label={label}>
          {values.map((value) => (
            <li key={value} className={styles.entry}>
              <span className={formStyles.mono}>{value}</span>
              {!disabled && (
                <IconButton
                  type="button"
                  size="1"
                  variant="ghost"
                  color="gray"
                  aria-label={t('security.ssrf.remove', { value })}
                  onClick={() => onChange(values.filter((item) => item !== value))}
                >
                  <X size={14} aria-hidden="true" />
                </IconButton>
              )}
            </li>
          ))}
        </ul>
      )}
    </FieldShell>
  );
}
