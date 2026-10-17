import type { ReactNode } from 'react';
import { useT } from '../../../i18n';
import { CheckboxField, TextAreaField, TextField } from '../../../ui';
import { SettingsFields, SettingsSection } from '../../../ui/SettingsForm';
import styles from './Capabilities.module.css';
import type { CapabilityType, CapabilityValues } from './model';

export function CapabilityFields({
  type,
  values,
  errors,
  readOnly,
  onChange,
  lead,
}: {
  lead?: ReactNode;
  type: CapabilityType;
  values: CapabilityValues;
  errors: Record<string, string>;
  readOnly: boolean;
  onChange(values: CapabilityValues): void;
}) {
  const t = useT();
  const setProperty = (name: string, value: string) =>
    onChange({ ...values, properties: { ...values.properties, [name]: value } });
  return (
    <div className={styles.fields}>
      {lead}
      <SettingsSection title={t('notify.capabilities.settings')}>
        <SettingsFields>
          <CheckboxField
            label={t('notify.capabilities.enable')}
            value={values.enabled}
            disabled={readOnly}
            onChange={(enabled) => onChange({ ...values, enabled })}
          />
          {type.formFields.map((field) => {
            const value = values.properties[field.id] ?? '';
            if (field.type === 'checkbox') {
              return (
                <CheckboxField
                  key={field.id}
                  label={field.label}
                  value={value === 'true'}
                  disabled={readOnly}
                  onChange={(checked) => setProperty(field.id, checked ? 'true' : 'false')}
                />
              );
            }
            if (field.type === 'text-area') {
              return (
                <TextAreaField
                  key={field.id}
                  label={field.label}
                  hint={field.helpText ?? undefined}
                  required={field.required}
                  value={value}
                  rows={5}
                  monospace
                  disabled={readOnly}
                  error={errors[field.id]}
                  onChange={(next) => setProperty(field.id, next)}
                />
              );
            }
            return (
              <TextField
                key={field.id}
                label={field.label}
                hint={field.helpText ?? undefined}
                required={field.required}
                value={value}
                type={field.type === 'url' ? 'url' : 'text'}
                placeholder={field.type === 'url' ? 'https://...' : undefined}
                disabled={readOnly}
                error={errors[field.id]}
                onChange={(next) => setProperty(field.id, next)}
              />
            );
          })}
        </SettingsFields>
      </SettingsSection>
      <SettingsSection title={t('notify.capabilities.notes')}>
        <SettingsFields>
          <TextAreaField
            label={t('notify.capabilities.notes')}
            hint={t('notify.capabilities.notesHelp')}
            value={values.notes}
            rows={3}
            disabled={readOnly}
            onChange={(notes) => onChange({ ...values, notes })}
          />
        </SettingsFields>
      </SettingsSection>
    </div>
  );
}
