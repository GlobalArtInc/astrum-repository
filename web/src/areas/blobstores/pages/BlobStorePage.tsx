import { Button, Callout, Flex, Spinner, Tooltip } from '@radix-ui/themes';
import { CircleAlert, HardDrive, Info, Trash2 } from 'lucide-react';
import { useId, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useI18n } from '../../../i18n';
import { describeError, EmptyState, ErrorState, Page, useConfirm, useToast } from '../../../ui';
import { storeTypeName } from '../../repositories/components/EditTabs';
import {
  configPath,
  deletePath,
  LIST_ADDRESS,
  storeType,
  UI_BLOB_STORES,
  type UiBlobStore,
  useUiBlobStores,
} from '../api';
import styles from '../blobstores.module.css';
import { BlobStoreForm } from '../components/BlobStoreForm';
import type { StoreDocument } from '../model';

function Loading() {
  return (
    <Flex justify="center" py="9">
      <Spinner size="3" />
    </Flex>
  );
}

export function deleteRefusal(t: ReturnType<typeof useI18n>['t'], store: UiBlobStore): string | null {
  if (store.deletable) {
    return null;
  }
  const reasons: string[] = [];
  if (store.repositoryCount > 0) {
    reasons.push(
      store.repositories.length > 0
        ? t('blobstores.edit.usedByRepositories', { names: store.repositories.join(', ') })
        : t('blobstores.list.repositories', { count: store.repositoryCount }),
    );
  }
  if (store.groups.length > 0) {
    reasons.push(t('blobstores.edit.memberOf', { groups: store.groups.join(', ') }));
  }
  return `${t('blobstores.edit.cannotDelete')} ${reasons.join(' ')}`.trim();
}

export default function BlobStorePage() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const confirm = useConfirm();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const params = useParams();
  const name = params.name ?? '';
  const type = storeType(params.type ?? '');
  const canUpdate = useCan('astrum:blobstores:update');
  const canDelete = useCan('astrum:blobstores:delete');
  const list = useUiBlobStores();
  const store = list.data?.find((candidate) => candidate.name === name);
  const configAddress = type ? configPath(type, name) : null;
  const config = useApi<StoreDocument>(configAddress);
  const formId = useId();
  const [dirty, setDirty] = useState(false);
  const [version, setVersion] = useState(0);
  const crumbs = [
    { label: t('blobstores.crumbs.settings'), to: '/settings' },
    { label: t('blobstores.crumbs.list'), to: LIST_ADDRESS },
    { label: name },
  ];

  if (!type) {
    return (
      <Page title={name} breadcrumbs={crumbs}>
        <EmptyState
          icon={HardDrive}
          title={t('blobstores.edit.unsupported')}
          description={t('blobstores.edit.unsupportedText', { type: params.type ?? '' })}
        />
      </Page>
    );
  }
  if (list.isPending) {
    return (
      <Page title={name} breadcrumbs={crumbs}>
        <Loading />
      </Page>
    );
  }
  if (list.error) {
    return (
      <Page title={name} breadcrumbs={crumbs}>
        <ErrorState error={list.error} onRetry={() => void list.refetch()} />
      </Page>
    );
  }
  if (!store || storeType(store.type) !== type) {
    return (
      <Page title={t('blobstores.edit.notFoundTitle')} breadcrumbs={crumbs}>
        <EmptyState
          icon={HardDrive}
          title={t('blobstores.edit.notFoundTitle')}
          description={t('blobstores.edit.notFound', { name })}
          action={
            <Button asChild variant="soft">
              <Link to={LIST_ADDRESS}>{t('blobstores.edit.backToList')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }

  const refusal = deleteRefusal(t, store);
  const remove = async () => {
    const agreed = await confirm({
      title: t('blobstores.delete.title', { name }),
      description: t('blobstores.delete.text'),
      confirmLabel: t('blobstores.delete.confirm'),
      danger: true,
    });
    if (!agreed) {
      return;
    }
    try {
      await api.delete(deletePath(name));
      toast.success(t('blobstores.delete.deleted', { name }));
      navigate(LIST_ADDRESS, { replace: true });
      void invalidate(UI_BLOB_STORES);
    } catch (error) {
      toast.error(describeError(t, error));
    }
  };
  const deleteButton = (
    <Button color="gray" variant="soft" size="3" disabled={refusal !== null} onClick={() => void remove()}>
      <Trash2 size={16} aria-hidden="true" />
      {t('blobstores.edit.delete')}
    </Button>
  );

  return (
    <Page
      title={t('blobstores.edit.title', { name })}
      breadcrumbs={crumbs}
      description={t('blobstores.edit.subtitle', { type: storeTypeName(store.type) })}
    >
      <div className={styles.bar}>
        <span>
          {canDelete && (refusal ? <Tooltip content={refusal}>{deleteButton}</Tooltip> : deleteButton)}
        </span>
        {canUpdate && (
          <span className={styles.barRight}>
            <Button variant="outline" highContrast onClick={() => setVersion(version + 1)}>
              {t('blobstores.form.discard')}
            </Button>
            <Button type="submit" form={formId} disabled={!dirty}>
              {t('blobstores.form.save')}
            </Button>
          </span>
        )}
      </div>
      <div className={styles.body}>
        {store.unavailable && (
          <Callout.Root color="red" role="status" className={styles.callout}>
            <Callout.Icon>
              <CircleAlert size={16} />
            </Callout.Icon>
            <Callout.Text>{t('blobstores.edit.unavailable')}</Callout.Text>
          </Callout.Root>
        )}
        <Callout.Root className={styles.callout} variant="surface">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>{t('blobstores.edit.warning')}</Callout.Text>
        </Callout.Root>
        {config.error ? (
          <ErrorState error={config.error} onRetry={() => void config.refetch()} />
        ) : !config.data ? (
          <Loading />
        ) : (
          <BlobStoreForm
            key={`${config.dataUpdatedAt}-${version}`}
            type={type}
            creating={false}
            storeName={name}
            document={config.data}
            readOnly={!canUpdate}
            stores={list.data ?? []}
            submitLabel={t('blobstores.form.save')}
            formId={formId}
            onDirtyChange={setDirty}
            onSubmit={async (body) => {
              if (configAddress) {
                await api.put(configAddress, body);
              }
            }}
            onSaved={() => {
              toast.success(t('blobstores.form.saved', { name }));
              void invalidate(UI_BLOB_STORES);
              if (configAddress) {
                void invalidate(configAddress);
              }
            }}
          />
        )}
      </div>
    </Page>
  );
}
