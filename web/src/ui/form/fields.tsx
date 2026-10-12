import {
  Checkbox,
  IconButton,
  TextField as RadixTextField,
  Select,
  Switch,
  Text,
  TextArea,
} from '@radix-ui/themes';
import { Eye, EyeOff } from 'lucide-react';
import { type ReactNode, useId, useState } from 'react';
import { useT } from '../../i18n';
import styles from './Field.module.css';

export interface FieldProps {
  label: string;
  name?: string;
  id?: string;
  hint?: ReactNode;
  error?: string;
  required?: boolean;
  disabled?: boolean;
  size?: '1' | '2' | '3';
}

export interface Option {
  value: string;
  label: string;
}

export function describedBy(id: string, hint: unknown, error: unknown): string | undefined {
  const ids = [hint ? `${id}-hint` : '', error ? `${id}-error` : ''].filter(Boolean);
  return ids.length > 0 ? ids.join(' ') : undefined;
}

export function FieldShell({
  id,
  label,
  hint,
  error,
  required,
  children,
}: Omit<FieldProps, 'id'> & { id: string; children: ReactNode }) {
  return (
    <div className={styles.field}>
      <Text as="label" htmlFor={id} size="2" weight="medium" className={styles.label}>
        {label}
        {required && (
          <span className={styles.required} aria-hidden="true">
            *
          </span>
        )}
      </Text>
      {children}
      {hint && (
        <Text id={`${id}-hint`} size="1" color="gray">
          {hint}
        </Text>
      )}
      {error && (
        <Text id={`${id}-error`} size="1" color="red" role="alert">
          {error}
        </Text>
      )}
    </div>
  );
}

export function useFieldId(id?: string): string {
  const generated = useId();
  return id ?? generated;
}

export interface TextFieldProps extends FieldProps {
  value: string;
  onChange(value: string): void;
  type?: 'text' | 'email' | 'url' | 'search' | 'tel';
  placeholder?: string;
  autoComplete?: string;
  autoFocus?: boolean;
  maxLength?: number;
}

export function TextField(props: TextFieldProps) {
  const id = useFieldId(props.id);
  return (
    <FieldShell {...props} id={id}>
      <RadixTextField.Root
        id={id}
        size={props.size}
        name={props.name}
        type={props.type ?? 'text'}
        value={props.value}
        placeholder={props.placeholder}
        autoComplete={props.autoComplete}
        autoFocus={props.autoFocus}
        maxLength={props.maxLength}
        disabled={props.disabled}
        required={props.required}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={describedBy(id, props.hint, props.error)}
        onChange={(event) => props.onChange(event.target.value)}
      />
    </FieldShell>
  );
}

export interface NumberFieldProps extends FieldProps {
  value: number | null;
  onChange(value: number | null): void;
  min?: number;
  max?: number;
  step?: number;
  placeholder?: string;
}

export function parseNumber(text: string): number | null {
  if (text.trim() === '') {
    return null;
  }
  const value = Number(text);
  return Number.isFinite(value) ? value : null;
}

export function NumberField(props: NumberFieldProps) {
  const id = useFieldId(props.id);
  return (
    <FieldShell {...props} id={id}>
      <RadixTextField.Root
        id={id}
        size={props.size}
        name={props.name}
        type="number"
        inputMode="numeric"
        value={props.value ?? ''}
        min={props.min}
        max={props.max}
        step={props.step}
        placeholder={props.placeholder}
        disabled={props.disabled}
        required={props.required}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={describedBy(id, props.hint, props.error)}
        onChange={(event) => props.onChange(parseNumber(event.target.value))}
      />
    </FieldShell>
  );
}

export interface SecretFieldProps extends FieldProps {
  value: string;
  onChange(value: string): void;
  stored?: boolean;
  autoComplete?: string;
  autoFocus?: boolean;
  placeholder?: string;
}

export function SecretField(props: SecretFieldProps) {
  const t = useT();
  const id = useFieldId(props.id);
  const [visible, setVisible] = useState(false);
  const keeps = props.stored === true;
  const hint = props.hint ?? (keeps ? t('ui.form.secretStored') : undefined);
  return (
    <FieldShell {...props} hint={hint} id={id} required={props.required && !keeps}>
      <RadixTextField.Root
        id={id}
        size={props.size}
        name={props.name}
        type={visible ? 'text' : 'password'}
        value={props.value}
        placeholder={keeps ? '••••••••' : props.placeholder}
        autoComplete={props.autoComplete ?? 'new-password'}
        autoFocus={props.autoFocus}
        disabled={props.disabled}
        required={props.required && !keeps}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={describedBy(id, hint, props.error)}
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
    </FieldShell>
  );
}

export const PasswordField = SecretField;

export interface SelectFieldProps extends FieldProps {
  value: string;
  onChange(value: string): void;
  options: Option[];
  placeholder?: string;
}

export function SelectField(props: SelectFieldProps) {
  const t = useT();
  const id = useFieldId(props.id);
  return (
    <FieldShell {...props} id={id}>
      <Select.Root
        size={props.size}
        value={props.value === '' ? undefined : props.value}
        onValueChange={props.onChange}
        disabled={props.disabled}
        name={props.name}
        required={props.required}
      >
        <Select.Trigger
          id={id}
          placeholder={props.placeholder ?? t('ui.form.choose')}
          aria-invalid={props.error ? true : undefined}
          aria-describedby={describedBy(id, props.hint, props.error)}
        />
        <Select.Content position="popper">
          {props.options.map((option) => (
            <Select.Item key={option.value} value={option.value}>
              {option.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
    </FieldShell>
  );
}

export interface ToggleFieldProps extends Omit<FieldProps, 'required'> {
  value: boolean;
  onChange(value: boolean): void;
  description?: ReactNode;
}

function ToggleShell({
  id,
  label,
  description,
  error,
  children,
}: {
  id: string;
  label: string;
  description?: ReactNode;
  error?: string;
  children: ReactNode;
}) {
  return (
    <div className={styles.field}>
      <Text as="label" htmlFor={id} size="2" className={styles.inline}>
        {children}
        <span>
          <Text weight="medium">{label}</Text>
          {description && (
            <Text id={`${id}-hint`} size="1" color="gray">
              {description}
            </Text>
          )}
        </span>
      </Text>
      {error && (
        <Text id={`${id}-error`} size="1" color="red" role="alert">
          {error}
        </Text>
      )}
    </div>
  );
}

export function CheckboxField(props: ToggleFieldProps) {
  const id = useFieldId(props.id);
  return (
    <ToggleShell id={id} label={props.label} description={props.description} error={props.error}>
      <Checkbox
        id={id}
        name={props.name}
        checked={props.value}
        disabled={props.disabled}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={describedBy(id, props.description, props.error)}
        onCheckedChange={(checked) => props.onChange(checked === true)}
      />
    </ToggleShell>
  );
}

export function SwitchField(props: ToggleFieldProps) {
  const id = useFieldId(props.id);
  return (
    <ToggleShell id={id} label={props.label} description={props.description} error={props.error}>
      <Switch
        id={id}
        name={props.name}
        checked={props.value}
        disabled={props.disabled}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={describedBy(id, props.description, props.error)}
        onCheckedChange={props.onChange}
      />
    </ToggleShell>
  );
}

export interface TextAreaFieldProps extends FieldProps {
  value: string;
  onChange(value: string): void;
  rows?: number;
  placeholder?: string;
  monospace?: boolean;
}

export function TextAreaField(props: TextAreaFieldProps) {
  const id = useFieldId(props.id);
  return (
    <FieldShell {...props} id={id}>
      <TextArea
        id={id}
        size={props.size}
        name={props.name}
        value={props.value}
        rows={props.rows ?? 4}
        placeholder={props.placeholder}
        disabled={props.disabled}
        required={props.required}
        style={props.monospace ? { fontFamily: 'var(--code-font-family)' } : undefined}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={describedBy(id, props.hint, props.error)}
        onChange={(event) => props.onChange(event.target.value)}
      />
    </FieldShell>
  );
}
