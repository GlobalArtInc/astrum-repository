import { Button, Dialog, Flex } from '@radix-ui/themes';
import { Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  describeError,
  ErrorState,
  FormError,
  Page,
  TextAreaField,
  TextField,
  useConfirm,
  useForm,
  useToast,
} from '../../../ui';
import { TRUSTED_KEYS } from '../api';
import { type ListColumn, ListTable, type RowButton } from '../shared/AdminList';
import list from '../shared/AdminList.module.css';
import styles from '../shared/Form.module.css';
import { explainRefusal } from '../shared/messages';
import type { TrustedKey } from './model';
import { POLICY_CREATE, POLICY_DELETE, POLICY_READ } from './permissions';

interface KeyForm {
  name: string;
  publicKey: string;
  notes: string;
}

const EMPTY_KEY_FORM: KeyForm = { name: '', publicKey: '', notes: '' };

interface KeyFormProps {
  onDone(): void;
}

function KeyFormFields({ onDone }: KeyFormProps) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const form = useForm<KeyForm>({
    initial: EMPTY_KEY_FORM,
    onSubmit: (values) =>
      explainRefusal(t, async () => {
        const saved = await api.post<TrustedKey>(TRUSTED_KEYS, values);
        toast.success(t('security.keys.added', { name: saved.name }), saved.fingerprint);
        await invalidate(TRUSTED_KEYS);
        onDone();
      }),
  });
  return (
    <form onSubmit={form.submit} noValidate>
      <Flex direction="column" gap="4">
        <div className={styles.card}>
          <TextField label={t('security.keys.name')} {...form.field('name')} required autoFocus />
          <TextAreaField
            label={t('security.keys.publicKey')}
            {...form.field('publicKey')}
            hint={t('security.keys.publicKeyHint')}
            placeholder="-----BEGIN PUBLIC KEY-----"
            monospace
            required
          />
          <TextAreaField label={t('security.keys.notes')} {...form.field('notes')} />
        </div>
        <FormError message={form.formError} />
        <Flex gap="3" justify="end" wrap="wrap">
          <Dialog.Close>
            <Button type="button" variant="soft" color="gray">
              {t('security.form.cancel')}
            </Button>
          </Dialog.Close>
          <Button type="submit" loading={form.submitting}>
            {t('security.keys.add')}
          </Button>
        </Flex>
      </Flex>
    </form>
  );
}

export default function TrustedKeysPage() {
  const t = useT();
  const toast = useToast();
  const confirm = useConfirm();
  const invalidate = useInvalidateApi();
  const canRead = useCan(POLICY_READ);
  const canCreate = useCan(POLICY_CREATE);
  const canDelete = useCan(POLICY_DELETE);
  const keys = useApi<TrustedKey[]>(canRead ? TRUSTED_KEYS : null);
  const [adding, setAdding] = useState(false);
  const columns: ListColumn<TrustedKey>[] = [
    {
      id: 'name',
      header: t('security.keys.columns.name'),
      cell: (key) => key.name,
    },
    {
      id: 'algorithm',
      header: t('security.keys.columns.algorithm'),
      cell: (key) => key.algorithm,
    },
    {
      id: 'fingerprint',
      header: t('security.keys.columns.fingerprint'),
      cell: (key) => <span className={styles.mono}>{key.fingerprint}</span>,
    },
  ];
  const remove = async (key: TrustedKey) => {
    const agreed = await confirm({
      title: t('security.keys.deleteTitle', { name: key.name }),
      description: t('security.keys.deleteText'),
      confirmLabel: t('security.form.delete'),
      danger: true,
    });
    if (!agreed) {
      return;
    }
    try {
      await api.delete(`${TRUSTED_KEYS}${path`/${key.name}`}`);
      toast.success(t('security.keys.deleted', { name: key.name }));
      await invalidate(TRUSTED_KEYS);
    } catch (error) {
      toast.error(t('security.keys.deleteRefused'), describeError(t, error));
    }
  };
  const rowButtons = (key: TrustedKey): RowButton[] =>
    canDelete
      ? [{ label: t('security.form.delete'), icon: Trash2, onClick: () => void remove(key), danger: true }]
      : [];
  return (
    <Page
      title={t('security.keys.title')}
      description={t('security.keys.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.keys.title') },
      ]}
      actions={
        canCreate && (
          <Button size="3" className={list.bigButton} onClick={() => setAdding(true)}>
            <Plus size={16} aria-hidden="true" />
            {t('security.keys.add')}
          </Button>
        )
      }
    >
      {keys.error ? (
        <ErrorState error={keys.error} onRetry={() => void keys.refetch()} />
      ) : (
        <ListTable
          plain
          label={t('security.keys.title')}
          columns={columns}
          rows={keys.data ?? []}
          loading={keys.isPending}
          getRowId={(key) => key.name}
          buttons={rowButtons}
          empty={<div className={list.count}>{t('security.keys.empty')}</div>}
        />
      )}
      <Dialog.Root open={adding} onOpenChange={setAdding}>
        <Dialog.Content maxWidth="560px">
          <Dialog.Title>{t('security.keys.addTitle')}</Dialog.Title>
          <Dialog.Description size="2" color="gray">
            {t('security.keys.intro')}
          </Dialog.Description>
          {adding && <KeyFormFields onDone={() => setAdding(false)} />}
        </Dialog.Content>
      </Dialog.Root>
    </Page>
  );
}
