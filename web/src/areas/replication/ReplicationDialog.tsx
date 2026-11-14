import { Button, Callout, Dialog, Flex, Text } from '@radix-ui/themes';
import { TriangleAlert } from 'lucide-react';
import { useMemo } from 'react';
import { api, useInvalidateApi } from '../../api';
import { type Translate, useT } from '../../i18n';
import {
  describeError,
  FormError,
  MultiSelectField,
  NumberField,
  type Option,
  SelectField,
  StatusBadge,
  SwitchField,
  TextField,
  useForm,
  useToast,
} from '../../ui';
import styles from '../security/shared/Form.module.css';
import {
  REPLICATIONS,
  type Replication,
  type ReplicationRemote,
  replicationPath,
  useAlertChannels,
  useSourceComponents,
  useSourceRepositories,
} from './api';
import {
  isReconcileMinutes,
  type Preview,
  previewFilters,
  RECONCILE_MINUTES,
  type ReconcileMinutes,
  type ReplicationValues,
  replicationBody,
  replicationValues,
  validateReplication,
} from './model';

const RECONCILE_LABELS: Record<ReconcileMinutes, string> = {
  5: 'replication.reconcile.every5Minutes',
  10: 'replication.reconcile.every10Minutes',
  15: 'replication.reconcile.every15Minutes',
  20: 'replication.reconcile.every20Minutes',
  30: 'replication.reconcile.every30Minutes',
  60: 'replication.reconcile.everyHour',
  120: 'replication.reconcile.every2Hours',
  180: 'replication.reconcile.every3Hours',
  240: 'replication.reconcile.every4Hours',
  360: 'replication.reconcile.every6Hours',
  480: 'replication.reconcile.every8Hours',
  720: 'replication.reconcile.every12Hours',
  1440: 'replication.reconcile.daily',
};

function reconcileOptions(t: Translate, stored: number | null): Option[] {
  const options: Option[] = RECONCILE_MINUTES.map((minutes) => ({
    value: String(minutes),
    label: t(RECONCILE_LABELS[minutes]),
  }));
  if (stored !== null && !isReconcileMinutes(stored)) {
    options.push({ value: String(stored), label: t('replication.reconcile.legacy', { minutes: stored }) });
    options.sort((left, right) => Number(left.value) - Number(right.value));
  }
  return options;
}

export interface ReplicationDialogProps {
  open: boolean;
  replication: Replication | null;
  remotes: ReplicationRemote[];
  onOpenChange(open: boolean): void;
}

interface FilterPreviewProps {
  repository: string;
  loading: boolean;
  error: unknown;
  preview: Preview;
}

function FilterPreview({ repository, loading, error, preview }: FilterPreviewProps) {
  const t = useT();
  if (repository === '') {
    return (
      <Text size="2" color="gray">
        {t('replication.replication.previewNoSource')}
      </Text>
    );
  }
  if (error !== undefined && error !== null) {
    return (
      <Text size="2" color="red" role="alert">
        {describeError(t, error)}
      </Text>
    );
  }
  if (loading) {
    return (
      <Text size="2" color="gray">
        {t('replication.replication.previewLoading')}
      </Text>
    );
  }
  if (preview.kind === 'invalid') {
    return null;
  }
  if (preview.rows.length === 0) {
    return (
      <Text size="2" color="gray">
        {t('replication.replication.previewEmpty')}
      </Text>
    );
  }
  const heading = t('replication.replication.previewTitle', { count: preview.rows.length, repository });
  return (
    <div className={styles.card}>
      <Text size="2" weight="medium">
        {heading}
      </Text>
      <ul className={styles.card} aria-label={heading}>
        {preview.rows.map((row) => (
          <li key={row.key} className={styles.actions}>
            <Text size="2" className={styles.mono}>
              {row.name}:{row.version}
            </Text>
            <StatusBadge tone={row.sent ? 'success' : 'neutral'}>
              {row.sent ? t('replication.replication.wouldSend') : t('replication.replication.wouldSkip')}
            </StatusBadge>
          </li>
        ))}
      </ul>
    </div>
  );
}

interface ReplicationFormProps {
  replication: Replication | null;
  remotes: ReplicationRemote[];
  onDone(): void;
}

function ReplicationForm({ replication, remotes, onDone }: ReplicationFormProps) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const repositories = useSourceRepositories();
  const channels = useAlertChannels();
  const form = useForm<ReplicationValues>({
    initial: replicationValues(replication),
    validate: (values) => validateReplication(t, values),
    onSubmit: async (values) => {
      const body = replicationBody(values);
      if (replication === null) {
        await api.post(REPLICATIONS, body);
      } else {
        await api.put(replicationPath(replication.id), body);
      }
      toast.success(t('replication.replications.saved', { name: body.name }));
      await invalidate(REPLICATIONS);
      onDone();
    },
  });
  const sourceComponents = useSourceComponents(form.values.sourceRepository);
  const items = sourceComponents.data?.items;
  const preview = useMemo(
    () => previewFilters(items ?? [], form.values.nameFilter, form.values.versionFilter),
    [items, form.values.nameFilter, form.values.versionFilter],
  );
  const repositoryOptions = (repositories.data ?? [])
    .filter((repository) => repository.type === 'hosted')
    .map((repository) => ({ value: repository.name, label: repository.name }));
  const remoteOptions = remotes.map((remote) => ({ value: remote.id, label: remote.name }));
  const channelOptions = (channels.data ?? []).map((channel) => ({ value: channel.id, label: channel.name }));
  const invalidFilter = preview.kind === 'invalid' ? preview.field : null;
  return (
    <form className={styles.form} onSubmit={form.submit} noValidate>
      <div className={styles.card}>
        <TextField label={t('replication.replication.name')} autoComplete="off" {...form.field('name')} />
        <SelectField
          label={t('replication.replication.source')}
          hint={t('replication.replication.sourceHint')}
          options={repositoryOptions}
          {...form.field('sourceRepository')}
        />
        <SelectField
          label={t('replication.replication.remote')}
          options={remoteOptions}
          {...form.field('remoteId')}
        />
        <TextField
          label={t('replication.replication.remoteRepository')}
          hint={t('replication.replication.remoteRepositoryHint')}
          autoComplete="off"
          {...form.field('remoteRepository')}
        />
        <SwitchField label={t('replication.replication.enabled')} {...form.field('enabled')} />
        <TextField
          label={t('replication.replication.nameFilter')}
          hint={t('replication.replication.nameFilterHint')}
          autoComplete="off"
          {...form.field('nameFilter')}
          error={
            form.errors.nameFilter ??
            (invalidFilter === 'nameFilter' ? t('replication.replication.invalidPattern') : undefined)
          }
        />
        <TextField
          label={t('replication.replication.versionFilter')}
          hint={t('replication.replication.versionFilterHint')}
          autoComplete="off"
          {...form.field('versionFilter')}
          error={
            form.errors.versionFilter ??
            (invalidFilter === 'versionFilter' ? t('replication.replication.invalidPattern') : undefined)
          }
        />
        <div className={styles.card}>
          <Text size="2" weight="medium">
            {t('replication.replication.previewHeading')}
          </Text>
          <Text size="1" color="gray">
            {t('replication.replication.serverSyntax')}
          </Text>
          <FilterPreview
            repository={form.values.sourceRepository}
            loading={sourceComponents.isPending && form.values.sourceRepository !== ''}
            error={sourceComponents.error}
            preview={preview}
          />
        </div>
        <SwitchField
          label={t('replication.replication.propagateDeletes')}
          description={t('replication.replication.propagateDeletesText')}
          {...form.field('propagateDeletes')}
        />
        {form.values.propagateDeletes && (
          <Callout.Root color="amber" role="status">
            <Callout.Icon>
              <TriangleAlert size={16} aria-hidden="true" />
            </Callout.Icon>
            <Callout.Text>{t('replication.replication.propagateDeletesWarning')}</Callout.Text>
          </Callout.Root>
        )}
        <NumberField
          label={t('replication.replication.settle')}
          hint={t('replication.replication.settleHint')}
          min={0}
          max={3600}
          step={1}
          {...form.field('settleSeconds')}
        />
        <SelectField
          label={t('replication.replication.reconcile')}
          hint={t('replication.replication.reconcileHint')}
          options={reconcileOptions(t, form.values.reconcileMinutes)}
          value={form.values.reconcileMinutes === null ? '' : String(form.values.reconcileMinutes)}
          onChange={(value) => form.set('reconcileMinutes', Number(value))}
          error={form.errors.reconcileMinutes}
        />
        <MultiSelectField
          label={t('replication.replication.channels')}
          hint={
            channels.error
              ? t('replication.replication.channelsError')
              : t('replication.replication.channelsHint')
          }
          options={channelOptions}
          {...form.field('alertChannels')}
        />
      </div>
      <FormError message={form.formError} />
      <Flex gap="2" justify="end" wrap="wrap">
        <Button type="button" variant="soft" color="gray" onClick={onDone}>
          {t('replication.form.cancel')}
        </Button>
        <Button type="submit" loading={form.submitting}>
          {t('replication.form.save')}
        </Button>
      </Flex>
    </form>
  );
}

export function ReplicationDialog({ open, replication, remotes, onOpenChange }: ReplicationDialogProps) {
  const t = useT();
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content maxWidth="720px">
        <Dialog.Title>
          {replication === null
            ? t('replication.replication.createTitle')
            : t('replication.replication.editTitle')}
        </Dialog.Title>
        <ReplicationForm replication={replication} remotes={remotes} onDone={() => onOpenChange(false)} />
      </Dialog.Content>
    </Dialog.Root>
  );
}
