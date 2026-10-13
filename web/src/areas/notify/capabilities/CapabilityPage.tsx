import { Badge, Button, Card, Spinner } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { FileText, Info, NotebookPen, Power, Settings, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY, useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ConfirmDialog, ErrorState, Page, QueryTabs, useToast } from '../../../ui';
import styles from '../Notify.module.css';
import { CapabilityForm } from './CapabilityForm';
import {
  aboutParagraphs,
  CAPABILITIES,
  type Capability,
  type CapabilityType,
  type CapabilityValues,
  categoryLabel,
  DELETE,
  LIST_PAGE,
  stateLabel,
  TYPES,
  toCapabilityRequest,
  UPDATE,
} from './model';

export function About({ type }: { type: CapabilityType }) {
  return (
    <div className={styles.aboutText}>
      {aboutParagraphs(type.about).map((paragraph) => (
        <p key={paragraph}>{paragraph}</p>
      ))}
    </div>
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
  const refresh = () =>
    Promise.all([invalidate(CAPABILITIES), client.invalidateQueries({ queryKey: STATE_KEY })]);

  const save = async (values: CapabilityValues) => {
    await api.put(path, toCapabilityRequest(type, values));
    toast.success(t('notify.capabilities.updated', { name: capability.typeName }));
    await refresh();
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
      toast.error(t('notify.capabilities.saveFailed'), error instanceof Error ? error.message : undefined);
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
        <div className={styles.headerActions}>
          {canUpdate && (
            <Button variant="outline" color="gray" loading={busy} onClick={() => void toggle()}>
              <Power size={16} aria-hidden="true" />
              {capability.enabled ? t('notify.capabilities.disable') : t('notify.capabilities.enableAction')}
            </Button>
          )}
          {canDelete && (
            <Button color="red" onClick={() => setDeleting(true)}>
              <Trash2 size={16} aria-hidden="true" />
              {t('notify.capabilities.delete')}
            </Button>
          )}
        </div>
      }
    >
      <QueryTabs
        tabs={[
          {
            value: 'summary',
            label: (
              <span className={styles.tabLabel}>
                <Info size={14} aria-hidden="true" />
                {t('notify.capabilities.summary')}
              </span>
            ),
            content: (
              <Card className={styles.summary}>
                <div>
                  <p className={styles.summaryLabel}>
                    <FileText size={14} aria-hidden="true" />
                    {t('notify.capabilities.descriptionLabel')}
                  </p>
                  <p>{capability.description ?? '-'}</p>
                </div>
                <div>
                  <p className={styles.summaryLabel}>
                    <NotebookPen size={14} aria-hidden="true" />
                    {t('notify.capabilities.notesLabel')}
                  </p>
                  <p>{capability.notes || '-'}</p>
                </div>
                <div>
                  <p className={styles.summaryLabel}>{t('notify.capabilities.categoryLabel')}</p>
                  <p>{categoryLabel(t, capability.tags.Category)}</p>
                </div>
              </Card>
            ),
          },
          {
            value: 'settings',
            label: (
              <span className={styles.tabLabel}>
                <Settings size={14} aria-hidden="true" />
                {t('notify.capabilities.settings')}
              </span>
            ),
            content: (
              <CapabilityForm
                key={JSON.stringify(capability)}
                type={type}
                capability={capability}
                readOnly={!canUpdate}
                submitLabel={t('notify.capabilities.save')}
                onSubmit={save}
              />
            ),
          },
          {
            value: 'status',
            label: (
              <span className={styles.tabLabel}>
                <FileText size={14} aria-hidden="true" />
                {t('notify.capabilities.status')}
              </span>
            ),
            content: (
              <div className={styles.tabBody}>
                <h2 className={styles.formTitle}>{t('notify.capabilities.status')}</h2>
                <p className={styles.muted}>{capability.status ?? t('notify.capabilities.noStatus')}</p>
              </div>
            ),
          },
          {
            value: 'about',
            label: (
              <span className={styles.tabLabel}>
                <Info size={14} aria-hidden="true" />
                {t('notify.capabilities.about')}
              </span>
            ),
            content: (
              <div className={styles.tabBody}>
                <h2 className={styles.formTitle}>{t('notify.capabilities.about')}</h2>
                <About type={type} />
              </div>
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
