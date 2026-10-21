import { Button, Flex, Spinner } from '@radix-ui/themes';
import { FileText, ListChecks, Lock, PackageSearch, ScrollText, Settings } from 'lucide-react';
import { useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, useApi, useInvalidateApi } from '../../../api';
import { useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import { EmptyState, ErrorState, Page, useToast } from '../../../ui';
import { documentPath, LIST_ADDRESS, UI_REPOSITORIES, type UiRepository, useUiRepositories } from '../api';
import { AccessTab } from '../components/AccessTab';
import { AuditCard, SummaryTab, TasksTab } from '../components/EditTabs';
import { ProfileTabs } from '../components/ProfileTabs';
import { RepositoryForm } from '../components/RepositoryForm';
import { RepositoryBadges, RepositoryButtons } from '../components/RepositoryHeader';
import headerStyles from '../components/RepositoryHeader.module.css';
import { useRepositoryActions } from '../components/useRepositoryActions';
import type { RepositoryDocument } from '../model/document';
import { apiFormat, type Spec } from '../model/spec';

export function findRepository(
  rows: readonly UiRepository[] | undefined,
  name: string,
): UiRepository | undefined {
  if (!rows) {
    return undefined;
  }
  return (
    rows.find((row) => row.name === name) ?? rows.find((row) => row.name.toLowerCase() === name.toLowerCase())
  );
}

export function Loading() {
  return (
    <Flex justify="center" py="9">
      <Spinner size="3" />
    </Flex>
  );
}

export function TabLabel({ icon: Icon, label }: { icon: typeof FileText; label: string }) {
  return (
    <span className={headerStyles.tabLabel}>
      <Icon size={16} aria-hidden="true" />
      {label}
    </span>
  );
}

export default function RepositoryPage() {
  const t = useT();
  const navigate = useNavigate();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const { name = '' } = useParams();
  const { state, can } = useSession();
  const list = useUiRepositories();
  const repository = findRepository(list.data, name);
  const visible = repository?.permissions.admin === true;
  const docPath =
    repository && visible
      ? documentPath(apiFormat(repository.format, state.formats), repository.type, repository.name)
      : null;
  const document = useApi<RepositoryDocument>(docPath);
  const [dirty, setDirty] = useState(false);
  const actions = useRepositoryActions({ onDeleted: () => navigate(LIST_ADDRESS, { replace: true }) });
  const crumbs = [
    { label: t('repositories.crumbs.settings'), to: '/settings' },
    { label: t('repositories.crumbs.repositories'), to: LIST_ADDRESS },
    { label: repository?.name ?? name },
  ];

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
  if (!repository || !visible) {
    return (
      <Page title={t('repositories.edit.notFoundTitle')} breadcrumbs={crumbs}>
        <EmptyState
          icon={PackageSearch}
          title={t('repositories.edit.notFoundTitle')}
          description={t('repositories.edit.notFound', { name })}
          action={
            <Button asChild variant="soft">
              <Link to={LIST_ADDRESS}>{t('repositories.edit.backToList')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }

  const spec: Spec = { format: repository.format, type: repository.type };
  const online = document.data ? document.data.online !== false : repository.online;
  const current = { ...repository, online };

  const settings = document.error ? (
    <ErrorState error={document.error} onRetry={() => void document.refetch()} />
  ) : !document.data ? (
    <Loading />
  ) : (
    <RepositoryForm
      key={document.dataUpdatedAt}
      spec={spec}
      creating={false}
      document={document.data}
      readOnly={!repository.permissions.adminEdit}
      repositories={list.data ?? []}
      blobStores={null}
      submitLabel={t('repositories.form.save')}
      onDirtyChange={setDirty}
      onCancel={() => navigate(LIST_ADDRESS)}
      onSubmit={async (body) => {
        if (docPath) {
          await api.put(docPath, body);
        }
      }}
      onSaved={() => {
        toast.success(t('repositories.form.saved', { name: repository.name }));
        void Promise.all([invalidate(UI_REPOSITORIES), docPath ? invalidate(docPath) : Promise.resolve()]);
      }}
    />
  );

  return (
    <Page
      title={repository.name}
      breadcrumbs={crumbs}
      description={<RepositoryBadges repository={repository} online={online} />}
      actions={
        <RepositoryButtons
          repository={repository}
          online={online}
          busy={dirty || !document.data}
          onRebuild={() => void actions.maintain(repository, 'rebuild-index')}
          onInvalidate={() => void actions.maintain(repository, 'invalidate-cache')}
          onToggleOnline={() => void actions.toggleOnline(current)}
          onDelete={() => actions.remove(repository)}
        />
      }
    >
      <ProfileTabs
        defaultTab="settings"
        tabs={[
          {
            value: 'summary',
            label: <TabLabel icon={FileText} label={t('repositories.edit.tabs.summary')} />,
            content: <SummaryTab repository={repository} online={online} />,
          },
          {
            value: 'settings',
            label: <TabLabel icon={Settings} label={t('repositories.edit.tabs.settings')} />,
            content: settings,
            keepMounted: true,
          },
          ...(can('astrum:audit:read')
            ? [
                {
                  value: 'audit',
                  label: <TabLabel icon={ScrollText} label={t('repositories.edit.tabs.audit')} />,
                  content: <AuditCard name={repository.name} />,
                },
              ]
            : []),
          {
            value: 'tasks',
            label: <TabLabel icon={ListChecks} label={t('repositories.edit.tabs.tasks')} />,
            content: <TasksTab repository={repository.name} />,
          },
          {
            value: 'access',
            label: <TabLabel icon={Lock} label={t('repositories.edit.tabs.access')} />,
            content: <AccessTab repository={repository} />,
          },
        ]}
      />
      {actions.dialog}
    </Page>
  );
}
