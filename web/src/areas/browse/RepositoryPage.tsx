import { Badge, Button, Heading, IconButton, Text, TextField } from '@radix-ui/themes';
import { useQueries } from '@tanstack/react-query';
import { Copy, FolderOpen, FolderX, RefreshCw, Search, Settings, X } from 'lucide-react';
import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react';
import { Link, useLocation, useNavigate, useParams } from 'react-router';
import { api, apiKey, useApi } from '../../api';
import { useT } from '../../i18n';
import {
  copyText,
  EmptyState,
  ErrorState,
  formatColor,
  formatLabel,
  PHONE,
  readStored,
  useDocumentTitle,
  useMediaQuery,
  useToast,
  writeStored,
} from '../../ui';
import { AssetPanel, assetPath, ComponentPanel, FolderPanel, treePath } from './details/panels';
import { settingsPath } from './list/RepositoryListPage';
import { OnlineStatus } from './parts';
import styles from './RepositoryPage.module.css';
import { RepositorySearch } from './RepositorySearch';
import {
  ancestorsOf,
  browseUrl,
  expandAlone,
  findNode,
  flatten,
  type Loaded,
  parentOf,
  ROOT,
  selectionFromPathname,
} from './tree/model';
import { RepositoryTree } from './tree/RepositoryTree';
import { clamp, Splitter } from './tree/Splitter';
import {
  type AssetDetails,
  REPOSITORIES_PATH,
  type RepositoryItem,
  type TreeListing,
  type TreeNode,
} from './types';

export const WIDTH_KEY = 'astrum.browse.treeWidth';
const DEFAULT_WIDTH = 340;
const MIN_WIDTH = 220;

const TREE_TITLES: Record<string, string> = {
  raw: 'browse.tree.title.raw',
  maven2: 'browse.tree.title.maven2',
  docker: 'browse.tree.title.docker',
  npm: 'browse.tree.title.npm',
  pypi: 'browse.tree.title.pypi',
  nuget: 'browse.tree.title.nuget',
  helm: 'browse.tree.title.helm',
};

const TYPE_LABELS = {
  hosted: 'browse.list.type.hosted',
  proxy: 'browse.list.type.proxy',
  group: 'browse.list.type.group',
} as const;

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

function withSlash(url: string): string {
  return url.endsWith('/') ? url : `${url}/`;
}

function Crumbs({ name }: { name: string }) {
  const t = useT();
  const navigate = useNavigate();
  return (
    <nav aria-label={t('browse.repository.breadcrumb')} className={styles.crumbs}>
      <Text size="2" color="gray">
        <button type="button" className={styles.crumb} onClick={() => navigate('/browse')}>
          {t('browse.list.title')}
        </button>
        <span aria-hidden="true" className={styles.separator}>
          /
        </span>
        <span aria-current="page">{name}</span>
      </Text>
    </nav>
  );
}

function HeaderSearch({ repository, onOpen }: { repository: RepositoryItem; onOpen(path: string): void }) {
  const t = useT();
  const [text, setText] = useState('');
  const [open, setOpen] = useState(false);
  const query = useDebounced(text.trim(), 300);
  const box = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) {
      return;
    }
    const close = (event: MouseEvent) => {
      if (box.current && !box.current.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', close);
    return () => document.removeEventListener('mousedown', close);
  }, [open]);
  return (
    <div ref={box} className={styles.search}>
      <TextField.Root
        type="search"
        value={text}
        placeholder={t('browse.search.placeholder', { name: repository.name })}
        aria-label={t('browse.search.label', { name: repository.name })}
        onChange={(event) => {
          setText(event.target.value);
          setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        onKeyDown={(event) => {
          if (event.key === 'Escape') {
            setOpen(false);
          }
        }}
      >
        <TextField.Slot>
          <Search size={16} aria-hidden="true" />
        </TextField.Slot>
        {text !== '' && (
          <TextField.Slot>
            <IconButton
              size="1"
              variant="ghost"
              color="gray"
              aria-label={t('browse.search.clear')}
              onClick={() => setText('')}
            >
              <X size={14} aria-hidden="true" />
            </IconButton>
          </TextField.Slot>
        )}
      </TextField.Root>
      {open && query !== '' && (
        <div className={styles.searchPanel}>
          <RepositorySearch
            repository={repository}
            query={query}
            selection=""
            onOpen={(target) => {
              setOpen(false);
              onOpen(target);
            }}
          />
        </div>
      )}
    </div>
  );
}

function Header({ repository, onOpen }: { repository: RepositoryItem; onOpen(path: string): void }) {
  const t = useT();
  const toast = useToast();
  const copy = async () => {
    try {
      await copyText(withSlash(repository.url));
      toast.success(t('browse.repository.urlCopied'));
    } catch {
      toast.error(t('browse.list.urlCopyFailed'));
    }
  };
  return (
    <header className={styles.header}>
      <Crumbs name={repository.name} />
      <div className={styles.titleRow}>
        <div className={styles.titleGroup}>
          <Heading as="h1" size="7" weight="bold" className={styles.title}>
            {repository.name}
          </Heading>
          <div className={styles.badges}>
            <Badge size="2" color={formatColor(repository.format)} variant="soft" radius="full">
              {formatLabel(repository.format)}
            </Badge>
            <Badge size="2" color="gray" variant="soft" radius="full" className={styles.lower}>
              {t(TYPE_LABELS[repository.type])}
            </Badge>
            <OnlineStatus online={repository.online} />
          </div>
        </div>
        <div className={styles.headerActions}>
          <HeaderSearch repository={repository} onOpen={onOpen} />
          <Button variant="surface" color="gray" highContrast onClick={copy}>
            <Copy size={14} aria-hidden="true" />
            {t('browse.repository.copyUrl')}
          </Button>
          {repository.permissions.admin && (
            <Button asChild variant="surface" color="gray" highContrast>
              <Link to={settingsPath(repository.name)}>
                <Settings size={14} aria-hidden="true" />
                {t('browse.repository.settings')}
              </Link>
            </Button>
          )}
        </div>
      </div>
    </header>
  );
}

function Missing({ repository, selection }: { repository: RepositoryItem; selection: string }) {
  const t = useT();
  return (
    <section className={styles.emptyCard}>
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
    </section>
  );
}

function Resolve({ repository, selection }: { repository: RepositoryItem; selection: string }) {
  const navigate = useNavigate();
  const query = useApi<AssetDetails>(assetPath(repository.name, selection));
  const target = query.data?.treePath;
  useEffect(() => {
    if (target && target !== selection) {
      navigate(browseUrl(repository.name, target), { replace: true });
    }
  }, [target, selection, repository.name, navigate]);
  if (query.isPending || (target && target !== selection)) {
    return null;
  }
  return <Missing repository={repository} selection={selection} />;
}

function Nothing() {
  const t = useT();
  return (
    <section className={styles.nothing}>
      <Text as="p" size="3" weight="medium">
        {t('browse.repository.nothingSelected')}
      </Text>
      <Text as="p" size="2" color="gray">
        {t('browse.repository.nothingSelectedText')}
      </Text>
    </section>
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
    return <Nothing />;
  }
  if (!node) {
    if (!missing) {
      return (
        <Text as="p" size="2" color="gray" aria-busy="true">
          {t('browse.tree.loading')}
        </Text>
      );
    }
    return selection.endsWith('/') ? (
      <Missing repository={repository} selection={selection} />
    ) : (
      <Resolve repository={repository} selection={selection} />
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

function opened(selection: string): string[] {
  const wanted = ancestorsOf(selection);
  return selection.endsWith('/') && selection !== ROOT ? [...wanted, selection] : wanted;
}

function Browser({ repository }: { repository: RepositoryItem }) {
  const t = useT();
  const navigate = useNavigate();
  const phone = useMediaQuery(PHONE);
  const location = useLocation();
  const selection = selectionFromPathname(location.pathname);
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(opened(selection)));
  const [filter, setFilter] = useState('');
  const [width, setWidth] = useTreeWidth();
  const split = useRef<HTMLDivElement>(null);
  const [room, setRoom] = useState(1200);

  useEffect(() => {
    const wanted = opened(selection);
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

  const open = (path: string) => navigate(browseUrl(repository.name, path));
  const select = (selected: TreeNode) => navigate(browseUrl(repository.name, selected.path));
  const toggle = (path: string, isOpen: boolean) => {
    setExpanded((current) => {
      if (isOpen) {
        return expandAlone(current, path);
      }
      const next = new Set(current);
      for (const known of current) {
        if (known.startsWith(path)) {
          next.delete(known);
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
  const title = t(TREE_TITLES[repository.format] ?? 'browse.tree.title.default');
  let body: ReactNode;
  if (root?.status === 'error') {
    body = <ErrorState error={root.error} onRetry={() => retry(ROOT)} title={t('browse.tree.rootError')} />;
  } else if (root?.status === 'success' && root.listing?.children.length === 0) {
    body = (
      <div className={styles.empty}>
        <FolderOpen size={32} aria-hidden="true" className={styles.emptyIcon} />
        <Text as="p" size="3" weight="medium">
          {t('browse.tree.emptyTitle')}
        </Text>
        <Text as="p" size="1" color="gray" className={styles.emptyText}>
          {t('browse.tree.emptyText')}
        </Text>
        <Button onClick={() => retry(ROOT)}>
          <RefreshCw size={16} aria-hidden="true" />
          {t('browse.tree.retry')}
        </Button>
      </div>
    );
  } else {
    body = (
      <RepositoryTree
        label={treeLabel}
        format={repository.format}
        rows={rows}
        selection={selection}
        filter={filter}
        onSelect={select}
        onToggle={toggle}
        onRetry={retry}
      />
    );
  }

  const treePane = (
    <section className={styles.treePane} id="browse-tree" aria-label={treeLabel}>
      <div className={styles.paneHead}>
        <Text as="p" size="3" weight="bold">
          {title}
        </Text>
        <TextField.Root
          type="search"
          value={filter}
          placeholder={t('browse.tree.filterPlaceholder')}
          aria-label={t('browse.tree.filterLabel')}
          onChange={(event) => setFilter(event.target.value)}
        >
          <TextField.Slot>
            <Search size={14} aria-hidden="true" />
          </TextField.Slot>
          {filter !== '' && (
            <TextField.Slot>
              <IconButton
                size="1"
                variant="ghost"
                color="gray"
                aria-label={t('browse.tree.clearFilter')}
                onClick={() => setFilter('')}
              >
                <X size={14} aria-hidden="true" />
              </IconButton>
            </TextField.Slot>
          )}
        </TextField.Root>
      </div>
      <div className={styles.treeScroll}>{body}</div>
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
    return (
      <div className={styles.phone}>
        {treePane}
        {details}
      </div>
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

function Shell({ name, children }: { name: string; children: ReactNode }) {
  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <Crumbs name={name} />
        <Heading as="h1" size="7" weight="bold" className={styles.title}>
          {name}
        </Heading>
      </header>
      {children}
    </div>
  );
}

export default function RepositoryPage() {
  const t = useT();
  const navigate = useNavigate();
  const { repository: name = '' } = useParams();
  const list = useApi<RepositoryItem[]>(REPOSITORIES_PATH);
  const repository = list.data?.find((item) => item.name === name);
  useDocumentTitle(
    repository ? repository.name : list.isPending ? name : t('browse.repository.notFoundTitle'),
  );

  if (list.isPending) {
    return (
      <Shell name={name}>
        <Text size="2" color="gray" aria-busy="true">
          {t('browse.tree.loading')}
        </Text>
      </Shell>
    );
  }
  if (list.isError) {
    return (
      <Shell name={name}>
        <ErrorState error={list.error} onRetry={() => list.refetch()} />
      </Shell>
    );
  }
  if (!repository) {
    return (
      <Shell name={t('browse.repository.notFoundTitle')}>
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
      </Shell>
    );
  }
  return (
    <div className={styles.page}>
      <Header repository={repository} onOpen={(target) => navigate(browseUrl(repository.name, target))} />
      <Browser key={repository.name} repository={repository} />
    </div>
  );
}
