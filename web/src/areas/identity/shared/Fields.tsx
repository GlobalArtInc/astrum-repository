import { Checkbox, IconButton, TextField as RadixTextField, Select, TextArea } from '@radix-ui/themes';
import { Eye, EyeOff } from 'lucide-react';
import { type ReactNode, useId, useState } from 'react';
import { useT } from '../../../i18n';
import styles from './Fields.module.css';

interface ShellProps {
  label: string;
  help?: ReactNode;
  required?: boolean;
  error?: string;
}

function ids(id: string, help: unknown, error: unknown): string | undefined {
  const list = [help ? `${id}-help` : '', error ? `${id}-error` : ''].filter(Boolean);
  return list.length > 0 ? list.join(' ') : undefined;
}

function Shell({
  id,
  label,
  help,
  required,
  error,
  children,
}: ShellProps & { id: string; children: ReactNode }) {
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
      {help && (
        <p id={`${id}-help`} className={styles.help}>
          {help}
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

export interface InputProps extends ShellProps {
  value: string;
  onChange(value: string): void;
  name?: string;
  placeholder?: string;
  disabled?: boolean;
  autoFocus?: boolean;
  autoComplete?: string;
  inputMode?: 'numeric' | 'text';
  mono?: boolean;
}

export function Input(props: InputProps) {
  const id = useId();
  return (
    <Shell id={id} {...props}>
      <RadixTextField.Root
        id={id}
        size="2"
        className={styles.control}
        name={props.name}
        value={props.value}
        placeholder={props.placeholder}
        disabled={props.disabled}
        autoFocus={props.autoFocus}
        autoComplete={props.autoComplete ?? 'off'}
        inputMode={props.inputMode}
        required={props.required}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={ids(id, props.help, props.error)}
        onChange={(event) => props.onChange(event.target.value)}
      />
    </Shell>
  );
}

export function Password(props: Omit<InputProps, 'inputMode' | 'mono'>) {
  const t = useT();
  const id = useId();
  const [shown, setShown] = useState(false);
  return (
    <Shell id={id} {...props}>
      <RadixTextField.Root
        id={id}
        size="2"
        className={styles.control}
        type={shown ? 'text' : 'password'}
        name={props.name}
        value={props.value}
        placeholder={props.placeholder}
        disabled={props.disabled}
        autoFocus={props.autoFocus}
        autoComplete={props.autoComplete ?? 'new-password'}
        required={props.required}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={ids(id, props.help, props.error)}
        onChange={(event) => props.onChange(event.target.value)}
      >
        {props.value !== '' && (
          <RadixTextField.Slot side="right">
            <IconButton
              type="button"
              size="1"
              variant="ghost"
              color="gray"
              aria-label={shown ? t('identity.ldap.hidePassword') : t('identity.ldap.showPassword')}
              onClick={() => setShown((value) => !value)}
            >
              {shown ? <EyeOff size={14} aria-hidden="true" /> : <Eye size={14} aria-hidden="true" />}
            </IconButton>
          </RadixTextField.Slot>
        )}
      </RadixTextField.Root>
    </Shell>
  );
}

export interface ChoiceProps<V extends string> extends ShellProps {
  value: V | '';
  onChange(value: V): void;
  options: Array<{ value: V; label: string }>;
  placeholder?: string;
  disabled?: boolean;
  name?: string;
}

export function Choice<V extends string>(props: ChoiceProps<V>) {
  const id = useId();
  return (
    <Shell id={id} {...props}>
      <Select.Root
        value={props.value === '' ? undefined : props.value}
        onValueChange={(value) => props.onChange(value as V)}
        disabled={props.disabled}
        name={props.name}
      >
        <Select.Trigger
          id={id}
          className={`${styles.select} ${styles.control}`}
          placeholder={props.placeholder}
          aria-invalid={props.error ? true : undefined}
          aria-describedby={ids(id, props.help, props.error)}
        />
        <Select.Content position="popper">
          {props.options.map((option) => (
            <Select.Item key={option.value} value={option.value}>
              {option.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
    </Shell>
  );
}

export interface AreaProps extends ShellProps {
  mono?: boolean;
  value: string;
  onChange(value: string): void;
  name?: string;
  rows?: number;
  placeholder?: string;
  disabled?: boolean;
}

export function Area(props: AreaProps) {
  const id = useId();
  return (
    <Shell id={id} {...props}>
      <TextArea
        id={id}
        size="2"
        className={props.mono === false ? styles.plainArea : styles.textarea}
        name={props.name}
        rows={props.rows ?? 4}
        value={props.value}
        placeholder={props.placeholder}
        disabled={props.disabled}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={ids(id, props.help, props.error)}
        onChange={(event) => props.onChange(event.target.value)}
      />
    </Shell>
  );
}

export interface CheckProps {
  label: string;
  help?: ReactNode;
  checked: boolean;
  onChange(checked: boolean): void;
  disabled?: boolean;
  name?: string;
  children?: ReactNode;
}

export function Check({ label, help, checked, onChange, disabled, name, children }: CheckProps) {
  const id = useId();
  return (
    <div className={styles.check} data-disabled={disabled || undefined}>
      <Checkbox
        id={id}
        size="3"
        name={name}
        className={styles.checkBox}
        checked={checked}
        disabled={disabled}
        aria-describedby={help ? `${id}-help` : undefined}
        onCheckedChange={(value) => onChange(value === true)}
      />
      <label htmlFor={id} className={styles.checkLabel}>
        {label}
      </label>
      {help && (
        <p id={`${id}-help`} className={styles.checkHelp}>
          {help}
        </p>
      )}
      {children && <div className={styles.checkHelp}>{children}</div>}
    </div>
  );
}

export function Section({
  title,
  children,
  id,
  first,
}: {
  title?: string;
  children: ReactNode;
  id?: string;
  first?: boolean;
}) {
  return (
    <section
      className={styles.section}
      data-first={first || undefined}
      aria-labelledby={title ? id : undefined}
    >
      {title && (
        <h3 id={id} className={styles.sectionTitle}>
          {title}
        </h3>
      )}
      {children}
    </section>
  );
}

export { styles as fieldStyles };
