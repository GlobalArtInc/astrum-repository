import { Button, Callout, Text, TextField } from '@radix-ui/themes';
import { Filter, Info, Plus, Search, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useSearchParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { type Column, ConfirmDialog, DataTable, EmptyState, Page, useToast } from '../../ui';
import { type ContentSelector, PRIVILEGES, SELECTORS, selectorHref } from './model';
import styles from './Selectors.module.css';

export function filterSelectors(selectors: readonly ContentSelector[], query: string): ContentSelector[] {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  return selectors.filter((selector) => {
    const text = `${selector.name} ${selector.description ?? ''} ${selector.expression}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

export default function SelectorsPage() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canCreate = useCan('astrum:selectors:create');
  const canDelete = useCan('astrum:selectors:delete');
  const [params, setParams] = useSearchParams();
  const query = params.get('q') ?? '';
  const [deleting, setDeleting] = useState<ContentSelector | null>(null);
  const selectors = useApi<ContentSelector[]>(SELECTORS);
  const rows = useMemo(
    () => (selectors.data ? filterSelectors(selectors.data, query) : undefined),
    [selectors.data, query],
  );

  const columns: Column<ContentSelector>[] = [
    {
      id: 'name',
      header: t('selectors.list.name'),
      cell: (row) => row.name,
      sortValue: (row) => row.name,
      primary: true,
    },
    {
      id: 'description',
      header: t('selectors.list.description'),
      cell: (row) => (
        <Text size="2" color="gray">
          {row.description}
        </Text>
      ),
      sortValue: (row) => row.description ?? '',
      hideOnPhone: true,
    },
    {
      id: 'expression',
      header: t('selectors.list.expression'),
      cell: (row) => <code className={styles.mono}>{row.expression}</code>,
    },
  ];

  const remove = async () => {
    if (!deleting) {
      return;
    }
    await api.delete(`${SELECTORS}${path`/${deleting.name}`}`);
    toast.success(t('selectors.list.deleted', { name: deleting.name }));
    await Promise.all([invalidate(SELECTORS), invalidate(PRIVILEGES)]);
  };

  return (
    <Page
      title={t('selectors.list.title')}
      description={t('selectors.list.intro')}
      breadcrumbs={[
        { label: t('selectors.crumbs.settings'), to: '/settings' },
        { label: t('selectors.list.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild>
            <Link to="/admin/repository/selectors/create">
              <Plus size={16} aria-hidden="true" />
              {t('selectors.list.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={styles.toolbar}>
        <TextField.Root
          type="search"
          value={query}
          placeholder={t('selectors.list.search')}
          aria-label={t('selectors.list.search')}
          style={{ flex: '1 1 240px', minWidth: 0 }}
          onChange={(event) =>
            setParams(
              (previous) => {
                const next = new URLSearchParams(previous);
                if (event.target.value) {
                  next.set('q', event.target.value);
                } else {
                  next.delete('q');
                }
                return next;
              },
              { replace: true },
            )
          }
        >
          <TextField.Slot>
            <Search size={16} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
      </div>
      <DataTable
        label={t('selectors.list.title')}
        columns={columns}
        rows={rows}
        getRowId={(row) => row.name}
        loading={selectors.isPending}
        error={selectors.error}
        onRetry={() => void selectors.refetch()}
        rowHref={(row) => selectorHref(row.name)}
        initialSort={{ column: 'name', direction: 'asc' }}
        pageSize={50}
        empty={
          <EmptyState
            icon={Filter}
            title={query ? t('selectors.list.noMatches') : t('selectors.list.empty')}
            description={query ? undefined : t('selectors.list.emptyText')}
          />
        }
        rowActions={
          canDelete
            ? () => [
                {
                  id: 'delete',
                  label: t('selectors.form.delete'),
                  icon: Trash2,
                  danger: true,
                  onSelect: setDeleting,
                },
              ]
            : undefined
        }
      />
      <Callout.Root color="gray" variant="surface">
        <Callout.Icon>
          <Info size={16} />
        </Callout.Icon>
        <Callout.Text>{t('selectors.list.about')}</Callout.Text>
      </Callout.Root>
      <ConfirmDialog
        open={deleting !== null}
        onOpenChange={(open) => !open && setDeleting(null)}
        onConfirm={remove}
        title={t('selectors.list.deleteTitle', { name: deleting?.name ?? '' })}
        description={t('selectors.list.deleteText')}
        confirmLabel={t('selectors.form.delete')}
        danger
      />
    </Page>
  );
}
