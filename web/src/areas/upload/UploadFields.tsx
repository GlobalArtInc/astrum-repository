import { IconButton, TextField as RadixTextField, Select, Text } from '@radix-ui/themes';
import { ChevronDown, ChevronUp, CircleAlert, File as FileIcon, Upload, X } from 'lucide-react';
import {
  type DragEvent,
  type KeyboardEvent,
  type ReactNode,
  useEffect,
  useId,
  useRef,
  useState,
} from 'react';
import { formatBytes, useI18n } from '../../i18n';
import styles from './Upload.module.css';

function Required() {
  return (
    <span className={styles.required} aria-hidden="true">
      *
    </span>
  );
}

function FieldError({ id, message }: { id: string; message: string }) {
  return (
    <Text as="p" size="1" id={id} className={styles.fieldError}>
      <CircleAlert size={12} aria-hidden="true" />
      {message}
    </Text>
  );
}

export function SettingsField({
  label,
  help,
  error,
  required,
  disabled,
  value,
  placeholder,
  name,
  onChange,
}: {
  label: string;
  help?: string;
  error?: string;
  required?: boolean;
  disabled?: boolean;
  value: string;
  placeholder?: string;
  name: string;
  onChange(value: string): void;
}) {
  const id = useId();
  return (
    <div className={styles.field}>
      <label htmlFor={id} className={styles.label}>
        {label}
        {required && <Required />}
      </label>
      {help && !error && (
        <Text as="p" size="1" id={`${id}-help`} className={styles.help}>
          {help}
        </Text>
      )}
      <RadixTextField.Root
        id={id}
        name={name}
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        autoComplete="off"
        aria-required={required || undefined}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? `${id}-error` : help ? `${id}-help` : undefined}
        onChange={(event) => onChange(event.target.value)}
      />
      {error && <FieldError id={`${id}-error`} message={error} />}
    </div>
  );
}

export function SettingsSelect({
  label,
  help,
  disabled,
  value,
  options,
  onChange,
}: {
  label: string;
  help?: string;
  disabled?: boolean;
  value: string;
  options: string[];
  onChange(value: string): void;
}) {
  const id = useId();
  return (
    <div className={styles.field}>
      <label htmlFor={id} id={`${id}-label`} className={styles.label}>
        {label}
      </label>
      {help && (
        <Text as="p" size="1" id={`${id}-help`} className={styles.help}>
          {help}
        </Text>
      )}
      <Select.Root value={value} disabled={disabled} onValueChange={onChange}>
        <Select.Trigger
          id={id}
          className={styles.selectTrigger}
          aria-labelledby={`${id}-label`}
          aria-describedby={help ? `${id}-help` : undefined}
        />
        <Select.Content position="popper">
          {options.map((option) => (
            <Select.Item key={option} value={option}>
              {option}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
    </div>
  );
}

export function SettingsCheckbox({
  label,
  description,
  checked,
  disabled,
  onChange,
}: {
  label: string;
  description?: string;
  checked: boolean;
  disabled?: boolean;
  onChange(checked: boolean): void;
}) {
  return (
    <label className={styles.checkbox} data-disabled={disabled ? 'true' : undefined}>
      <span className={styles.checkboxBox}>
        <input
          type="checkbox"
          className={styles.checkboxInput}
          checked={checked}
          disabled={disabled}
          onChange={(event) => onChange(event.target.checked)}
        />
      </span>
      <span className={styles.checkboxContent}>
        <span className={styles.checkboxLabel}>{label}</span>
        {description && (
          <Text size="1" className={styles.checkboxDescription}>
            {description}
          </Text>
        )}
      </span>
    </label>
  );
}

export interface ComboOption {
  value: string;
  description: string;
}

export function TargetCombobox({
  label,
  help,
  placeholder,
  ariaLabel,
  value,
  options,
  disabled,
  onSelect,
}: {
  label: string;
  help: string;
  placeholder: string;
  ariaLabel: string;
  value: string;
  options: ComboOption[];
  disabled?: boolean;
  onSelect(value: string): void;
}) {
  const id = useId();
  const [text, setText] = useState(value);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setText(value);
  }, [value]);

  const wanted = text.trim().toLowerCase();
  const shown =
    text === value || wanted === ''
      ? options
      : options.filter((option) => option.value.toLowerCase().includes(wanted));

  const close = () => {
    setOpen(false);
    setText(value);
  };

  const choose = (option: ComboOption | undefined) => {
    if (!option) {
      return;
    }
    setOpen(false);
    setText(option.value);
    if (option.value !== value) {
      onSelect(option.value);
    }
  };

  const keyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      if (!open) {
        setOpen(true);
        setActive(0);
      } else {
        setActive((current) => Math.min(current + 1, shown.length - 1));
      }
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive((current) => Math.max(current - 1, 0));
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (open) {
        choose(shown[active]);
      }
    } else if (event.key === 'Escape') {
      close();
    }
  };

  const Chevron = open ? ChevronUp : ChevronDown;
  return (
    <div className={styles.field}>
      <label htmlFor={id} className={styles.label}>
        {label}
        <Required />
      </label>
      <Text as="p" size="1" id={`${id}-help`} className={styles.help}>
        {help}
      </Text>
      <div className={styles.combo}>
        <RadixTextField.Root
          ref={input}
          id={id}
          value={text}
          role="combobox"
          placeholder={placeholder}
          disabled={disabled}
          autoComplete="off"
          spellCheck={false}
          aria-required
          aria-expanded={open}
          aria-controls={`${id}-list`}
          aria-autocomplete="list"
          aria-activedescendant={open && shown[active] ? `${id}-option-${active}` : undefined}
          aria-describedby={`${id}-help`}
          onChange={(event) => {
            setText(event.target.value);
            setOpen(true);
            setActive(0);
          }}
          onClick={() => setOpen(true)}
          onKeyDown={keyDown}
          onBlur={() => window.setTimeout(close, 120)}
        >
          <RadixTextField.Slot side="right" className={styles.comboChevron}>
            <Chevron size={16} aria-hidden="true" />
          </RadixTextField.Slot>
        </RadixTextField.Root>
        {open && shown.length > 0 && (
          <div id={`${id}-list`} role="listbox" aria-label={ariaLabel} className={styles.listbox}>
            {shown.map((option, index) => (
              <div
                key={option.value}
                tabIndex={-1}
                id={`${id}-option-${index}`}
                role="option"
                aria-selected={index === active}
                className={styles.option}
                data-active={index === active ? 'true' : undefined}
                onMouseEnter={() => setActive(index)}
                onMouseDown={(event) => {
                  event.preventDefault();
                  choose(option);
                }}
              >
                <span className={styles.optionLabel}>{option.value}</span>
                <span className={styles.optionDescription}>{option.description}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function Dropzone({
  label,
  help,
  file,
  accept,
  error,
  disabled,
  name,
  onChange,
}: {
  label: string;
  help: string;
  file: File | null;
  accept?: string;
  error?: string;
  disabled?: boolean;
  name: string;
  onChange(file: File | null): void;
}) {
  const { t, language } = useI18n();
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  const [dragging, setDragging] = useState(false);
  const pick = () => {
    if (!disabled) {
      input.current?.click();
    }
  };
  const drop = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setDragging(false);
    const dropped = event.dataTransfer.files[0];
    if (dropped && !disabled) {
      onChange(dropped);
    }
  };
  let content: ReactNode;
  if (file) {
    content = (
      <div className={styles.fileRow}>
        <div className={styles.fileInfo}>
          <FileIcon size={16} aria-hidden="true" />
          <Text size="2" className={styles.fileName}>
            {file.name}
          </Text>
          <Text size="1" className={styles.gray}>
            ({formatBytes(language, file.size)})
          </Text>
        </div>
        <IconButton
          type="button"
          size="1"
          variant="ghost"
          color="red"
          disabled={disabled}
          aria-label={t('upload.file.clear', { name: file.name })}
          onClick={(event) => {
            event.stopPropagation();
            onChange(null);
          }}
        >
          <X size={16} aria-hidden="true" />
        </IconButton>
      </div>
    );
  } else {
    content = (
      <div className={styles.dropEmpty}>
        <Upload size={32} aria-hidden="true" className={styles.dropIcon} />
        <Text size="2" weight="medium" className={styles.dropText}>
          {t('upload.file.drop')}
        </Text>
        <Text size="1" className={styles.gray}>
          {t('upload.file.choose')}
        </Text>
      </div>
    );
  }
  return (
    <div className={styles.dropzone}>
      <label htmlFor={id} className={styles.dropLabel}>
        {label}
        <Required />
      </label>
      <Text as="p" size="1" className={styles.gray}>
        {help}
      </Text>
      <input
        ref={input}
        id={id}
        name={name}
        type="file"
        className="visually-hidden"
        accept={accept}
        disabled={disabled}
        aria-invalid={error ? true : undefined}
        onChange={(event) => {
          onChange(event.target.files?.[0] ?? null);
          event.target.value = '';
        }}
      />
      <div
        role="button"
        tabIndex={disabled ? -1 : 0}
        aria-label={label}
        className={styles.dropArea}
        data-file={file ? 'true' : undefined}
        data-dragging={dragging ? 'true' : undefined}
        onClick={file ? undefined : pick}
        onKeyDown={(event) => {
          if (!file && (event.key === 'Enter' || event.key === ' ')) {
            event.preventDefault();
            pick();
          }
        }}
        onDragOver={(event) => {
          event.preventDefault();
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={drop}
      >
        {content}
      </div>
      {error && (
        <div role="alert" className={styles.dropError}>
          <CircleAlert size={14} aria-hidden="true" />
          <Text size="1">{error}</Text>
        </div>
      )}
    </div>
  );
}
