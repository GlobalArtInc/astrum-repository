import { Button, Flex, IconButton, Link as RadixLink, Text, TextField } from '@radix-ui/themes';
import { useQueries } from '@tanstack/react-query';
import { ArrowLeft, FolderX, MousePointerClick, Plug, Search, Settings2, X } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { Link, useLocation, useNavigate, useParams } from 'react-router';
import { api, apiKey, useApi } from '../../api';
import { useSession } from '../../auth/state';
import { formatNumber, useI18n, useT } from '../../i18n';
import {
  DateTime,
  EmptyState,
  ErrorState,
  FileSize,
  FormatBadge,
  formatName,
  Page,
  PHONE,
  readStored,
  TypeBadge,
  useMediaQuery,
  writeStored,
} from '../../ui';
import { AssetPanel, ComponentPanel, FolderPanel, treePath } from './details/panels';
import { settingsPath } from './list/RepositoryListPage';
import { type DetailItem, DetailList, OnlineStatus, Section, UrlLine } from './parts';
import styles from './RepositoryPage.module.css';
import { RepositorySearch } from './RepositorySearch';
import {
  ancestorsOf,
  browseUrl,
  findNode,
  flatten,
  type Loaded,
  parentOf,
  ROOT,
  selectionFromPathname,
} from './tree/model';
import { RepositoryTree } from './tree/RepositoryTree';
import { clamp, Splitter } from './tree/Splitter';
import { REPOSITORIES_PATH, type RepositoryItem, type TreeListing, type TreeNode } from './types';
import { supportsUsage, UsageDialog, UsageSnippets } from './usage';

export const WIDTH_KEY = 'astrum.browse.treeWidth';
const DEFAULT_WIDTH = 340;
const MIN_WIDTH = 220;

function useDebounced<T>(value: T, delay: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => setDebounced(value), delay);
    return () => window.clearTimeout(timer);
  }, [value, delay]);
  return debounced;
}

function useTreeWidth(): [number, (value: number) => void] {
  const [width, setWidth] = useState(() => {
    const stored = Number(readStored(WIDTH_KEY));
    return Number.isFinite(stored) && stored >= MIN_WIDTH ? stored : DEFAULT_WIDTH;
  });
  return [
    width,
    (value) => {
      setWidth(value);
      writeStored(WIDTH_KEY, String(value));
    },
  ];
}

function RepositoryOverview({ repository }: { repository: RepositoryItem }) {
  const t = useT();
  const { language } = useI18n();
  const { state } = useSession();
  const group = repository.type === 'group';
  const count = (value: number | null) => (value === null || group ? '—' : formatNumber(language, value));
  const items: DetailItem[] = [
    { id: 'format', label: t('browse.details.format'), value: formatName(repository.format) },
    { id: 'components', label: t('browse.repository.components'), value: count(repository.componentCount) },
    { id: 'assets', label: t('browse.repository.assets'), value: count(repository.assetCount) },
    {
      id: 'size',
      label: t('browse.details.size'),
      value: repository.totalSize === null || group ? '—' : <FileSize bytes={repository.totalSize} />,
    },
    {
      id: 'uploaded',
      label: t('browse.repository.lastUploaded'),
      value: repository.lastUploaded ? <DateTime value={repository.lastUploaded} /> : '—',
    },
  ];
  if (repository.blobStoreName) {
    items.push({ id: 'store', label: t('browse.details.blobStore'), value: repository.blobStoreName });
  }
  if (repository.remoteUrl) {
    items.push({
      id: 'remote',
      label: t('browse.repository.remote'),
      value: repository.remoteUrl,
      mono: true,
    });
  }
  if (repository.members) {
    items.push({
      id: 'members',
      label: t('browse.repository.members'),
      value:
        repository.members.length === 0 ? (
          '—'
        ) : (
          <span className={styles.links}>
            {repository.members.map((member) => (
              <RadixLink asChild key={member}>
                <Link to={browseUrl(member)}>{member}</Link>
              </RadixLink>
            ))}
          </span>
        ),
    });
  }
  if (repository.memberOf.length > 0) {
    items.push({
      id: 'memberOf',
      label: t('browse.repository.memberOf'),
      value: (
        <span className={styles.links}>
          {repository.memberOf.map((group) => (
            <RadixLink asChild key={group}>
              <Link to={browseUrl(group)}>{group}</Link>
            </RadixLink>
          ))}
        </span>
      ),
    });
  }
  return (
    <div className={styles.overview}>
      <EmptyState
        icon={MousePointerClick}
        title={t('browse.repository.nothingSelected')}
        description={t('browse.repository.nothingSelectedText')}
      />
      <Section title={t('browse.repository.about')}>
        {repository.limited && (
          <Text as="p" size="2" color="gray">
            {t('browse.repository.limited')}
          </Text>
        )}
        <DetailList items={items} />
      </Section>
      {supportsUsage(repository.format) && (
        <Section title={t('browse.details.usage')}>
          <UsageSnippets
            context={{
              format: repository.format,
              repositoryName: repository.name,
              repositoryType: repository.type,
              baseUrl: state.baseUrl,
            }}
          />
        </Section>
      )}
    </div>
  );
}

function Details({
  repository,
  node,
  selection,
  missing,
  onNavigate,
}: {
  repository: RepositoryItem;
  node: TreeNode | undefined;
  selection: string;
  missing: boolean;
  onNavigate(path: string): void;
}) {
  const t = useT();
  if (selection === '') {
    return <RepositoryOverview repository={repository} />;
  }
  if (!node) {
    return missing ? (
      <EmptyState
        icon={FolderX}
        title={t('browse.repository.missingTitle')}
        description={t('browse.repository.missingText', { path: selection, repository: repository.name })}
        action={
          <Button asChild variant="soft">
            <Link to={browseUrl(repository.name)}>{t('browse.repository.toRoot')}</Link>
          </Button>
        }
      />
    ) : (
      <Text as="p" size="2" color="gray" aria-busy="true">
        {t('browse.tree.loading')}
      </Text>
    );
  }
  if (node.kind === 'asset') {
    return <AssetPanel key={node.path} repository={repository} node={node} onNavigate={onNavigate} />;
  }
  if (node.kind === 'component') {
    return <ComponentPanel key={node.path} repository={repository} node={node} onNavigate={onNavigate} />;
  }
  return <FolderPanel key={node.path} repository={repository} node={node} onNavigate={onNavigate} />;
}

function Browser({ repository }: { repository: RepositoryItem }) {
  const t = useT();
  const navigate = useNavigate();
  const phone = useMediaQuery(PHONE);
  const location = useLocation();
  const selection = selectionFromPathname(location.pathname);
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(ancestorsOf(selection)));
  const [showDetails, setShowDetails] = useState(selection !== '');
  const [search, setSearch] = useState('');
  const query = useDebounced(search.trim(), 300);
  const [width, setWidth] = useTreeWidth();
  const split = useRef<HTMLDivElement>(null);
  const [room, setRoom] = useState(1200);

  useEffect(() => {
    const wanted = ancestorsOf(selection);
    setExpanded((current) => {
      if (wanted.every((path) => current.has(path))) {
        return current;
      }
      return new Set([...current, ...wanted]);
    });
  }, [selection]);

  useEffect(() => {
    const element = split.current;
    if (!element || typeof ResizeObserver === 'undefined') {
      return;
    }
    const observer = new ResizeObserver((entries) => {
      const box = entries[0]?.contentRect;
      if (box) {
        setRoom(box.width);
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const folders = useMemo(() => [ROOT, ...[...expanded].filter((path) => path !== ROOT)], [expanded]);
  const listings = useQueries({
    queries: folders.map((folder) => {
      const key = treePath(repository.name, folder);
      return {
        queryKey: apiKey(key),
        queryFn: ({ signal }: { signal: AbortSignal }) => api.get<TreeListing>(key, { signal }),
      };
    }),
  });
  const loaded = new Map<string, Loaded>();
  folders.forEach((folder, index) => {
    const result = listings[index];
    if (result) {
      loaded.set(folder, { status: result.status, listing: result.data, error: result.error });
    }
  });
  const rows = flatten(loaded, expanded);
  const parent = selection === '' ? undefined : loaded.get(parentOf(selection));
  const node = selection === '' ? undefined : findNode(parent?.listing, selection);
  const missing = parent?.status === 'error' || (parent?.status === 'success' && !node);
  const root = loaded.get(ROOT);

  const open = (path: string) => {
    navigate(browseUrl(repository.name, path));
    setShowDetails(true);
  };
  const select = (selected: TreeNode) => {
    navigate(browseUrl(repository.name, selected.path));
    if (selected.kind === 'asset') {
      setShowDetails(true);
    }
  };
  const toggle = (path: string, isOpen: boolean) => {
    setExpanded((current) => {
      const next = new Set(current);
      if (isOpen) {
        next.add(path);
      } else {
        for (const known of current) {
          if (known.startsWith(path)) {
            next.delete(known);
          }
        }
      }
      return next;
    });
  };
  const retry = (folder: string) => {
    const index = folders.indexOf(folder);
    listings[index]?.refetch();
  };

  const treeLabel = t('browse.tree.label', { name: repository.name });
  const searchBox = (
    <TextField.Root
      type="search"
      value={search}
      placeholder={t('browse.search.placeholder', { name: repository.name })}
      aria-label={t('browse.search.label', { name: repository.name })}
      onChange={(event) => setSearch(event.target.value)}
    >
      <TextField.Slot>
        <Search size={15} aria-hidden="true" />
      </TextField.Slot>
      {search !== '' && (
        <TextField.Slot>
          <IconButton
            size="1"
            variant="ghost"
            color="gray"
            aria-label={t('browse.search.clear')}
            onClick={() => setSearch('')}
          >
            <X size={14} aria-hidden="true" />
          </IconButton>
        </TextField.Slot>
      )}
    </TextField.Root>
  );

  const treePane = (
    <section className={styles.treePane} id="browse-tree" aria-label={treeLabel}>
      <div className={styles.paneHead}>{searchBox}</div>
      <div className={styles.treeScroll}>
        {query !== '' ? (
          <RepositorySearch repository={repository} query={query} selection={selection} onOpen={open} />
        ) : root?.status === 'error' ? (
          <ErrorState error={root.error} onRetry={() => retry(ROOT)} title={t('browse.tree.rootError')} />
        ) : root?.status === 'success' && root.listing?.children.length === 0 ? (
          <EmptyState
            title={t('browse.tree.emptyTitle')}
            description={
              repository.type === 'proxy' ? t('browse.tree.emptyProxy') : t('browse.tree.emptyText')
            }
          />
        ) : (
          <RepositoryTree
            label={treeLabel}
            rows={rows}
            selection={selection}
            onSelect={select}
            onToggle={toggle}
            onRetry={retry}
            onDetails={phone ? (target) => open(target.path) : undefined}
          />
        )}
      </div>
    </section>
  );

  const details = (
    <section className={styles.detailsPane} aria-label={t('browse.details.label')}>
      <Details
        repository={repository}
        node={node}
        selection={selection}
        missing={missing}
        onNavigate={open}
      />
    </section>
  );

  if (phone) {
    return showDetails && selection !== '' ? (
      <div className={styles.phone}>
        <Button variant="soft" color="gray" onClick={() => setShowDetails(false)} className={styles.back}>
          <ArrowLeft size={15} aria-hidden="true" />
          {t('browse.details.back')}
        </Button>
        {details}
      </div>
    ) : (
      <div className={styles.phone}>{treePane}</div>
    );
  }

  const max = Math.max(MIN_WIDTH, Math.min(720, room - 360));
  const shown = clamp(width, MIN_WIDTH, max);
  return (
    <div ref={split} className={styles.split} style={{ ['--tree-width' as string]: `${shown}px` }}>
      {treePane}
      <Splitter
        value={shown}
        min={MIN_WIDTH}
        max={max}
        defaultValue={DEFAULT_WIDTH}
        label={t('browse.tree.resize')}
        controls="browse-tree"
        onChange={setWidth}
      />
      {details}
    </div>
  );
}

export default function RepositoryPage() {
  const t = useT();
  const { state } = useSession();
  const { repository: name = '' } = useParams();
  const list = useApi<RepositoryItem[]>(REPOSITORIES_PATH);
  const repository = list.data?.find((item) => item.name === name);
  const crumbs = [{ label: t('browse.list.title'), to: '/browse' }, { label: name }];

  if (list.isPending) {
    return (
      <Page title={name} breadcrumbs={crumbs}>
        <Text size="2" color="gray" aria-busy="true">
          {t('browse.tree.loading')}
        </Text>
      </Page>
    );
  }
  if (list.isError) {
    return (
      <Page title={name} breadcrumbs={crumbs}>
        <ErrorState error={list.error} onRetry={() => list.refetch()} />
      </Page>
    );
  }
  if (!repository) {
    return (
      <Page title={t('browse.repository.notFoundTitle')} breadcrumbs={crumbs}>
        <EmptyState
          icon={FolderX}
          title={t('browse.repository.notFoundTitle')}
          description={t('browse.repository.notFoundText', { name })}
          action={
            <Button asChild variant="soft">
              <Link to="/browse">{t('browse.repository.allRepositories')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }
  return (
    <Page
      title={repository.name}
      breadcrumbs={crumbs}
      description={
        <Flex as="span" gap="2" align="center" wrap="wrap" className={styles.badges}>
          <FormatBadge format={repository.format} />
          <TypeBadge type={repository.type} />
          <OnlineStatus online={repository.online} />
          <UrlLine url={repository.url} label={t('browse.repository.copyUrl')} />
        </Flex>
      }
      actions={
        <>
          {supportsUsage(repository.format) && (
            <UsageDialog
              title={t('browse.usage.title', { name: repository.name })}
              context={{
                format: repository.format,
                repositoryName: repository.name,
                repositoryType: repository.type,
                baseUrl: state.baseUrl,
              }}
              trigger={
                <Button variant="soft">
                  <Plug size={15} aria-hidden="true" />
                  {t('browse.repository.usage')}
                </Button>
              }
            />
          )}
          {repository.permissions.admin && (
            <Button asChild variant="soft" color="gray">
              <Link to={settingsPath(repository.name)}>
                <Settings2 size={15} aria-hidden="true" />
                {t('browse.repository.settings')}
              </Link>
            </Button>
          )}
        </>
      }
    >
      <Browser key={repository.name} repository={repository} />
    </Page>
  );
}
