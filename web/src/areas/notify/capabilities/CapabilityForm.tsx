import { Button, Callout, Checkbox, TextArea, TextField } from '@radix-ui/themes';
import { type FormEvent, useId, useState } from 'react';
import { ApiError } from '../../../api';
import { useT } from '../../../i18n';
import { describeError, UnsavedChangesGuard } from '../../../ui';
import styles from '../Notify.module.css';
import {
  type Capability,
  type CapabilityType,
  type CapabilityValues,
  initialCapabilityValues,
  validateCapability,
} from './model';

export function CapabilityForm({
  type,
  capability,
  readOnly,
  submitLabel,
  onSubmit,
  onCancel,
}: {
  type: CapabilityType;
  capability: Capability | null;
  readOnly: boolean;
  submitLabel: string;
  onSubmit(values: CapabilityValues): Promise<void>;
  onCancel?: () => void;
}) {
  const t = useT();
  const id = useId();
  const start = initialCapabilityValues(type, capability);
  const [baseline, setBaseline] = useState(start);
  const [values, setValues] = useState(start);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const dirty = JSON.stringify(values) !== JSON.stringify(baseline);
  const setProperty = (name: string, value: string) => {
    setValues((current) => ({ ...current, properties: { ...current.properties, [name]: value } }));
    setErrors((current) => ({ ...current, [name]: '' }));
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const problems = validateCapability(t, type, values);
    setErrors(problems);
    if (Object.values(problems).some(Boolean)) {
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      await onSubmit(values);
      setBaseline(values);
    } catch (error) {
      if (error instanceof ApiError && error.status === 400 && error.fieldErrors.length > 0) {
        const mapped: Record<string, string> = {};
        const other: string[] = [];
        for (const problem of error.fieldErrors) {
          const name = problem.id.replace(/^properties\./, '');
          if (type.formFields.some((field) => field.id === name)) {
            mapped[name] = problem.message;
          } else {
            other.push(problem.message);
          }
        }
        setErrors(mapped);
        setFormError(other.length > 0 ? other.join(' ') : null);
      } else {
        setFormError(describeError(t, error));
      }
    } finally {
      setSaving(false);
    }
  };

  return (
    <form className={styles.capabilityForm} onSubmit={submit} noValidate>
      <UnsavedChangesGuard when={dirty && !saving} />
      <h2 className={styles.formTitle}>{t('notify.capabilities.settings')}</h2>
      {/* biome-ignore lint/a11y/noLabelWithoutControl: the checkbox inside is the control */}
      <label className={styles.check}>
        <Checkbox
          checked={values.enabled}
          disabled={readOnly}
          onCheckedChange={(checked) => setValues((current) => ({ ...current, enabled: checked === true }))}
        />
        <span className={styles.checkTitle}>{t('notify.capabilities.enable')}</span>
      </label>
      {type.formFields.map((field) => {
        const fieldId = `${id}-${field.id}`;
        const value = values.properties[field.id] ?? '';
        const error = errors[field.id];
        if (field.type === 'checkbox') {
          return (
            // biome-ignore lint/a11y/noLabelWithoutControl: the checkbox inside is the control
            <label key={field.id} className={styles.check}>
              <Checkbox
                checked={value === 'true'}
                disabled={readOnly}
                onCheckedChange={(checked) => setProperty(field.id, checked === true ? 'true' : 'false')}
              />
              <span>
                <span className={styles.checkTitle}>{field.label}</span>
                {field.helpText && <span className={styles.checkText}>{field.helpText}</span>}
              </span>
            </label>
          );
        }
        return (
          <div key={field.id} className={styles.field}>
            <label htmlFor={fieldId} className={styles.label}>
              {field.label}
              {field.required && <span className={styles.required}> *</span>}
            </label>
            {field.helpText && <p className={styles.hint}>{field.helpText}</p>}
            {field.type === 'text-area' ? (
              <TextArea
                id={fieldId}
                value={value}
                disabled={readOnly}
                rows={5}
                className={styles.code}
                onChange={(event) => setProperty(field.id, event.target.value)}
              />
            ) : (
              <TextField.Root
                id={fieldId}
                value={value}
                disabled={readOnly}
                inputMode={field.type === 'number' ? 'numeric' : undefined}
                type={field.type === 'url' ? 'url' : 'text'}
                aria-invalid={error ? true : undefined}
                onChange={(event) => setProperty(field.id, event.target.value)}
              />
            )}
            {error && (
              <p className={styles.error} role="alert">
                {error}
              </p>
            )}
          </div>
        );
      })}
      <hr className={styles.rule} />
      <h2 className={styles.formTitle}>{t('notify.capabilities.notes')}</h2>
      <div className={styles.field}>
        <label htmlFor={`${id}-notes`} className={styles.label}>
          {t('notify.capabilities.notes')}
        </label>
        <p className={styles.hint}>{t('notify.capabilities.notesHelp')}</p>
        <TextArea
          id={`${id}-notes`}
          value={values.notes}
          disabled={readOnly}
          rows={3}
          onChange={(event) => setValues((current) => ({ ...current, notes: event.target.value }))}
        />
      </div>
      {formError && (
        <Callout.Root color="red" role="alert">
          <Callout.Text>{formError}</Callout.Text>
        </Callout.Root>
      )}
      {!readOnly && (
        <div className={styles.formActions}>
          {onCancel && (
            <Button type="button" variant="soft" color="gray" onClick={onCancel}>
              {t('notify.capabilities.cancel')}
            </Button>
          )}
          <Button type="submit" loading={saving} disabled={!dirty && capability !== null}>
            {submitLabel}
          </Button>
        </div>
      )}
    </form>
  );
}
