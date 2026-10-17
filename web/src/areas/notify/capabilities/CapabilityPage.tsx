import { Badge, Button, Flex, Spinner } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { FileText, Info, PenLine, PowerOff, Settings, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { ApiError, api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY, useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  ConfirmDialog,
  describeError,
  ErrorState,
  FormError,
  Page,
  QueryTabs,
  UnsavedChangesGuard,
  useToast,
} from '../../../ui';
import { SettingsForm, SettingsSection, settingsButton } from '../../../ui/SettingsForm';
import styles from './Capabilities.module.css';
import { CapabilityFields } from './CapabilityFields';
import {
  aboutParagraphs,
  CAPABILITIES,
  type Capability,
  type CapabilityType,
  categoryLabel,
  DELETE,
  initialCapabilityValues,
  LIST_PAGE,
  serverProblems,
  stateLabel,
  TYPES,
  toCapabilityRequest,
  UPDATE,
  validateCapability,
} from './model';

export function About({ type }: { type: CapabilityType }) {
  return (
    <div className={styles.aboutContent}>
      {aboutParagraphs(type.about).map((paragraph) => (
        <p key={paragraph}>{paragraph}</p>
      ))}
    </div>
  );
}

function Summary({ capability }: { capability: Capability }) {
  const t = useT();
  return (
    <div className={styles.summary}>
      <div className={styles.summaryItem}>
        <span className={styles.summaryLabel}>
          <FileText size={16} aria-hidden="true" />
          {t('notify.capabilities.descriptionLabel')}
        </span>
        <span className={styles.summaryValue}>{capability.description || '-'}</span>
      </div>
      <div className={styles.summaryItem}>
        <span className={styles.summaryLabel}>
          <PenLine size={16} aria-hidden="true" />
          {t('notify.capabilities.notesLabel')}
        </span>
        <span className={styles.summaryValue}>{capability.notes || '-'}</span>
      </div>
      <div className={styles.summaryItem}>
        <span className={styles.summaryLabel}>{t('notify.capabilities.categoryLabel')}</span>
        <span className={styles.summaryValue}>{categoryLabel(t, capability.tags.Category)}</span>
      </div>
    </div>
  );
}

function SettingsTab({
  capability,
  type,
  readOnly,
  onSaved,
}: {
  capability: Capability;
  type: CapabilityType;
  readOnly: boolean;
  onSaved(): Promise<void>;
}) {
  const t = useT();
  const toast = useToast();
  const [baseline, setBaseline] = useState(() => initialCapabilityValues(type, capability));
  const [values, setValues] = useState(baseline);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const dirty = JSON.stringify(values) !== JSON.stringify(baseline);

  const save = async () => {
    const problems = validateCapability(t, type, values);
    setErrors(problems);
    if (Object.values(problems).some(Boolean)) {
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      await api.put(
        `${CAPABILITIES}/${encodeURIComponent(capability.id)}`,
        toCapabilityRequest(type, values),
      );
      toast.success(t('notify.capabilities.updated', { name: capability.typeName }));
      setBaseline(values);
      await onSaved();
    } catch (error) {
      if (error instanceof ApiError && error.status === 400 && error.fieldErrors.length > 0) {
        const { fields, other } = serverProblems(type, error.fieldErrors);
        setErrors(fields);
        setFormError(other.length > 0 ? other.join(' ') : null);
      } else {
        setFormError(describeError(t, error));
      }
    } finally {
      setSaving(false);
    }
  };

  return (
    <SettingsForm
      wide
      label={t('notify.capabilities.settings')}
      onSubmit={() => void save()}
      end={
        dirty && !readOnly ? (
          <>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setValues(baseline);
                setErrors({});
                setFormError(null);
              }}
            >
              {t('notify.capabilities.cancel')}
            </Button>
            <Button type="submit" loading={saving}>
              {t('notify.capabilities.save')}
            </Button>
          </>
        ) : undefined
      }
    >
      <UnsavedChangesGuard when={dirty && !saving} />
      <FormError message={formError} />
      <CapabilityFields
        type={type}
        values={values}
        errors={errors}
        readOnly={readOnly}
        onChange={setValues}
      />
    </SettingsForm>
  );
}

function Detail({ capability, type }: { capability: Capability; type: CapabilityType }) {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const canUpdate = useCan(UPDATE);
  const canDelete = useCan(DELETE);
  const [deleting, setDeleting] = useState(false);
  const [busy, setBusy] = useState(false);
  const path = `${CAPABILITIES}/${encodeURIComponent(capability.id)}`;
  const refresh = async () => {
    await Promise.all([invalidate(CAPABILITIES), client.invalidateQueries({ queryKey: STATE_KEY })]);
  };

  const toggle = async () => {
    setBusy(true);
    try {
      await api.put(path, {
        type: capability.type,
        enabled: !capability.enabled,
        notes: capability.notes,
        properties: capability.properties,
      });
      toast.success(
        capability.enabled
          ? t('notify.capabilities.disabledToast', { name: capability.typeName })
          : t('notify.capabilities.enabledToast', { name: capability.typeName }),
      );
      await refresh();
    } catch (error) {
      toast.error(t('notify.capabilities.saveFailed'), describeError(t, error));
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    await api.delete(path);
    toast.success(t('notify.capabilities.deleted', { name: capability.typeName }));
    await refresh();
    navigate(LIST_PAGE);
  };

  const label = (Icon: typeof Info, text: string) => (
    <Flex gap="2" align="center">
      <Icon size={16} aria-hidden="true" />
      {text}
    </Flex>
  );

  return (
    <Page
      title={capability.typeName}
      titleExtra={<Badge color={capability.active ? 'green' : 'gray'}>{stateLabel(t, capability)}</Badge>}
      breadcrumbs={[
        { label: t('notify.crumbs.settings'), to: '/settings' },
        { label: t('notify.capabilities.title'), to: LIST_PAGE },
        { label: capability.typeName },
      ]}
      actions={
        <>
          {canUpdate && (
            <Button
              type="button"
              size="3"
              variant="outline"
              color="gray"
              className={settingsButton}
              loading={busy}
              onClick={() => void toggle()}
            >
              <PowerOff size={16} aria-hidden="true" />
              {capability.enabled ? t('notify.capabilities.disable') : t('notify.capabilities.enableAction')}
            </Button>
          )}
          {canDelete && (
            <Button
              type="button"
              size="3"
              color="red"
              className={settingsButton}
              onClick={() => setDeleting(true)}
            >
              <Trash2 size={16} aria-hidden="true" />
              {t('notify.capabilities.delete')}
            </Button>
          )}
        </>
      }
    >
      <QueryTabs
        className={styles.tabs}
        tabs={[
          {
            value: 'summary',
            label: label(Info, t('notify.capabilities.summary')),
            content: <Summary capability={capability} />,
          },
          {
            value: 'settings',
            label: label(Settings, t('notify.capabilities.settings')),
            content: (
              <SettingsTab
                key={JSON.stringify(capability)}
                capability={capability}
                type={type}
                readOnly={!canUpdate}
                onSaved={refresh}
              />
            ),
          },
          {
            value: 'status',
            label: label(FileText, t('notify.capabilities.status')),
            content: (
              <SettingsSection title={t('notify.capabilities.status')}>
                <span className={styles.noStatus}>
                  {capability.status ?? t('notify.capabilities.noStatus')}
                </span>
              </SettingsSection>
            ),
          },
          {
            value: 'about',
            label: label(Info, t('notify.capabilities.about')),
            content: (
              <SettingsSection title={t('notify.capabilities.about')}>
                <About type={type} />
              </SettingsSection>
            ),
          },
        ]}
      />
      <ConfirmDialog
        open={deleting}
        danger
        title={t('notify.capabilities.deleteTitle')}
        description={t('notify.capabilities.deleteText')}
        confirmLabel={t('notify.capabilities.delete')}
        onConfirm={remove}
        onOpenChange={(next) => !next && setDeleting(false)}
      />
    </Page>
  );
}

export default function CapabilityPage() {
  const t = useT();
  const { id = '' } = useParams();
  const capability = useApi<Capability>(`${CAPABILITIES}/${encodeURIComponent(id)}`);
  const types = useApi<CapabilityType[]>(TYPES);
  const error = capability.error ?? types.error;
  if (error) {
    return (
      <Page
        title={t('notify.capabilities.title')}
        breadcrumbs={[
          { label: t('notify.crumbs.settings'), to: '/settings' },
          { label: t('notify.capabilities.title'), to: LIST_PAGE },
        ]}
      >
        <ErrorState error={error} onRetry={() => void capability.refetch()} />
      </Page>
    );
  }
  const type = types.data?.find((known) => known.id === capability.data?.type);
  if (!capability.data || !type) {
    return (
      <div className={styles.loading}>
        <Spinner size="3" />
      </div>
    );
  }
  return <Detail capability={capability.data} type={type} />;
}
