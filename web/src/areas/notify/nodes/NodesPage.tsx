import { Badge, Table } from '@radix-ui/themes';
import { ExternalLink, Info } from 'lucide-react';
import { useApi } from '../../../api';
import { useT } from '../../../i18n';
import { ErrorState, Page } from '../../../ui';
import styles from '../Notify.module.css';

export const NODES = '/service/rest/internal/ui/nodes';
export const READ = 'astrum:settings:read';

export interface Node {
  id: string;
  name: string;
  version: string;
  startedAt: string;
  lastSeen: string;
  current: boolean;
}

export default function NodesPage() {
  const t = useT();
  const nodes = useApi<{ nodes: Node[] }>(NODES);
  const list = nodes.data?.nodes ?? [];
  return (
    <Page
      title={t('notify.nodes.title')}
      gutter="small"
      description={t('notify.nodes.description')}
      breadcrumbs={[
        { label: t('notify.crumbs.settings'), to: '/settings' },
        { label: t('notify.nodes.title') },
      ]}
    >
      {nodes.error ? (
        <ErrorState error={nodes.error} onRetry={() => void nodes.refetch()} />
      ) : (
        <>
          <div className={styles.alert} role="note">
            <Info size={18} aria-hidden="true" className={styles.alertIcon} />
            <div>
              {list.length > 1
                ? t('notify.nodes.replicas', { count: list.length })
                : t.rich('notify.nodes.single', {
                    link: (
                      <a
                        key="link"
                        href="https://links.globalart.dev/astrum/high-availability"
                        target="_blank"
                        rel="noreferrer"
                        className={styles.inlineLink}
                      >
                        {t('notify.nodes.docs')}
                        <ExternalLink size={12} aria-hidden="true" />
                      </a>
                    ),
                  })}
            </div>
          </div>
          <h2 className={styles.sectionTitle}>{t('notify.nodes.title')}</h2>
          <p className={styles.sectionText}>{t('notify.nodes.participating')}</p>
          <div className={styles.scroll}>
            <Table.Root variant="surface" className={styles.nodes} aria-label={t('notify.nodes.title')}>
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell>{t('notify.nodes.name')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('notify.nodes.identity')}</Table.ColumnHeaderCell>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {nodes.isPending && (
                  <Table.Row>
                    <Table.Cell colSpan={2} className={styles.muted}>
                      {t('notify.nodes.loading')}
                    </Table.Cell>
                  </Table.Row>
                )}
                {!nodes.isPending && list.length === 0 && (
                  <Table.Row>
                    <Table.Cell colSpan={2} className={styles.muted}>
                      {t('notify.nodes.none')}
                    </Table.Cell>
                  </Table.Row>
                )}
                {list.map((node) => (
                  <Table.Row key={node.id} align="center">
                    <Table.RowHeaderCell>
                      <span className={styles.nodeName}>
                        {node.name}
                        {node.current && (
                          <Badge color="gray" title={t('notify.nodes.currentHint')}>
                            {t('notify.nodes.current')}
                          </Badge>
                        )}
                      </span>
                    </Table.RowHeaderCell>
                    <Table.Cell>{node.id}</Table.Cell>
                  </Table.Row>
                ))}
              </Table.Body>
            </Table.Root>
          </div>
        </>
      )}
    </Page>
  );
}
