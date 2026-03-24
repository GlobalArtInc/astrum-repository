import { Button, Flex, Spinner } from '@radix-ui/themes';
import { PackageX } from 'lucide-react';
import { useCallback, useId, useMemo, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, type RepositoryType, useInvalidateApi } from '../../../api';
import { useSession } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import { EmptyState, ErrorState, formatName, Page, useToast } from '../../../ui';
import {
  CREATE_ADDRESS,
  createPath,
  LIST_ADDRESS,
  UI_REPOSITORIES,
  useRecipes,
  useUiRepositories,
} from '../api';
import { type BlobStoreChoice, RepositoryForm } from '../components/RepositoryForm';
import { SaveError } from '../components/SaveError';
import { useBlobStoreChoices } from '../components/useBlobStoreChoices';
import { StepTitle, WizardBar } from '../components/Wizard';
import wizard from '../components/Wizard.module.css';
import type { RepositoryDocument } from '../model/document';
import { template } from '../model/form';
import { apiFormat, formatFromUrl, isRepositoryType, type Spec } from '../model/spec';

const TYPE_NAMES: Record<RepositoryType, string> = {
  hosted: 'ui.repositoryType.hosted',
  proxy: 'ui.repositoryType.proxy',
  group: 'ui.repositoryType.group',
};

export function createTitle(t: Translate, spec: Spec): string {
  return t('repositories.create.formTitle', {
    format: formatName(spec.format),
    type: t(TYPE_NAMES[spec.type]),
  });
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
  const formId = useId();
  const [ready, setReady] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const onSaved = useCallback(
    (saved: RepositoryDocument) => {
      const name = String(saved.name ?? '');
      toast.success(t('repositories.create.created', { name }));
      void invalidate(UI_REPOSITORIES);
      navigate(LIST_ADDRESS, { replace: true });
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
    <Page
      title={title}
      breadcrumbs={crumbs.map((crumb, index) => (index === 2 ? { label: crumb.label } : crumb))}
    >
      <SaveError message={saveError} onDismiss={() => setSaveError(null)} />
      <WizardBar
        step={3}
        onCancel={() => navigate(LIST_ADDRESS)}
        onBack={() => navigate(`${CREATE_ADDRESS}?format=${encodeURIComponent(spec.format)}`)}
        next={
          <Button type="submit" form={formId} disabled={!ready}>
            {t('repositories.create.next')}
          </Button>
        }
      />
      <div className={wizard.body}>
        <StepTitle title={t('repositories.create.configure')} />
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
            formId={formId}
            onReadyChange={setReady}
            onFailure={setSaveError}
            onSubmit={async (body) => {
              await api.post(createPath(apiFormat(spec.format, state.formats), spec.type), body);
            }}
            onSaved={onSaved}
            onCancel={() => navigate(CREATE_ADDRESS)}
          />
        )}
      </div>
    </Page>
  );
}
