import { Button, Flex, Spinner } from '@radix-ui/themes';
import { HardDrive } from 'lucide-react';
import { useCallback, useMemo } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, useInvalidateApi } from '../../../api';
import { useT } from '../../../i18n';
import { EmptyState, ErrorState, Page, useToast } from '../../../ui';
import {
  CREATE_ADDRESS,
  configPath,
  LIST_ADDRESS,
  type StoreType,
  storeAddress,
  storeType,
  UI_BLOB_STORES,
  useUiBlobStores,
} from '../api';
import { BlobStoreForm } from '../components/BlobStoreForm';
import { type StoreDocument, template } from '../model';

const TITLES: Record<StoreType, string> = {
  file: 'blobstores.create.titleFile',
  s3: 'blobstores.create.titleS3',
  group: 'blobstores.create.titleGroup',
};

export default function CreateBlobStorePage() {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const params = useParams();
  const type = storeType(params.type ?? '');
  const stores = useUiBlobStores();
  const document = useMemo(() => (type ? template(type) : null), [type]);
  const onSaved = useCallback(
    (saved: StoreDocument) => {
      const name = String(saved.name ?? '');
      toast.success(t('blobstores.create.created', { name }));
      void invalidate(UI_BLOB_STORES);
      if (type) {
        navigate(storeAddress(type, name), { replace: true });
      }
    },
    [invalidate, navigate, t, toast, type],
  );
  const crumbs = [
    { label: t('blobstores.crumbs.settings'), to: '/settings' },
    { label: t('blobstores.crumbs.list'), to: LIST_ADDRESS },
    { label: t('blobstores.crumbs.create'), to: CREATE_ADDRESS },
  ];

  if (!type || !document) {
    return (
      <Page title={t('blobstores.edit.unsupported')} breadcrumbs={crumbs}>
        <EmptyState
          icon={HardDrive}
          title={t('blobstores.edit.unsupported')}
          description={t('blobstores.edit.unsupportedText', { type: params.type ?? '' })}
          action={
            <Button asChild variant="soft">
              <Link to={CREATE_ADDRESS}>{t('blobstores.create.back')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }

  return (
    <Page title={t(TITLES[type])} breadcrumbs={crumbs}>
      {stores.error ? (
        <ErrorState error={stores.error} onRetry={() => void stores.refetch()} />
      ) : stores.isPending ? (
        <Flex justify="center" py="9">
          <Spinner size="3" />
        </Flex>
      ) : (
        <BlobStoreForm
          type={type}
          creating
          document={document}
          stores={stores.data ?? []}
          submitLabel={t('blobstores.create.submit')}
          onSubmit={async (body) => {
            await api.post(configPath(type), body);
          }}
          onSaved={onSaved}
          onCancel={() => navigate(CREATE_ADDRESS)}
        />
      )}
    </Page>
  );
}
