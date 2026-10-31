import { Badge, Card, Table, Text } from '@radix-ui/themes';
import { Bot, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { api, useApi, useInvalidateApi } from '../../api';
import { useT } from '../../i18n';
import { ConfirmDialog, DateTime, useToast } from '../../ui';
import { ListPager, SearchBox, usePaging } from '../security/shared/AdminList';
import { type AgentGrant, filterGrants, GRANTS, grantPath } from './agents';
import { sortNewestFirst } from './model';
import styles from './Tokens.module.css';

const GRANTS_PAGE_SIZE = 10;

export function AgentGrants() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const grants = useApi<AgentGrant[]>(GRANTS);
  const [revoking, setRevoking] = useState<AgentGrant | null>(null);
  const [query, setQuery] = useState('');
  const rows = useMemo(() => filterGrants(sortNewestFirst(grants.data ?? []), query), [grants.data, query]);
  const paging = usePaging(rows, GRANTS_PAGE_SIZE);

  const revoke = async () => {
    if (!revoking) {
      return;
    }
    await api.delete(grantPath(revoking.id));
    toast.success(t('user.agents.revoked', { name: revoking.clientName }));
    await invalidate(GRANTS);
  };

  return (
    <section aria-labelledby="agents-title" className={styles.agents}>
      <div className={styles.titleRow}>
        <Bot size={20} aria-hidden="true" className={styles.icon} />
        <h2 id="agents-title" className={styles.cardTitle}>
          {t('user.agents.title')}
        </h2>
      </div>
      <p className={styles.intro}>{t('user.agents.intro')}</p>
      {grants.error ? (
        <Text as="p" size="2" color="red" role="alert">
          {t('user.agents.loadFailed')}
        </Text>
      ) : grants.data && grants.data.length > 0 ? (
        <>
          <SearchBox
            value={query}
            onChange={(value) => {
              setQuery(value);
              paging.setPage(0);
            }}
            placeholder={t('user.agents.search')}
          />
          {rows.length > 0 ? (
            <>
              <Table.Root variant="surface" className={styles.table} aria-label={t('user.agents.title')}>
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeaderCell>{t('user.agents.client')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{t('user.tokens.scope')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{t('user.agents.connected')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{t('user.tokens.lastUsed')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>
                      <span className={styles.hidden}>{t('user.tokens.actions')}</span>
                    </Table.ColumnHeaderCell>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {paging.visible.map((grant) => (
                    <Table.Row key={grant.id}>
                      <Table.RowHeaderCell>
                        <span className={styles.name}>{grant.clientName}</span>
                      </Table.RowHeaderCell>
                      <Table.Cell>
                        <Badge size="1">
                          {grant.scope === 'WRITE' ? t('user.tokens.scopeWrite') : t('user.tokens.scopeRead')}
                        </Badge>
                      </Table.Cell>
                      <Table.Cell>
                        <DateTime value={grant.createdAt} dateOnly />
                      </Table.Cell>
                      <Table.Cell>
                        {grant.lastUsedAt ? (
                          <DateTime value={grant.lastUsedAt} />
                        ) : (
                          <span className={styles.muted}>{t('user.tokens.neverUsed')}</span>
                        )}
                      </Table.Cell>
                      <Table.Cell className={styles.actions}>
                        <button
                          type="button"
                          className={styles.revoke}
                          aria-label={t('user.agents.revokeOne', { name: grant.clientName })}
                          title={t('user.agents.revoke')}
                          onClick={() => setRevoking(grant)}
                        >
                          <Trash2 size={16} aria-hidden="true" />
                        </button>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </Table.Body>
              </Table.Root>
              <ListPager
                range={t('user.agents.range', { from: paging.from, to: paging.to, total: paging.total })}
                page={paging.page}
                pages={paging.pages}
                onPage={paging.setPage}
              />
            </>
          ) : (
            <Card size="1" className={styles.empty}>
              {t('user.agents.noMatches')}
            </Card>
          )}
        </>
      ) : (
        <Card size="1" className={styles.empty}>
          {grants.isPending ? '' : t('user.agents.empty')}
        </Card>
      )}
      <ConfirmDialog
        open={revoking !== null}
        onOpenChange={(open) => !open && setRevoking(null)}
        onConfirm={revoke}
        title={t('user.agents.revokeTitle')}
        description={t('user.agents.revokeText', { name: revoking?.clientName ?? '' })}
        confirmLabel={t('user.agents.revoke')}
        danger
      />
    </section>
  );
}
