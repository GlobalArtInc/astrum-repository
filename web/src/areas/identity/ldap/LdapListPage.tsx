import { Button, IconButton, Spinner, TextField, Tooltip } from '@radix-ui/themes';
import {
  ArrowUpDown,
  CircleHelp,
  ExternalLink,
  GripVertical,
  Info,
  Pencil,
  Plus,
  RefreshCw,
  Search,
  Trash2,
} from 'lucide-react';
import { type DragEvent, useMemo, useState } from 'react';
import { Link } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ConfirmDialog, ErrorState, Page, useToast } from '../../../ui';
import { LDAP, LDAP_CACHE, LDAP_DOCS, LDAP_HELP, LDAP_ORDER, type LdapServer } from '../api';
import layout from '../shared/Layout.module.css';
import { ChangeOrderDialog } from './ChangeOrderDialog';
import styles from './LdapList.module.css';
import { LDAP_PAGE, ldapHref } from './links';
import { matches, move, serverUrl } from './model';

const DRAG_TYPE = 'application/x-astrum-ldap';

export function AboutLdap() {
  const t = useT();
  return (
    <aside className={layout.card} aria-labelledby="about-ldap">
      <h2 id="about-ldap" className={layout.cardTitle}>
        <CircleHelp size={14} aria-hidden="true" color="var(--gray-11)" />
        {t('identity.ldap.about')}
      </h2>
      <p className={layout.helpText}>{t('identity.ldap.aboutText')}</p>
      <a className={layout.helpLink} href={LDAP_HELP} target="_blank" rel="noreferrer">
        {t('identity.ldap.viewDocs')}
        <ExternalLink size={12} aria-hidden="true" />
      </a>
    </aside>
  );
}

function AboutServers() {
  const t = useT();
  return (
    <section className={layout.card} aria-labelledby="about-ldap-servers">
      <h2 id="about-ldap-servers" className={layout.cardTitle}>
        <Info size={16} aria-hidden="true" color="var(--accent-11)" />
        {t('identity.ldap.aboutServers')}
      </h2>
      <p className={layout.cardText}>{t('identity.ldap.aboutServersText')}</p>
      <p className={layout.cardText}>
        {t.rich('identity.ldap.seeDocs', {
          link: (
            <a key="link" className={layout.link} href={LDAP_DOCS} target="_blank" rel="noreferrer">
              {t('identity.ldap.docsLink')}
              <ExternalLink size={12} aria-hidden="true" />
            </a>
          ),
        })}
      </p>
    </section>
  );
}

function ServerTable({ servers, filter }: { servers: LdapServer[]; filter: string }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canUpdate = useCan('astrum:ldap:update');
  const canDelete = useCan('astrum:ldap:delete');
  const [dragging, setDragging] = useState<number | null>(null);
  const [over, setOver] = useState<number | null>(null);
  const [deleting, setDeleting] = useState<LdapServer | null>(null);
  const shown = servers.filter((server) => matches(server, filter));
  const reorderable = canUpdate && filter.trim() === '' && servers.length > 1;

  const drop = async (event: DragEvent, target: number) => {
    event.preventDefault();
    const from = dragging;
    setDragging(null);
    setOver(null);
    if (from === null || from === target) {
      return;
    }
    const next = move(servers, from, target);
    try {
      await api.post(
        LDAP_ORDER,
        next.map((server) => server.name),
      );
      toast.success(t('identity.ldap.orderSaved'));
    } catch (failure) {
      toast.error(failure instanceof Error ? failure.message : String(failure));
    }
    await invalidate(LDAP);
  };

  const remove = async () => {
    if (!deleting) {
      return;
    }
    await api.delete(`${LDAP}${path`/${deleting.name}`}`);
    toast.success(t('identity.ldap.deleted', { name: deleting.name }));
    await invalidate(LDAP);
  };

  if (servers.length === 0) {
    return (
      <div className={styles.empty}>
        <span>{t('identity.ldap.empty')}</span>
        <span>{t('identity.ldap.emptyHint')}</span>
      </div>
    );
  }
  if (shown.length === 0) {
    return <div className={styles.empty}>{t('identity.ldap.noMatch')}</div>;
  }
  return (
    <div className={styles.frame}>
      <table className={styles.table}>
        <thead>
          <tr>
            <th className={styles.orderColumn}>{t('identity.ldap.order')}</th>
            <th>{t('identity.ldap.name')}</th>
            <th className={styles.urlColumn}>{t('identity.ldap.url')}</th>
            <th className={styles.actionsColumn}>
              <span className="visually-hidden">{t('identity.ldap.actions')}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {shown.map((server) => {
            const index = servers.indexOf(server);
            return (
              <tr
                key={server.name}
                draggable={reorderable}
                data-over={over === index || undefined}
                data-dragging={dragging === index || undefined}
                onDragStart={(event) => {
                  event.dataTransfer.setData(DRAG_TYPE, server.name);
                  event.dataTransfer.effectAllowed = 'move';
                  setDragging(index);
                }}
                onDragEnd={() => {
                  setDragging(null);
                  setOver(null);
                }}
                onDragOver={(event) => {
                  if (dragging !== null) {
                    event.preventDefault();
                    setOver(index);
                  }
                }}
                onDrop={(event) => drop(event, index)}
              >
                <td>
                  <span className={styles.orderCell}>
                    <span className={styles.grip} aria-hidden="true" aria-disabled={!reorderable}>
                      <GripVertical size={14} />
                    </span>
                    {server.order}
                  </span>
                </td>
                <td className={styles.name}>
                  <Link className={styles.nameLink} to={ldapHref(server.name)}>
                    {server.name}
                  </Link>
                </td>
                <td className={`${styles.url} ${styles.urlColumn}`}>{serverUrl(server)}</td>
                <td>
                  <span className={styles.rowActions}>
                    <Tooltip content={t('identity.ldap.edit', { name: server.name })}>
                      <IconButton asChild size="1" variant="ghost" color="gray">
                        <Link
                          to={ldapHref(server.name)}
                          aria-label={t('identity.ldap.edit', { name: server.name })}
                        >
                          <Pencil size={16} aria-hidden="true" />
                        </Link>
                      </IconButton>
                    </Tooltip>
                    {canDelete && (
                      <Tooltip content={t('identity.ldap.deleteNamed', { name: server.name })}>
                        <IconButton
                          size="1"
                          variant="ghost"
                          color="gray"
                          aria-label={t('identity.ldap.deleteNamed', { name: server.name })}
                          onClick={() => setDeleting(server)}
                        >
                          <Trash2 size={16} aria-hidden="true" />
                        </IconButton>
                      </Tooltip>
                    )}
                  </span>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <ConfirmDialog
        open={deleting !== null}
        onOpenChange={(open) => {
          if (!open) {
            setDeleting(null);
          }
        }}
        onConfirm={remove}
        title={t('identity.ldap.deleteTitle')}
        description={t('identity.ldap.deleteText', { name: deleting?.name ?? '' })}
        confirmLabel={t('identity.ldap.deleteServer')}
        cancelLabel={t('identity.ldap.cancel')}
        danger
      />
    </div>
  );
}

export default function LdapListPage() {
  const t = useT();
  const toast = useToast();
  const servers = useApi<LdapServer[]>(LDAP);
  const canCreate = useCan('astrum:ldap:create');
  const canUpdate = useCan('astrum:ldap:update');
  const canDelete = useCan('astrum:ldap:delete');
  const [filter, setFilter] = useState('');
  const [ordering, setOrdering] = useState(false);
  const [clearing, setClearing] = useState(false);
  const list = useMemo(() => [...(servers.data ?? [])].sort((a, b) => a.order - b.order), [servers.data]);

  const clear = async () => {
    setClearing(true);
    try {
      await api.delete(LDAP_CACHE);
      toast.success(t('identity.ldap.cacheCleared'));
    } catch (failure) {
      toast.error(failure instanceof Error ? failure.message : String(failure));
    } finally {
      setClearing(false);
    }
  };

  return (
    <Page
      title={t('identity.ldap.title')}
      description={t('identity.ldap.description')}
      breadcrumbs={[
        { label: t('identity.crumbs.settings'), to: '/settings' },
        { label: t('identity.ldap.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild size="3" className={layout.bigButton}>
            <Link to={`${LDAP_PAGE}/create`}>
              <Plus size={16} aria-hidden="true" />
              {t('identity.ldap.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={layout.layout}>
        <div className={layout.main}>
          <div className={styles.toolbar}>
            <TextField.Root
              className={styles.filter}
              size="2"
              value={filter}
              placeholder={t('identity.ldap.filter')}
              aria-label={t('identity.ldap.filterLabel')}
              onChange={(event) => setFilter(event.target.value)}
            >
              <TextField.Slot>
                <Search size={14} aria-hidden="true" />
              </TextField.Slot>
            </TextField.Root>
            <div className={styles.buttons}>
              {canUpdate && (
                <Button
                  size="3"
                  variant="outline"
                  color="gray"
                  className={layout.neutralButton}
                  disabled={list.length < 2}
                  onClick={() => setOrdering(true)}
                >
                  <ArrowUpDown size={16} aria-hidden="true" />
                  {t('identity.ldap.changeOrder')}
                </Button>
              )}
              {canDelete && (
                <Button
                  size="3"
                  variant="outline"
                  color="gray"
                  className={layout.neutralButton}
                  loading={clearing}
                  onClick={clear}
                >
                  <RefreshCw size={16} aria-hidden="true" />
                  {t('identity.ldap.clearCache')}
                </Button>
              )}
            </div>
          </div>
          {servers.isPending ? (
            <div className={styles.empty} role="status">
              <Spinner size="3" />
              <span>{t('identity.ldap.loading')}</span>
            </div>
          ) : servers.error ? (
            <ErrorState error={servers.error} onRetry={() => servers.refetch()} />
          ) : (
            <ServerTable servers={list} filter={filter} />
          )}
          <AboutServers />
        </div>
        <AboutLdap />
      </div>
      {ordering && <ChangeOrderDialog servers={list} onClose={() => setOrdering(false)} />}
    </Page>
  );
}
