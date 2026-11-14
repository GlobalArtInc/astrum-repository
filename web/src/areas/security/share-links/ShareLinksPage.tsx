import { SegmentedControl, Text } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { useState } from 'react';
import { api, path, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useCan, useSession } from '../../../auth/state';
import { formatNumber, useI18n, useT } from '../../../i18n';
import { ConfirmDialog, DateTime, ErrorState, Page, StatusBadge, useToast } from '../../../ui';
import { type ListColumn, ListPager, ListTable, usePaging } from '../shared/AdminList';
import { SHARE_DELETE, SHARE_LINKS, SHARE_READ, type ShareLink } from './model';

const PAGE_SIZE = 20;
const NONE: ShareLink[] = [];

export default function ShareLinksPage() {
  const t = useT();
  const { language } = useI18n();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const { user } = useSession();
  const canReadAll = useCan(SHARE_READ);
  const canDeleteAll = useCan(SHARE_DELETE);
  const [everyone, setEveryone] = useState(false);
  const showAll = everyone && canReadAll;
  const links = useApi<ShareLink[]>(showAll ? withQuery(SHARE_LINKS, { all: true }) : SHARE_LINKS);
  const [revoking, setRevoking] = useState<ShareLink | null>(null);
  const paging = usePaging(links.data ?? NONE, PAGE_SIZE);

  const revoke = async () => {
    if (!revoking) {
      return;
    }
    await api.delete(`${SHARE_LINKS}${path`/${revoking.id}`}`);
    toast.success(t('security.shareLinks.revoked', { path: revoking.path }));
    await invalidate(SHARE_LINKS);
  };

  const columns: ListColumn<ShareLink>[] = [
    { id: 'repository', header: t('security.shareLinks.repository'), cell: (row) => row.repository },
    {
      id: 'path',
      header: t('security.shareLinks.path'),
      cell: (row) => (
        <>
          <code>{row.path}</code>
          {row.note && (
            <Text as="p" size="2" color="gray">
              {row.note}
            </Text>
          )}
        </>
      ),
    },
    { id: 'creator', header: t('security.shareLinks.creator'), cell: (row) => row.createdBy },
    {
      id: 'expires',
      header: t('security.shareLinks.expires'),
      cell: (row) => <DateTime value={row.expiresAt} />,
    },
    {
      id: 'downloads',
      header: t('security.shareLinks.downloads'),
      cell: (row) => (
        <>
          {formatNumber(language, row.downloads)}
          {row.maxDownloads !== null && (
            <Text as="p" size="2" color="gray">
              {t('security.shareLinks.limit', { limit: row.maxDownloads })}
            </Text>
          )}
        </>
      ),
    },
    {
      id: 'status',
      header: t('security.shareLinks.status'),
      cell: (row) =>
        row.expired ? (
          <StatusBadge tone="neutral">{t('security.shareLinks.expired')}</StatusBadge>
        ) : (
          <StatusBadge tone="success">{t('security.shareLinks.active')}</StatusBadge>
        ),
    },
  ];

  const canRevoke = (row: ShareLink) => row.createdBy === user?.userId || canDeleteAll;

  return (
    <Page
      gutter="small"
      title={t('security.shareLinks.title')}
      description={t('security.shareLinks.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.shareLinks.title') },
      ]}
    >
      {canReadAll && (
        <SegmentedControl.Root
          value={showAll ? 'all' : 'mine'}
          onValueChange={(value) => {
            setEveryone(value === 'all');
            paging.setPage(0);
          }}
        >
          <SegmentedControl.Item value="mine">{t('security.shareLinks.mine')}</SegmentedControl.Item>
          <SegmentedControl.Item value="all">{t('security.shareLinks.everyone')}</SegmentedControl.Item>
        </SegmentedControl.Root>
      )}
      {links.error ? (
        <ErrorState error={links.error} onRetry={() => void links.refetch()} />
      ) : (
        <>
          <ListTable
            label={t('security.shareLinks.title')}
            columns={columns}
            rows={paging.visible}
            getRowId={(row) => row.id}
            loading={links.isPending}
            empty={
              <Text as="p" size="2" color="gray">
                {t('security.shareLinks.empty')}
              </Text>
            }
            buttons={(row) =>
              canRevoke(row)
                ? [
                    {
                      label: t('security.shareLinks.revokeOne', { path: row.path }),
                      icon: Trash2,
                      danger: true,
                      onClick: () => setRevoking(row),
                    },
                  ]
                : []
            }
          />
          {paging.total > PAGE_SIZE && (
            <ListPager
              range={t('security.shareLinks.range', {
                from: paging.from,
                to: paging.to,
                total: paging.total,
              })}
              page={paging.page}
              pages={paging.pages}
              onPage={paging.setPage}
            />
          )}
        </>
      )}
      <ConfirmDialog
        open={revoking !== null}
        onOpenChange={(open) => !open && setRevoking(null)}
        onConfirm={revoke}
        title={t('security.shareLinks.revokeTitle')}
        description={t('security.shareLinks.revokeText', { path: revoking?.path ?? '' })}
        confirmLabel={t('security.shareLinks.revoke')}
        danger
      />
    </Page>
  );
}
