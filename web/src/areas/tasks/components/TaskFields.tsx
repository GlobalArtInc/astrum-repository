import { Text } from '@radix-ui/themes';
import { useId } from 'react';
import { type Translate, useT } from '../../../i18n';
import { CheckboxField, SelectField, TextField } from '../../../ui';
import {
  ALL,
  type Choices,
  type FormField,
  fieldOptions,
  type Problems,
  type TaskValues,
  type Template,
} from '../model';
import styles from '../Tasks.module.css';

const FIELD_LABELS: Record<string, [string, string]> = {
  blobstoreName: ['tasks.fields.blobStore', 'tasks.fields.blobStoreHint'],
  blobsOlderThan: ['tasks.fields.blobsOlderThan', 'tasks.fields.blobsOlderThanHint'],
  repositoryName: ['tasks.fields.repository', 'tasks.fields.repositoryHint'],
  lastUsed: ['tasks.fields.lastUsed', 'tasks.fields.lastUsedHint'],
  deployOffset: ['tasks.fields.deployOffset', 'tasks.fields.deployOffsetHint'],
};

export function fieldLabel(t: Translate, field: FormField): string {
  const known = FIELD_LABELS[field.id];
  return known ? t(known[0]) : field.label;
}

function fieldHint(t: Translate, field: FormField): string | undefined {
  const known = FIELD_LABELS[field.id];
  return known ? t(known[1]) : (field.helpText ?? undefined);
}

export const CONDITIONS = [
  { value: 'FAILURE', label: 'tasks.settings.onFailure' },
  { value: 'SUCCESS_FAILURE', label: 'tasks.settings.onBoth' },
];

function NumberInput({
  field,
  label,
  hint,
  value,
  onChange,
  error,
  disabled,
}: {
  field: FormField;
  label: string;
  hint?: string;
  value: string;
  onChange(value: string): void;
  error?: string;
  disabled?: boolean;
}) {
  const id = useId();
  const described = [hint ? `${id}-help` : '', error ? `${id}-error` : ''].filter(Boolean).join(' ');
  return (
    <div className={styles.dynamicField}>
      <label htmlFor={id} className={styles.dynamicLabel}>
        {label}
        {field.required && (
          <span className={styles.required} aria-hidden="true">
            *
          </span>
        )}
      </label>
      <input
        id={id}
        className={styles.dynamicInput}
        type="number"
        inputMode="numeric"
        value={value}
        min={field.minValue ?? undefined}
        max={field.maxValue ?? undefined}
        placeholder={field.minValue ?? undefined}
        disabled={disabled}
        required={field.required}
        aria-invalid={error ? true : undefined}
        aria-describedby={described || undefined}
        onChange={(event) => onChange(event.target.value)}
      />
      {hint && (
        <Text id={`${id}-help`} size="1" className={styles.dynamicHelp}>
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

function StatusToggle({
  label,
  value,
  onChange,
  disabled,
}: {
  label: string;
  value: boolean;
  onChange(value: boolean): void;
  disabled?: boolean;
}) {
  const id = useId();
  return (
    <div className={styles.toggle}>
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={value}
        disabled={disabled}
        className={styles.toggleSwitch}
        onClick={() => onChange(!value)}
      >
        <span className={styles.toggleThumb} />
      </button>
      <label htmlFor={id} className={styles.toggleLabel}>
        {label}
      </label>
    </div>
  );
}

export function PropertyFields({
  template,
  values,
  onChange,
  errors,
  choices,
  disabled,
}: {
  template: Template;
  values: Record<string, string>;
  onChange(values: Record<string, string>): void;
  errors: Problems;
  choices: Choices | undefined;
  disabled?: boolean;
}) {
  const t = useT();
  if (template.formFields.length === 0) {
    return (
      <Text as="p" size="2" color="gray">
        {t('tasks.settings.noProperties')}
      </Text>
    );
  }
  const set = (id: string, value: string) => onChange({ ...values, [id]: value });
  return (
    <div className={styles.card}>
      {template.formFields.map((field) => {
        const label = fieldLabel(t, field);
        const hint = fieldHint(t, field);
        const value = values[field.id] ?? '';
        const error = errors[field.id];
        if (field.type === 'checkbox') {
          return (
            <CheckboxField
              key={field.id}
              label={label}
              description={hint}
              value={value === 'true'}
              onChange={(checked) => set(field.id, checked ? 'true' : 'false')}
              error={error}
              disabled={disabled}
            />
          );
        }
        if (field.type === 'number') {
          return (
            <NumberInput
              key={field.id}
              field={field}
              label={label}
              hint={hint}
              value={value}
              onChange={(next) => set(field.id, next)}
              error={error}
              disabled={disabled}
            />
          );
        }
        if (field.type === 'combobox') {
          const options = fieldOptions(field, choices).map((option) => ({
            value: option.value,
            label: option.all
              ? field.storeApi === 'blobstores'
                ? t('tasks.fields.allBlobStores')
                : t('tasks.fields.allRepositories')
              : option.label,
          }));
          if (value !== '' && !options.some((option) => option.value === value)) {
            options.push({ value, label: value === ALL ? t('tasks.fields.allRepositories') : value });
          }
          return (
            <SelectField
              key={field.id}
              label={label}
              hint={hint}
              required={field.required}
              value={value}
              onChange={(next) => set(field.id, next)}
              options={options}
              placeholder={
                field.storeApi === 'blobstores'
                  ? t('tasks.fields.chooseBlobStore')
                  : t('tasks.fields.chooseRepository')
              }
              error={error}
              disabled={disabled}
            />
          );
        }
        return (
          <TextField
            key={field.id}
            label={label}
            hint={hint}
            required={field.required}
            value={value}
            onChange={(next) => set(field.id, next)}
            error={error}
            disabled={disabled}
          />
        );
      })}
    </div>
  );
}

export function BasicSettings({
  values,
  set,
  errors,
  disabled,
  existing,
  typeName,
}: {
  values: TaskValues;
  set<K extends keyof TaskValues>(name: K, value: TaskValues[K]): void;
  errors: Problems;
  disabled?: boolean;
  existing?: boolean;
  typeName?: string;
}) {
  const t = useT();
  return (
    <div className={styles.section}>
      {existing ? (
        <StatusToggle
          label={t('tasks.settings.taskStatus')}
          value={values.enabled}
          onChange={(value) => set('enabled', value)}
          disabled={disabled}
        />
      ) : (
        <CheckboxField
          label={t('tasks.settings.enabled')}
          value={values.enabled}
          onChange={(value) => set('enabled', value)}
          disabled={disabled}
        />
      )}
      <TextField
        label={t('tasks.settings.name')}
        hint={existing ? t('tasks.settings.nameFixed') : t('tasks.settings.nameHint')}
        placeholder={t('tasks.settings.namePlaceholder')}
        required={!existing}
        value={values.name}
        onChange={(value) => set('name', value)}
        error={errors.name}
        disabled={disabled || existing}
        maxLength={200}
      />
      {existing && typeName !== undefined && (
        <TextField
          label={t('tasks.settings.type')}
          hint={t('tasks.settings.typeFixed')}
          value={typeName}
          onChange={() => {}}
          disabled
        />
      )}
      <TextField
        type="email"
        label={existing ? t('tasks.settings.notificationEmail') : t('tasks.settings.email')}
        hint={existing ? t('tasks.settings.notificationEmailHint') : t('tasks.settings.emailHint')}
        placeholder={existing ? t('tasks.settings.emailExample') : t('tasks.settings.emailPlaceholder')}
        value={values.alertEmail}
        onChange={(value) => set('alertEmail', value)}
        error={errors.alertEmail}
        disabled={disabled}
      />
      {!existing && (
        <SelectField
          label={t('tasks.settings.sendNotification')}
          hint={t('tasks.settings.sendNotificationHint')}
          value={values.notificationCondition}
          onChange={(value) => set('notificationCondition', value)}
          options={CONDITIONS.map((condition) => ({ value: condition.value, label: t(condition.label) }))}
          disabled={disabled}
        />
      )}
    </div>
  );
}
