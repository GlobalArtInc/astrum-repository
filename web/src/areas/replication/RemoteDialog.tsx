import { Button, Callout, Dialog, Flex } from '@radix-ui/themes';
import { TriangleAlert } from 'lucide-react';
import { ApiError, api, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import { FormError, NumberField, SecretField, SwitchField, TextField, useForm, useToast } from '../../ui';
import styles from '../security/shared/Form.module.css';
import { REMOTES, type ReplicationRemote, remotePath } from './api';
import { type RemoteValues, remoteBody, remoteUrlChanged, remoteValues, validateRemote } from './model';

const BANDWIDTH_FIELD = 'bandwidthMiB';
const SERVER_BANDWIDTH = 'maxBytesPerSecond';

export interface RemoteDialogProps {
  open: boolean;
  remote: ReplicationRemote | null;
  onOpenChange(open: boolean): void;
}

function withFormFields(failure: ApiError): ApiError {
  const fieldErrors = failure.fieldErrors.map((problem) =>
    problem.id === SERVER_BANDWIDTH ? { ...problem, id: BANDWIDTH_FIELD } : problem,
  );
  return new ApiError(failure.status, failure.message, { fieldErrors });
}

function RemoteForm({ remote, onDone }: { remote: ReplicationRemote | null; onDone(): void }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const form = useForm<RemoteValues>({
    initial: remoteValues(remote),
    validate: (values) => validateRemote(t, values, remote),
    onSubmit: async (values) => {
      const body = remoteBody(values);
      try {
        if (remote === null) {
          await api.post(REMOTES, body);
        } else {
          await api.put(remotePath(remote.id), body);
        }
      } catch (failure) {
        if (failure instanceof ApiError && failure.status === 400) {
          throw withFormFields(failure);
        }
        throw failure;
      }
      toast.success(t('replication.remotes.saved', { name: values.name.trim() }));
      await invalidate(REMOTES);
      onDone();
    },
  });
  const urlChanged = remoteUrlChanged(form.values.url, remote);
  const tokenKept = remote?.hasToken === true && !urlChanged;
  const tokenHint = urlChanged
    ? t('replication.remote.tokenReason')
    : tokenKept
      ? t('replication.remote.tokenKept')
      : undefined;
  return (
    <form className={styles.form} onSubmit={form.submit} noValidate>
      <div className={styles.card}>
        <TextField label={t('replication.remote.name')} autoComplete="off" {...form.field('name')} />
        <TextField
          label={t('replication.remote.url')}
          hint={t('replication.remote.urlHint')}
          placeholder="https://astrum.example.com:8443"
          autoComplete="off"
          {...form.field('url')}
        />
        <SecretField
          label={t('replication.remote.token')}
          stored={tokenKept}
          required={remote === null || urlChanged}
          hint={tokenHint}
          autoComplete="off"
          {...form.field('token')}
        />
        <SwitchField
          label={t('replication.remote.trustStore')}
          description={t('replication.remote.trustStoreText')}
          {...form.field('useTrustStore')}
        />
        <SwitchField
          label={t('replication.remote.privateNetwork')}
          description={t('replication.remote.privateNetworkText')}
          {...form.field('allowPrivateNetwork')}
        />
        {form.values.allowPrivateNetwork && (
          <Callout.Root color="amber" role="status">
            <Callout.Icon>
              <TriangleAlert size={16} aria-hidden="true" />
            </Callout.Icon>
            <Callout.Text>{t('replication.remote.privateNetworkWarning')}</Callout.Text>
          </Callout.Root>
        )}
        <NumberField
          label={t('replication.remote.concurrency')}
          hint={t('replication.remote.concurrencyHint')}
          min={1}
          max={16}
          step={1}
          {...form.field('maxConcurrency')}
        />
        <NumberField
          label={t('replication.remote.bandwidth')}
          hint={t('replication.remote.bandwidthHint')}
          placeholder={t('replication.remote.unlimited')}
          min={0}
          step={0.01}
          {...form.field('bandwidthMiB')}
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

export function RemoteDialog({ open, remote, onOpenChange }: RemoteDialogProps) {
  const t = useT();
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content maxWidth="560px">
        <Dialog.Title>
          {remote === null ? t('replication.remote.createTitle') : t('replication.remote.editTitle')}
        </Dialog.Title>
        <RemoteForm remote={remote} onDone={() => onOpenChange(false)} />
      </Dialog.Content>
    </Dialog.Root>
  );
}
