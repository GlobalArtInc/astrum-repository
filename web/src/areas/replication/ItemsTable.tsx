import { Button, Checkbox, Flex, Text } from '@radix-ui/themes';
import { useState } from 'react';
import { api, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import { type Column, ConfirmDialog, DataTable, DateTime, EmptyState, SelectField, useToast } from '../../ui';
import styles from '../security/shared/Form.module.css';
import {
  type DiscardAnswer,
  ITEM_STATES,
  type ItemSelection,
  type ItemState,
  itemsActionPath,
  REPLICATIONS,
  type ReplicationItem,
  type RetryAnswer,
  useReplicationItems,
} from './api';
import { excerpt } from './model';

type ItemsActionKind = 'retry' | 'discard';
type ItemsScope = 'selected' | 'all';

interface PendingAction {
  kind: ItemsActionKind;
  scope: ItemsScope;
  count: number;
}

interface ItemsTableProps {
  replicationId: string;
  dead: number;
  canUpdate: boolean;
}

const TITLES: Record<ItemsActionKind, Record<ItemsScope, string>> = {
  retry: {
    selected: 'replication.items.retrySelectedTitle',
    all: 'replication.items.retryAllTitle',
  },
  discard: {
    selected: 'replication.items.discardSelectedTitle',
    all: 'replication.items.discardAllTitle',
  },
};

const STATE_LABELS: Record<ItemState, string> = {
  dead: 'replication.items.stateDead',
  pending: 'replication.items.statePending',
  leased: 'replication.items.stateLeased',
};

export function ItemsTable({ replicationId, dead, canUpdate }: ItemsTableProps) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const [state, setState] = useState<ItemState>('dead');
  const [selected, setSelected] = useState<ReadonlySet<number>>(new Set());
  const [pending, setPending] = useState<PendingAction | null>(null);
  const items = useReplicationItems(replicationId, state);
  const rows = items.data?.pages.flatMap((page) => page.items);
  const actionable = state === 'dead' && dead > 0;

  const choose = (value: string) => {
    const next = ITEM_STATES.find((candidate) => candidate === value);
    if (next !== undefined) {
      setState(next);
      setSelected(new Set());
    }
  };

  const toggle = (item: ReplicationItem) => {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(item.id)) {
        next.delete(item.id);
      } else {
        next.add(item.id);
      }
      return next;
    });
  };

  const run = async () => {
    if (pending === null) {
      return;
    }
    const body: ItemSelection = pending.scope === 'all' ? { all: true } : { ids: [...selected] };
    if (pending.kind === 'retry') {
      const answer = await api.post<RetryAnswer>(itemsActionPath(replicationId, 'retry'), body);
      toast.success(t('replication.items.retried', { count: answer.retried }));
    } else {
      const answer = await api.post<DiscardAnswer>(itemsActionPath(replicationId, 'discard'), body);
      toast.success(t('replication.items.discarded', { count: answer.discarded }));
    }
    setSelected(new Set());
    await invalidate(REPLICATIONS);
  };

  const selectColumn: Column<ReplicationItem> = {
    id: 'select',
    header: '',
    width: '40px',
    cell: (item) => (
      <Checkbox
        checked={selected.has(item.id)}
        onCheckedChange={() => toggle(item)}
        aria-label={t('replication.items.select', { key: item.key })}
      />
    ),
  };
  const columns: Column<ReplicationItem>[] = [
    ...(actionable ? [selectColumn] : []),
    {
      id: 'key',
      header: t('replication.items.key'),
      primary: true,
      cell: (item) => (
        <Text size="2" className={styles.mono}>
          {item.key}
        </Text>
      ),
    },
    {
      id: 'attempts',
      header: t('replication.items.attempts'),
      align: 'right',
      cell: (item) => item.attempts,
    },
    {
      id: 'lastError',
      header: t('replication.items.lastError'),
      cell: (item) => (
        <Text size="2" title={item.lastError ?? undefined}>
          {excerpt(item.lastError)}
        </Text>
      ),
    },
    {
      id: 'firstQueued',
      header: t('replication.items.firstQueued'),
      cell: (item) => <DateTime value={item.firstQueuedAt} />,
    },
  ];

  const stateOptions = ITEM_STATES.map((value) => ({ value, label: t(STATE_LABELS[value]) }));
  const describe =
    pending?.kind === 'discard' ? 'replication.items.discardText' : 'replication.items.retryText';

  return (
    <Flex direction="column" gap="3">
      <Flex justify="between" align="end" gap="3" wrap="wrap">
        <SelectField
          label={t('replication.items.show')}
          value={state}
          onChange={choose}
          options={stateOptions}
        />
        {actionable && (
          <Flex gap="2" wrap="wrap">
            <Button
              variant="soft"
              disabled={!canUpdate || selected.size === 0}
              onClick={() => setPending({ kind: 'retry', scope: 'selected', count: selected.size })}
            >
              {t('replication.items.retrySelectedButton')}
            </Button>
            <Button
              variant="soft"
              disabled={!canUpdate}
              onClick={() => setPending({ kind: 'retry', scope: 'all', count: dead })}
            >
              {t('replication.items.retryAllButton')}
            </Button>
            <Button
              variant="soft"
              color="red"
              disabled={!canUpdate || selected.size === 0}
              onClick={() => setPending({ kind: 'discard', scope: 'selected', count: selected.size })}
            >
              {t('replication.items.discardSelectedButton')}
            </Button>
            <Button
              variant="soft"
              color="red"
              disabled={!canUpdate}
              onClick={() => setPending({ kind: 'discard', scope: 'all', count: dead })}
            >
              {t('replication.items.discardAllButton')}
            </Button>
          </Flex>
        )}
      </Flex>
      <DataTable
        label={t('replication.items.table')}
        columns={columns}
        rows={rows}
        getRowId={(item) => String(item.id)}
        loading={items.isPending}
        error={items.error}
        onRetry={() => void items.refetch()}
        empty={<EmptyState title={t('replication.items.empty')} />}
        hasMore={items.hasNextPage}
        onLoadMore={() => void items.fetchNextPage()}
        loadingMore={items.isFetchingNextPage}
      />
      <ConfirmDialog
        open={pending !== null}
        onOpenChange={(open) => {
          if (!open) {
            setPending(null);
          }
        }}
        onConfirm={run}
        title={pending === null ? '' : t(TITLES[pending.kind][pending.scope], { count: pending.count })}
        description={t(describe)}
        confirmLabel={t(
          pending?.kind === 'discard' ? 'replication.items.discardConfirm' : 'replication.items.retryConfirm',
        )}
        danger={pending?.kind === 'discard'}
      />
    </Flex>
  );
}
