import { Button, Flex, Spinner } from '@radix-ui/themes';
import { PackageX } from 'lucide-react';
import { useCallback, useMemo } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, type RepositoryType, useInvalidateApi } from '../../../api';
import { useSession } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import { EmptyState, ErrorState, formatName, Page, useToast } from '../../../ui';
import {
  CREATE_ADDRESS,
  createPath,
  editAddress,
  LIST_ADDRESS,
  UI_REPOSITORIES,
  useRecipes,
  useUiRepositories,
} from '../api';
import { type BlobStoreChoice, RepositoryForm } from '../components/RepositoryForm';
import { useBlobStoreChoices } from '../components/useBlobStoreChoices';
import type { RepositoryDocument } from '../model/document';
import { template } from '../model/form';
import { apiFormat, formatFromUrl, isRepositoryType, type Spec } from '../model/spec';

const TITLES: Record<RepositoryType, string> = {
  hosted: 'repositories.create.titleHosted',
  proxy: 'repositories.create.titleProxy',
  group: 'repositories.create.titleGroup',
};

export function createTitle(t: Translate, spec: Spec): string {
  return t(TITLES[spec.type], { format: formatName(spec.format) });
}

export function defaultBlobStore(stores: readonly BlobStoreChoice[] | null): string {
  if (!stores) {
    return 'default';
  }
  const usable = stores.filter((store) => store.groups.length === 0);
  return (usable.find((store) => store.name === 'default') ?? usable[0])?.name ?? '';
}

function Loading() {
  return (
    <Flex justify="center" py="9">
      <Spinner size="3" />
    </Flex>
  );
}

export default function CreateRepositoryPage() {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const params = useParams();
  const { state, can } = useSession();
  const format = formatFromUrl(params.format ?? '', state.formats);
  const type = params.type ?? '';
  const recipes = useRecipes();
  const repositories = useUiRepositories();
  const stores = useBlobStoreChoices();
  const known =
    isRepositoryType(type) &&
    can({ some: `astrum:repository-admin:${format}:*:add` }) &&
    (recipes.data?.some((recipe) => recipe.format === format && recipe.type === type) ?? true);
  const kind = isRepositoryType(type) ? type : null;
  const spec: Spec | null = kind ? { format, type: kind } : null;
  const storesReady = stores.ready;
  const blobStore = defaultBlobStore(stores.choices);
  const document = useMemo(
    () => (kind && storesReady ? template({ format, type: kind }, blobStore) : null),
    [format, kind, storesReady, blobStore],
  );
  const onSaved = useCallback(
    (saved: RepositoryDocument) => {
      const name = String(saved.name ?? '');
      toast.success(t('repositories.create.created', { name }));
      void invalidate(UI_REPOSITORIES);
      navigate(editAddress(name), { replace: true });
    },
    [invalidate, navigate, t, toast],
  );

  const crumbs = [
    { label: t('repositories.crumbs.settings'), to: '/settings' },
    { label: t('repositories.crumbs.repositories'), to: LIST_ADDRESS },
    { label: t('repositories.crumbs.create'), to: CREATE_ADDRESS },
  ];

  if (!spec || !known) {
    return (
      <Page title={t('repositories.create.unknown')} breadcrumbs={crumbs}>
        <EmptyState
          icon={PackageX}
          title={t('repositories.create.unknown')}
          description={t('repositories.create.unknownText', { format: formatName(format), type })}
          action={
            <Button asChild variant="soft">
              <Link to={CREATE_ADDRESS}>{t('repositories.create.back')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }

  const title = createTitle(t, spec);
  const failure = recipes.error ?? repositories.error;
  return (
    <Page title={title} breadcrumbs={[...crumbs, { label: formatName(spec.format) }]}>
      {failure ? (
        <ErrorState
          error={failure}
          onRetry={() => {
            void recipes.refetch();
            void repositories.refetch();
          }}
        />
      ) : !document || recipes.isPending || repositories.isPending ? (
        <Loading />
      ) : (
        <RepositoryForm
          spec={spec}
          creating
          document={document}
          repositories={repositories.data ?? []}
          blobStores={stores.choices}
          submitLabel={t('repositories.create.submit')}
          onSubmit={async (body) => {
            await api.post(createPath(apiFormat(spec.format, state.formats), spec.type), body);
          }}
          onSaved={onSaved}
          onCancel={() => navigate(CREATE_ADDRESS)}
        />
      )}
    </Page>
  );
}
