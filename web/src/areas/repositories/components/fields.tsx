import { Checkbox, IconButton, TextField as RadixTextField, Select, TextArea } from '@radix-ui/themes';
import { Eye, EyeOff } from 'lucide-react';
import { type ReactNode, useId, useState } from 'react';
import { useT } from '../../../i18n';
import styles from './fields.module.css';

interface Base {
  label: string;
  hint?: ReactNode;
  error?: string;
  required?: boolean;
  disabled?: boolean;
  name?: string;
}

function ids(id: string, hint: unknown, error: unknown): string | undefined {
  const found = [hint ? `${id}-hint` : '', error ? `${id}-error` : ''].filter(Boolean);
  return found.length > 0 ? found.join(' ') : undefined;
}

export function Section({
  title,
  description,
  children,
  first = false,
}: {
  title: string;
  description?: ReactNode;
  children: ReactNode;
  first?: boolean;
}) {
  return (
    <>
      {!first && <div className={styles.divider} aria-hidden="true" />}
      <section className={styles.section} aria-label={title}>
        <div className={styles.sectionHead}>
          <h2 className={styles.sectionTitle}>{title}</h2>
          {description && <p className={styles.sectionText}>{description}</p>}
        </div>
        {children}
      </section>
    </>
  );
}

export function Field({
  id,
  label,
  hint,
  error,
  required,
  children,
}: Base & { id: string; children: ReactNode }) {
  return (
    <div className={styles.field}>
      <label htmlFor={id} className={styles.label}>
        {label}
        {required && (
          <span className={styles.required} aria-hidden="true">
            *
          </span>
        )}
      </label>
      {hint && (
        <p id={`${id}-hint`} className={styles.hint}>
          {hint}
        </p>
      )}
      {children}
      {error && (
        <p id={`${id}-error`} className={styles.error} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}

export function TextInput(
  props: Base & {
    value: string;
    onChange(value: string): void;
    placeholder?: string;
    type?: 'text' | 'url';
    autoFocus?: boolean;
    maxLength?: number;
    autoComplete?: string;
  },
) {
  const id = useId();
  return (
    <Field {...props} id={id}>
      <RadixTextField.Root
        id={id}
        name={props.name}
        type={props.type ?? 'text'}
        value={props.value}
        placeholder={props.placeholder}
        disabled={props.disabled}
        autoFocus={props.autoFocus}
        maxLength={props.maxLength}
        autoComplete={props.autoComplete}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={ids(id, props.hint, props.error)}
        onChange={(event) => props.onChange(event.target.value)}
      />
    </Field>
  );
}

export function NumberInput(
  props: Base & {
    value: number | null;
    onChange(value: number | null): void;
    placeholder?: string;
    min?: number;
    max?: number;
    footer?: ReactNode;
  },
) {
  const id = useId();
  return (
    <Field {...props} id={id}>
      <RadixTextField.Root
        id={id}
        name={props.name}
        type="number"
        inputMode="numeric"
        value={props.value ?? ''}
        min={props.min}
        max={props.max}
        placeholder={props.placeholder}
        disabled={props.disabled}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={ids(id, props.hint, props.error)}
        onChange={(event) => {
          const text = event.target.value.trim();
          const value = Number(text);
          props.onChange(text === '' || !Number.isFinite(value) ? null : value);
        }}
      />
      {props.footer}
    </Field>
  );
}

export function SecretInput(
  props: Base & { value: string; onChange(value: string): void; stored?: boolean },
) {
  const t = useT();
  const id = useId();
  const [visible, setVisible] = useState(false);
  const hint = props.hint ?? (props.stored ? t('ui.form.secretStored') : undefined);
  return (
    <Field {...props} hint={hint} required={props.required && !props.stored} id={id}>
      <RadixTextField.Root
        id={id}
        name={props.name}
        type={visible ? 'text' : 'password'}
        value={props.value}
        placeholder={props.stored ? '••••••••' : undefined}
        autoComplete="new-password"
        disabled={props.disabled}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={ids(id, hint, props.error)}
        onChange={(event) => props.onChange(event.target.value)}
      >
        <RadixTextField.Slot side="right">
          <IconButton
            type="button"
            size="1"
            variant="ghost"
            color="gray"
            aria-label={visible ? t('ui.form.hideSecret') : t('ui.form.showSecret')}
            aria-pressed={visible}
            onClick={() => setVisible(!visible)}
          >
            {visible ? <EyeOff size={14} aria-hidden="true" /> : <Eye size={14} aria-hidden="true" />}
          </IconButton>
        </RadixTextField.Slot>
      </RadixTextField.Root>
    </Field>
  );
}

export function SelectInput(
  props: Base & {
    value: string;
    onChange(value: string): void;
    options: Array<{ value: string; label: string }>;
    placeholder?: string;
  },
) {
  const id = useId();
  return (
    <Field {...props} id={id}>
      <Select.Root
        value={props.value === '' ? undefined : props.value}
        onValueChange={props.onChange}
        disabled={props.disabled}
        name={props.name}
      >
        <Select.Trigger
          id={id}
          className={styles.select}
          placeholder={props.placeholder}
          aria-invalid={props.error ? true : undefined}
          aria-describedby={ids(id, props.hint, props.error)}
        />
        <Select.Content position="popper">
          {props.options.map((option) => (
            <Select.Item key={option.value} value={option.value}>
              {option.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
    </Field>
  );
}

export function TextAreaInput(props: Base & { value: string; onChange(value: string): void; rows?: number }) {
  const id = useId();
  return (
    <Field {...props} id={id}>
      <TextArea
        id={id}
        name={props.name}
        value={props.value}
        rows={props.rows ?? 3}
        disabled={props.disabled}
        style={{ fontFamily: 'var(--code-font-family)' }}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={ids(id, props.hint, props.error)}
        onChange={(event) => props.onChange(event.target.value)}
      />
    </Field>
  );
}

export function CheckInput({
  label,
  description,
  value,
  onChange,
  disabled,
  error,
  name,
}: {
  label: string;
  description?: ReactNode;
  value: boolean;
  onChange(value: boolean): void;
  disabled?: boolean;
  error?: string;
  name?: string;
}) {
  const id = useId();
  return (
    <div className={styles.field}>
      <label htmlFor={id} className={styles.check}>
        <Checkbox
          id={id}
          name={name}
          size="3"
          className={styles.box}
          checked={value}
          disabled={disabled}
          aria-describedby={description ? `${id}-hint` : undefined}
          onCheckedChange={(checked) => onChange(checked === true)}
        />
        <span>
          <span className={styles.checkLabel}>{label}</span>
          {description && (
            <span id={`${id}-hint`} className={styles.checkText}>
              {description}
            </span>
          )}
        </span>
      </label>
      {error && (
        <p className={styles.error} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
