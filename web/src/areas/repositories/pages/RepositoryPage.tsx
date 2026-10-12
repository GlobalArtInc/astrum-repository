import { Button, Card, DropdownMenu, Flex, IconButton, Spinner, Tooltip } from '@radix-ui/themes';
import {
  DatabaseZap,
  FolderOpen,
  MoreHorizontal,
  PackageSearch,
  Power,
  RefreshCw,
  Trash2,
} from 'lucide-react';
import { useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  describeError,
  EmptyState,
  ErrorState,
  FileSize,
  FormatBadge,
  Page,
  RelativeTime,
  TypeBadge,
  useConfirm,
  useToast,
} from '../../../ui';
import {
  browseAddress,
  documentPath,
  LIST_ADDRESS,
  repositoryPath,
  UI_REPOSITORIES,
  type UiRepository,
  useUiRepositories,
} from '../api';
import { AuditTab } from '../components/AuditTab';
import { Count, RepositoryUrl, Status } from '../components/bits';
import { DeleteRepositoryDialog } from '../components/DeleteRepositoryDialog';
import { MembershipTab } from '../components/MembershipTab';
import { ProfileTabs } from '../components/ProfileTabs';
import { RepositoryForm } from '../components/RepositoryForm';
import type { RepositoryDocument } from '../model/document';
import { documentMembers } from '../model/form';
import { apiFormat, type Spec } from '../model/spec';
import styles from '../repositories.module.css';

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

function Loading() {
  return (
    <Flex justify="center" py="9">
      <Spinner size="3" />
    </Flex>
  );
}

function Summary({ repository }: { repository: UiRepository }) {
  const t = useT();
  return (
    <Card size="2">
      <dl className={styles.summary} aria-label={t('repositories.edit.summary')}>
        <div className={styles.summaryWide}>
          <dt>{t('repositories.edit.url')}</dt>
          <dd>
            <RepositoryUrl name={repository.name} url={repository.url} />
          </dd>
        </div>
        {repository.remoteUrl && (
          <div className={styles.summaryWide}>
            <dt>{t('repositories.edit.remote')}</dt>
            <dd>{repository.remoteUrl}</dd>
          </div>
        )}
        <div>
          <dt>{t('repositories.edit.blobStore')}</dt>
          <dd>{repository.blobStoreName ?? '—'}</dd>
        </div>
        <div>
          <dt>{t('repositories.edit.size')}</dt>
          <dd>
            {repository.totalSize === null ? (
              t('repositories.edit.hidden')
            ) : (
              <FileSize bytes={repository.totalSize} />
            )}
          </dd>
        </div>
        <div>
          <dt>{t('repositories.edit.components')}</dt>
          <dd>
            <Count value={repository.componentCount} />
          </dd>
        </div>
        <div>
          <dt>{t('repositories.edit.assets')}</dt>
          <dd>
            <Count value={repository.assetCount} />
          </dd>
        </div>
        <div>
          <dt>{t('repositories.edit.lastUploaded')}</dt>
          <dd>
            {repository.lastUploaded ? (
              <RelativeTime value={repository.lastUploaded} />
            ) : (
              t('repositories.edit.never')
            )}
          </dd>
        </div>
      </dl>
    </Card>
  );
}

export default function RepositoryPage() {
  const t = useT();
  const navigate = useNavigate();
  const toast = useToast();
  const confirm = useConfirm();
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
  const [deleting, setDeleting] = useState(false);
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
  const flags = repository.permissions;
  const canEdit = flags.adminEdit;
  const refresh = async () => {
    await Promise.all([invalidate(UI_REPOSITORIES), docPath ? invalidate(docPath) : Promise.resolve()]);
  };

  const toggleOnline = async () => {
    const current = document.data;
    if (!current || !docPath) {
      return;
    }
    const online = current.online !== false;
    const agreed = await confirm({
      title: online
        ? t('repositories.edit.offlineTitle', { name: repository.name })
        : t('repositories.edit.onlineTitle', { name: repository.name }),
      description: online ? t('repositories.edit.offlineText') : t('repositories.edit.onlineText'),
      confirmLabel: online ? t('repositories.edit.takeOffline') : t('repositories.edit.bringOnline'),
      danger: online,
    });
    if (!agreed) {
      return;
    }
    try {
      await api.put(docPath, { ...current, online: !online });
      toast.success(
        online
          ? t('repositories.edit.nowOffline', { name: repository.name })
          : t('repositories.edit.nowOnline', { name: repository.name }),
      );
      await refresh();
    } catch (error) {
      toast.error(describeError(t, error));
    }
  };

  const maintain = async (action: 'invalidate-cache' | 'rebuild-index') => {
    const invalidating = action === 'invalidate-cache';
    const agreed = await confirm({
      title: invalidating
        ? t('repositories.edit.invalidateTitle', { name: repository.name })
        : t('repositories.edit.rebuildTitle', { name: repository.name }),
      description: invalidating ? t('repositories.edit.invalidateText') : t('repositories.edit.rebuildText'),
      confirmLabel: invalidating ? t('repositories.edit.invalidate') : t('repositories.edit.rebuild'),
    });
    if (!agreed) {
      return;
    }
    try {
      await api.post(path`/service/rest/v1/repositories/${repository.name}/${action}`);
      toast.success(
        invalidating
          ? t('repositories.edit.invalidated', { name: repository.name })
          : t('repositories.edit.rebuilt', { name: repository.name }),
      );
    } catch (error) {
      toast.error(describeError(t, error));
    }
  };

  const canInvalidate = canEdit && (spec.type === 'proxy' || spec.type === 'group');
  const canRebuild = canEdit && (spec.type === 'hosted' || spec.type === 'proxy');
  const canDelete = flags.adminDelete;
  const online = document.data ? document.data.online !== false : repository.online;
  const onlineButton = (
    <Button
      variant="soft"
      color={online ? 'gray' : 'green'}
      disabled={dirty || !document.data}
      onClick={() => void toggleOnline()}
    >
      <Power size={16} aria-hidden="true" />
      {online ? t('repositories.edit.takeOffline') : t('repositories.edit.bringOnline')}
    </Button>
  );

  const actions = (
    <Flex gap="2" wrap="wrap" align="center">
      {(flags.browse || flags.read) && (
        <Button asChild variant="soft">
          <Link to={browseAddress(repository.name)}>
            <FolderOpen size={16} aria-hidden="true" />
            {t('repositories.edit.browse')}
          </Link>
        </Button>
      )}
      {canEdit &&
        (dirty ? <Tooltip content={t('repositories.edit.saveFirst')}>{onlineButton}</Tooltip> : onlineButton)}
      {(canInvalidate || canRebuild || canDelete) && (
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            <IconButton variant="soft" color="gray" aria-label={t('repositories.edit.more')}>
              <MoreHorizontal size={16} aria-hidden="true" />
            </IconButton>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end">
            {canInvalidate && (
              <DropdownMenu.Item onSelect={() => void maintain('invalidate-cache')}>
                <DatabaseZap size={14} aria-hidden="true" />
                {t('repositories.edit.invalidate')}
              </DropdownMenu.Item>
            )}
            {canRebuild && (
              <DropdownMenu.Item onSelect={() => void maintain('rebuild-index')}>
                <RefreshCw size={14} aria-hidden="true" />
                {t('repositories.edit.rebuild')}
              </DropdownMenu.Item>
            )}
            {canDelete && (canInvalidate || canRebuild) && <DropdownMenu.Separator />}
            {canDelete && (
              <DropdownMenu.Item color="red" onSelect={() => setDeleting(true)}>
                <Trash2 size={14} aria-hidden="true" />
                {t('repositories.edit.delete')}
              </DropdownMenu.Item>
            )}
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      )}
    </Flex>
  );

  const members = document.data ? documentMembers(document.data) : (repository.members ?? []);

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
      readOnly={!canEdit}
      repositories={list.data ?? []}
      blobStores={null}
      submitLabel={t('repositories.form.save')}
      onDirtyChange={setDirty}
      onSubmit={async (body) => {
        if (docPath) {
          await api.put(docPath, body);
        }
      }}
      onSaved={() => {
        toast.success(t('repositories.form.saved', { name: repository.name }));
        void refresh();
      }}
    />
  );

  return (
    <Page
      title={repository.name}
      breadcrumbs={crumbs}
      description={
        <span className={styles.badges}>
          <FormatBadge format={repository.format} />
          <TypeBadge type={repository.type} />
          <Status online={online} />
        </span>
      }
      actions={actions}
    >
      <Summary repository={repository} />
      <ProfileTabs
        tabs={[
          {
            value: 'settings',
            label: t('repositories.edit.tabs.settings'),
            content: settings,
            keepMounted: true,
          },
          {
            value: 'membership',
            label: t('repositories.edit.tabs.membership'),
            content: (
              <MembershipTab repository={repository} members={members} repositories={list.data ?? []} />
            ),
          },
          ...(can('astrum:audit:read')
            ? [
                {
                  value: 'audit',
                  label: t('repositories.edit.tabs.audit'),
                  content: <AuditTab name={repository.name} />,
                },
              ]
            : []),
        ]}
      />
      {canDelete && (
        <DeleteRepositoryDialog
          open={deleting}
          onOpenChange={setDeleting}
          name={repository.name}
          type={repository.type}
          groups={repository.memberOf}
          onDelete={async () => {
            await api.delete(repositoryPath(repository.name));
            toast.success(t('repositories.delete.deleted', { name: repository.name }));
            navigate(LIST_ADDRESS, { replace: true });
            void invalidate(UI_REPOSITORIES);
          }}
        />
      )}
    </Page>
  );
}
