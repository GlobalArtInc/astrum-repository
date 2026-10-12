import { Badge, Button, Flex, SegmentedControl, Skeleton, Text } from '@radix-ui/themes';
import {
  ChartNoAxesColumn,
  Database,
  Download,
  File,
  FolderOpen,
  type LucideIcon,
  Package,
  Search,
  Upload as UploadIcon,
} from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { Link } from 'react-router';
import { useApi } from '../../api';
import { useCan, useSession } from '../../auth/state';
import { formatBytes, formatNumber, useI18n, useT } from '../../i18n';
import { EmptyState, ErrorState, formatName, Page, RelativeTime } from '../../ui';
import { BROWSE_NEED } from '../browse';
import { componentPageUrl, componentTitle } from '../browse/links';
import { browseUrl } from '../browse/tree/model';
import styles from './Dashboard.module.css';
import {
  DASHBOARD_PATH,
  type Dashboard,
  type FormatCount,
  fileName,
  type StoreUsage,
  share,
  storeUse,
  type TopAsset,
  type TopComponent,
  type Upload,
} from './types';

const SEARCH_READ = 'astrum:search:read';
const UPLOAD = 'astrum:component:create';
const TYPES = ['hosted', 'proxy', 'group'] as const;
const TYPE_LABELS = {
  hosted: 'dashboard.types.hosted',
  proxy: 'dashboard.types.proxy',
  group: 'dashboard.types.group',
} as const;

function Card({
  title,
  id,
  actions,
  children,
}: {
  title: string;
  id: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className={styles.card} aria-labelledby={id}>
      <div className={styles.cardHead}>
        <Text as="p" size="4" weight="bold" id={id}>
          {title}
        </Text>
        {actions}
      </div>
      {children}
    </section>
  );
}

function Tile({
  icon: Icon,
  label,
  value,
  caption,
}: {
  icon: LucideIcon;
  label: string;
  value: string;
  caption?: string;
}) {
  return (
    <div className={styles.tile}>
      <Text as="p" size="2" color="gray" className={styles.tileLabel}>
        <Icon size={15} aria-hidden="true" />
        {label}
      </Text>
      <Text as="p" size="7" weight="bold" className={styles.tileValue}>
        {value}
      </Text>
      {caption && (
        <Text as="p" size="1" color="gray">
          {caption}
        </Text>
      )}
    </div>
  );
}

function QuickLinks() {
  const t = useT();
  const canBrowse = useCan(BROWSE_NEED);
  const canSearch = useCan(SEARCH_READ);
  const canUpload = useCan(UPLOAD);
  return (
    <>
      {canBrowse && (
        <Button asChild variant="soft">
          <Link to="/browse">
            <FolderOpen size={15} aria-hidden="true" />
            {t('dashboard.links.browse')}
          </Link>
        </Button>
      )}
      {canSearch && (
        <Button asChild variant="soft">
          <Link to="/search">
            <Search size={15} aria-hidden="true" />
            {t('dashboard.links.search')}
          </Link>
        </Button>
      )}
      {canUpload && (
        <Button asChild variant="soft">
          <Link to="/upload">
            <UploadIcon size={15} aria-hidden="true" />
            {t('dashboard.links.upload')}
          </Link>
        </Button>
      )}
    </>
  );
}

function Formats({ formats }: { formats: FormatCount[] }) {
  const t = useT();
  const max = Math.max(0, ...formats.map((entry) => entry.total));
  const sorted = [...formats].sort(
    (left, right) => right.total - left.total || left.format.localeCompare(right.format),
  );
  return (
    <Card title={t('dashboard.formats.title')} id="dashboard-formats">
      <ul className={styles.legend} aria-label={t('dashboard.formats.legend')}>
        {TYPES.map((type) => (
          <li key={type}>
            <span className={styles.swatch} data-type={type} aria-hidden="true" />
            {t(TYPE_LABELS[type])}
          </li>
        ))}
      </ul>
      <ul className={styles.bars}>
        {sorted.map((entry) => {
          const parts = TYPES.filter((type) => entry[type] > 0);
          return (
            <li key={entry.format} className={styles.barRow}>
              <div className={styles.barHead}>
                <Text size="2" weight="medium">
                  {formatName(entry.format)}
                </Text>
                <Text size="2" weight="bold" className={styles.number}>
                  {entry.total}
                </Text>
              </div>
              <div className={styles.track} style={{ width: `${share(entry.total, max)}%` }}>
                {parts.map((type) => (
                  <span
                    key={type}
                    className={styles.segment}
                    data-type={type}
                    style={{ flexGrow: entry[type] }}
                    title={`${t(TYPE_LABELS[type])}: ${entry[type]}`}
                  />
                ))}
              </div>
              <Text size="1" color="gray">
                {parts
                  .map((type) =>
                    t('dashboard.formats.part', { type: t(TYPE_LABELS[type]), count: entry[type] }),
                  )
                  .join(' · ')}
              </Text>
            </li>
          );
        })}
      </ul>
    </Card>
  );
}

function DownloadBar({ value, max }: { value: number; max: number }) {
  return (
    <span className={styles.miniTrack} aria-hidden="true">
      <span className={styles.miniBar} style={{ width: `${share(value, max)}%` }} />
    </span>
  );
}

function componentLink(
  canSearch: boolean,
  entry: { format: string; group: string; name: string; version: string; repository: string },
) {
  return canSearch
    ? componentPageUrl(entry.format, entry.group, entry.name, entry.version || undefined)
    : browseUrl(entry.repository);
}

function TopDownloads({ components, assets }: { components: TopComponent[]; assets: TopAsset[] }) {
  const t = useT();
  const { language } = useI18n();
  const canSearch = useCan(SEARCH_READ);
  const [view, setView] = useState<'components' | 'assets'>('components');
  const rows =
    view === 'components'
      ? components.map((entry) => ({
          key: entry.componentId,
          to: componentLink(canSearch, entry),
          name: componentTitle(entry.format, entry.group, entry.name),
          version: entry.version,
          repository: entry.repository,
          format: entry.format,
          downloads: entry.downloads,
        }))
      : assets.map((entry) => ({
          key: entry.assetId,
          to: browseUrl(entry.repository, entry.path),
          name: entry.path,
          version: '',
          repository: entry.repository,
          format: entry.format,
          downloads: entry.downloads,
        }));
  const max = Math.max(0, ...rows.map((row) => row.downloads));
  return (
    <Card
      title={t('dashboard.top.title')}
      id="dashboard-top"
      actions={
        <SegmentedControl.Root
          size="1"
          value={view}
          onValueChange={(next) => setView(next === 'assets' ? 'assets' : 'components')}
          aria-label={t('dashboard.top.view')}
        >
          <SegmentedControl.Item value="components">{t('dashboard.top.components')}</SegmentedControl.Item>
          <SegmentedControl.Item value="assets">{t('dashboard.top.assets')}</SegmentedControl.Item>
        </SegmentedControl.Root>
      }
    >
      {rows.length === 0 ? (
        <Text as="p" size="2" color="gray">
          {t('dashboard.top.empty')}
        </Text>
      ) : (
        <ol
          className={styles.ranked}
          aria-label={view === 'components' ? t('dashboard.top.components') : t('dashboard.top.assets')}
        >
          {rows.map((row, index) => (
            <li key={row.key} className={styles.rankedRow}>
              <span className={styles.rank} aria-hidden="true">
                {index + 1}
              </span>
              <span className={styles.rankedName}>
                <Link to={row.to} className={styles.link}>
                  {row.name}
                </Link>
                {row.version && <code className={styles.version}>{row.version}</code>}
                <Text as="span" size="1" color="gray" className={styles.meta}>
                  {row.repository} · {formatName(row.format)}
                </Text>
              </span>
              <DownloadBar value={row.downloads} max={max} />
              <Text size="2" weight="bold" className={styles.number}>
                {formatNumber(language, row.downloads)}
              </Text>
            </li>
          ))}
        </ol>
      )}
    </Card>
  );
}

function uploadLink(canSearch: boolean, upload: Upload): string {
  if (upload.kind === 'component' && canSearch && upload.format !== 'raw') {
    return componentPageUrl(upload.format, upload.group ?? '', upload.name, upload.version || undefined);
  }
  return browseUrl(upload.repository, upload.path);
}

function RecentUploads({ uploads }: { uploads: Upload[] }) {
  const t = useT();
  const canSearch = useCan(SEARCH_READ);
  return (
    <Card title={t('dashboard.recent.title')} id="dashboard-recent">
      {uploads.length === 0 ? (
        <Text as="p" size="2" color="gray">
          {t('dashboard.recent.empty')}
        </Text>
      ) : (
        <ul className={styles.list}>
          {uploads.map((upload) => {
            const component = upload.kind === 'component' && upload.format !== 'raw';
            const Icon = component ? Package : File;
            return (
              <li key={`${upload.repository}|${upload.assetId}`} className={styles.listRow}>
                <span className={styles.icon} aria-hidden="true">
                  <Icon size={16} />
                </span>
                <span className={styles.rankedName}>
                  <Link to={uploadLink(canSearch, upload)} className={styles.link}>
                    {component
                      ? componentTitle(upload.format, upload.group ?? '', upload.name)
                      : fileName(upload.path)}
                  </Link>
                  {component && upload.version && <code className={styles.version}>{upload.version}</code>}
                  <Text as="span" size="1" color="gray" className={styles.meta}>
                    {[
                      upload.repository,
                      upload.uploader,
                      t('dashboard.recent.files', { count: upload.assetCount }),
                    ].join(' · ')}
                  </Text>
                </span>
                <Text size="1" color="gray" className={styles.when}>
                  <RelativeTime value={upload.uploadedAt} />
                </Text>
              </li>
            );
          })}
        </ul>
      )}
    </Card>
  );
}

function BlobStores({ stores }: { stores: StoreUsage[] }) {
  const t = useT();
  const { language } = useI18n();
  return (
    <Card title={t('dashboard.stores.title')} id="dashboard-stores">
      {stores.length === 0 ? (
        <Text as="p" size="2" color="gray">
          {t('dashboard.stores.empty')}
        </Text>
      ) : (
        <ul className={styles.list}>
          {stores.map((store) => {
            const used = storeUse(store);
            return (
              <li key={store.name} className={styles.store}>
                <div className={styles.barHead}>
                  <Flex gap="2" align="center" wrap="wrap">
                    <Text size="2" weight="medium">
                      {store.name}
                    </Text>
                    <Badge color="gray" variant="soft" radius="full">
                      {store.type}
                    </Badge>
                    {store.unavailable && (
                      <Badge color="red" variant="soft">
                        {t('dashboard.stores.unavailable')}
                      </Badge>
                    )}
                  </Flex>
                  <Text size="2" weight="bold" className={styles.number}>
                    {formatBytes(language, store.totalSize)}
                  </Text>
                </div>
                {used !== null && (
                  <span className={styles.miniTrack} aria-hidden="true">
                    <span
                      className={styles.miniBar}
                      style={{ width: `${Math.max(used, used > 0 ? 1 : 0)}%` }}
                    />
                  </span>
                )}
                <Text size="1" color="gray">
                  {[
                    t('dashboard.stores.blobs', {
                      count: store.blobCount,
                      shown: formatNumber(language, store.blobCount),
                    }),
                    store.availableSpace !== null
                      ? t('dashboard.stores.free', { size: formatBytes(language, store.availableSpace) })
                      : '',
                  ]
                    .filter(Boolean)
                    .join(' · ')}
                </Text>
              </li>
            );
          })}
        </ul>
      )}
    </Card>
  );
}

function Loading() {
  return (
    <div className={styles.grid} aria-busy="true">
      <div className={styles.tiles}>
        {[0, 1, 2, 3, 4].map((index) => (
          <Skeleton key={index} height="112px" />
        ))}
      </div>
      <Skeleton height="260px" />
    </div>
  );
}

function Overview({ data }: { data: Dashboard }) {
  const t = useT();
  const { language } = useI18n();
  return (
    <div className={styles.grid}>
      <div className={styles.tiles}>
        <Tile
          icon={FolderOpen}
          label={t('dashboard.kpi.repositories')}
          value={formatNumber(language, data.repositories.total)}
          caption={TYPES.map((type) =>
            t('dashboard.formats.part', {
              type: t(TYPE_LABELS[type]),
              count: data.repositories.byType[type],
            }),
          ).join(' · ')}
        />
        <Tile
          icon={Package}
          label={t('dashboard.kpi.components')}
          value={formatNumber(language, data.components)}
        />
        <Tile icon={File} label={t('dashboard.kpi.assets')} value={formatNumber(language, data.assets)} />
        <Tile
          icon={Database}
          label={t('dashboard.kpi.storage')}
          value={formatBytes(language, data.totalSize)}
          caption={data.blobStores ? t('dashboard.kpi.stores', { count: data.blobStores.length }) : undefined}
        />
        <Tile
          icon={Download}
          label={t('dashboard.kpi.downloads')}
          value={formatNumber(language, data.downloads.total)}
          caption={t('dashboard.kpi.downloadsCaption')}
        />
      </div>
      <div className={styles.cards}>
        <div className={styles.column}>
          <TopDownloads components={data.downloads.topComponents} assets={data.downloads.topAssets} />
          <RecentUploads uploads={data.recentUploads.filter((upload) => upload.kind === 'component')} />
        </div>
        <div className={styles.column}>
          <Formats formats={data.repositories.byFormat} />
          {data.blobStores && <BlobStores stores={data.blobStores} />}
        </div>
      </div>
    </div>
  );
}

export function DashboardPage() {
  const t = useT();
  const { state } = useSession();
  const canAdminister = useCan({ some: 'astrum:repository-admin' });
  const query = useApi<Dashboard>(DASHBOARD_PATH);
  return (
    <Page
      title={t('dashboard.home.title')}
      description={t('dashboard.home.description')}
      actions={<QuickLinks />}
    >
      {query.isPending ? (
        <Loading />
      ) : query.isError ? (
        <ErrorState error={query.error} onRetry={() => query.refetch()} />
      ) : query.data.repositories.total === 0 ? (
        <div className={styles.emptyCard}>
          <EmptyState
            icon={ChartNoAxesColumn}
            title={t('dashboard.home.placeholderTitle')}
            description={t('dashboard.home.placeholderText')}
            action={
              canAdminister && state.user ? (
                <Button asChild variant="soft">
                  <Link to="/admin/repository/repositories">{t('dashboard.home.createRepository')}</Link>
                </Button>
              ) : undefined
            }
          />
        </div>
      ) : (
        <Overview data={query.data} />
      )}
    </Page>
  );
}
