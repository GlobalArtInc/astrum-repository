import { Button, IconButton, Text } from '@radix-ui/themes';
import { File as FileIcon, FileUp, X } from 'lucide-react';
import { type DragEvent, useId, useRef, useState } from 'react';
import { useT } from '../../i18n';
import { FileSize } from '../../ui';
import styles from './Upload.module.css';

export interface FilePickerProps {
  label: string;
  name: string;
  file: File | null;
  accept?: string;
  required?: boolean;
  error?: string;
  hint?: string;
  disabled?: boolean;
  onChange(file: File | null): void;
}

export function FilePicker({
  label,
  name,
  file,
  accept,
  required,
  error,
  hint,
  disabled,
  onChange,
}: FilePickerProps) {
  const t = useT();
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  const [dragging, setDragging] = useState(false);
  const described = [hint ? `${id}-hint` : '', error ? `${id}-error` : ''].filter(Boolean).join(' ');

  const drop = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setDragging(false);
    const dropped = event.dataTransfer.files[0];
    if (dropped && !disabled) {
      onChange(dropped);
    }
  };

  return (
    <div className={styles.picker}>
      <Text as="label" htmlFor={id} size="2" weight="medium">
        {label}
        {required && (
          <span className={styles.required} aria-hidden="true">
            *
          </span>
        )}
      </Text>
      <div
        role="group"
        className={styles.drop}
        data-dragging={dragging ? 'true' : undefined}
        data-invalid={error ? 'true' : undefined}
        onDragOver={(event) => {
          event.preventDefault();
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={drop}
      >
        <input
          ref={input}
          id={id}
          name={name}
          type="file"
          className="visually-hidden"
          accept={accept}
          disabled={disabled}
          aria-invalid={error ? true : undefined}
          aria-describedby={described || undefined}
          onChange={(event) => {
            onChange(event.target.files?.[0] ?? null);
            event.target.value = '';
          }}
        />
        {file ? (
          <div className={styles.chosen}>
            <FileIcon size={18} aria-hidden="true" color="var(--accent-11)" />
            <span className={styles.chosenName}>{file.name}</span>
            <Text size="1" color="gray">
              <FileSize bytes={file.size} />
            </Text>
            <Button
              type="button"
              size="1"
              variant="soft"
              color="gray"
              disabled={disabled}
              onClick={() => input.current?.click()}
            >
              {t('upload.file.replace')}
            </Button>
            <IconButton
              type="button"
              size="1"
              variant="ghost"
              color="gray"
              disabled={disabled}
              aria-label={t('upload.file.clear', { name: file.name })}
              onClick={() => onChange(null)}
            >
              <X size={14} aria-hidden="true" />
            </IconButton>
          </div>
        ) : (
          <button
            type="button"
            className={styles.dropButton}
            disabled={disabled}
            onClick={() => input.current?.click()}
            aria-hidden="true"
            tabIndex={-1}
          >
            <FileUp size={22} aria-hidden="true" />
            <span>{t('upload.file.drop')}</span>
            <span className={styles.dropAction}>{t('upload.file.choose')}</span>
          </button>
        )}
      </div>
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
