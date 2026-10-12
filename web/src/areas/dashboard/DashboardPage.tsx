import { Badge, Button, Card, SegmentedControl, Select, Skeleton } from '@radix-ui/themes';
import { keepPreviousData } from '@tanstack/react-query';
import {
  Archive,
  ArrowRight,
  ChartColumn,
  Database,
  Download,
  File,
  Folder,
  type LucideIcon,
  Package,
} from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { Link } from 'react-router';
import { useApi, withQuery } from '../../api';
import { formatBytes, formatNumber, useI18n, useT } from '../../i18n';
import { ErrorState, FormatLogo, formatName, Page, RelativeTime } from '../../ui';
import { browseUrl } from '../browse/tree/model';
import styles from './Dashboard.module.css';
import { CHART_HEIGHT, DownloadsChart, shortDay } from './DownloadsChart';
import {
  componentName,
  PERIODS,
  type Period,
  peakOf,
  type RepositoryUsage,
  share,
  USAGE_PATH,
  type Usage,
  type UsageComponent,
} from './types';

const ALL = '*';
const RECENT = 6;
const CLEANUP_PATH = '/admin/repository/cleanup-policies';

function usePeriod() {
  const t = useT();
  return (days: number) => t('dashboard.home.days', { count: days });
}

function Heading({ id, children }: { id: string; children: ReactNode }) {
  return (
    <h2 id={id} className={styles.cardTitle}>
      {children}
    </h2>
  );
}

function Tile({
  icon: Icon,
  label,
  value,
  caption,
  tone,
}: {
  icon: LucideIcon;
  label: string;
  value: string;
  caption?: string;
  tone?: 'warning';
}) {
  return (
    <Card size="2" className={styles.tile}>
      <div className={styles.tileBody}>
        <span className={styles.tileLabel} data-tone={tone}>
          <Icon size={15} aria-hidden="true" />
          <span className={styles.ellipsis}>{label}</span>
        </span>
        <div className={styles.tileValue}>{value}</div>
        {caption && <div className={styles.caption}>{caption}</div>}
      </div>
    </Card>
  );
}

function Tiles({ data }: { data: Usage }) {
  const t = useT();
  const { language } = useI18n();
  const period = usePeriod();
  const totals = data.totals;
  return (
    <div className={styles.tilesFrame}>
      <div className={styles.tiles}>
        <Tile
          icon={Download}
          label={t('dashboard.kpi.downloads')}
          value={formatNumber(language, totals.downloadsInPeriod)}
          caption={t('dashboard.kpi.downloadsCaption', {
            today: totals.downloadsToday,
            average: Math.round(totals.downloadsInPeriod / Math.max(1, data.days)),
          })}
        />
        <Tile
          icon={Package}
          label={t('dashboard.kpi.components')}
          value={formatNumber(language, totals.components)}
          caption={t('dashboard.kpi.componentsCaption', { count: totals.componentsDownloadedInPeriod })}
        />
        <Tile
          icon={File}
          label={t('dashboard.kpi.assets')}
          value={formatNumber(language, totals.assets)}
          caption={t('dashboard.kpi.assetsCaption', { count: totals.repositories })}
        />
        <Tile
          icon={Database}
          label={t('dashboard.kpi.storage')}
          value={formatBytes(language, totals.storageBytes)}
          caption={
            totals.proxy > 0
              ? t('dashboard.kpi.storageCaption', { size: formatBytes(language, totals.proxyStorageBytes) })
              : undefined
          }
        />
        <Tile
          icon={Folder}
          label={t('dashboard.kpi.repositories')}
          value={formatNumber(language, totals.repositories)}
          caption={t('dashboard.kpi.repositoriesCaption', {
            hosted: totals.hosted,
            proxy: totals.proxy,
            group: totals.group,
          })}
        />
        <Tile
          icon={Archive}
          tone="warning"
          label={t('dashboard.kpi.unused', { period: period(data.unusedDays) })}
          value={formatNumber(language, data.unused.components)}
          caption={t('dashboard.kpi.unusedCaption', { size: formatBytes(language, data.unused.bytes) })}
        />
      </div>
    </div>
  );
}

function ChartCard({ data }: { data: Usage }) {
  const t = useT();
  const { language } = useI18n();
  const period = usePeriod();
  const peak = peakOf(data.downloadsPerDay);
  return (
    <Card size="3" asChild className={styles.card}>
      <section aria-labelledby="dashboard-chart">
        <div className={styles.chartHead}>
          <div>
            <Heading id="dashboard-chart">{t('dashboard.chart.title')}</Heading>
            <p className={styles.cardSubtitle}>
              {peak
                ? t('dashboard.chart.subtitle', {
                    period: period(data.days),
                    total: data.totals.downloadsInPeriod,
                    peak: peak.downloads,
                    date: shortDay(language, peak.day),
                  })
                : t('dashboard.chart.subtitleEmpty', { period: period(data.days) })}
            </p>
          </div>
          {peak && (
            <ul className={styles.legend}>
              <li>
                <span className={styles.swatch} data-kind="weekday" aria-hidden="true" />
                {t('dashboard.chart.weekdays')}
              </li>
              <li>
                <span className={styles.swatch} data-kind="weekend" aria-hidden="true" />
                {t('dashboard.chart.weekends')}
              </li>
            </ul>
          )}
        </div>
        {peak ? (
          <DownloadsChart days={data.downloadsPerDay} />
        ) : (
          <div className={styles.chartEmpty} style={{ height: CHART_HEIGHT }}>
            <ChartColumn size={28} strokeWidth={1.5} aria-hidden="true" />
            <p className={styles.emptyTitle}>{t('dashboard.chart.empty')}</p>
            <p className={styles.emptyHint}>{t('dashboard.chart.emptyHint')}</p>
          </div>
        )}
      </section>
    </Card>
  );
}

function Meter({ value, max }: { value: number; max: number }) {
  return (
    <span className={styles.track} aria-hidden="true">
      <span className={styles.fill} style={{ width: `${share(value, max)}%` }} />
    </span>
  );
}

function FormatName({ format }: { format: string }) {
  return (
    <Badge color="gray" variant="soft" size="1" className={styles.badge}>
      {formatName(format)}
    </Badge>
  );
}

function componentLink(component: UsageComponent): string {
  return browseUrl(component.repository, component.browsePath ?? '');
}

function TopCard({ data }: { data: Usage }) {
  const t = useT();
  const { language } = useI18n();
  const period = usePeriod();
  const [view, setView] = useState<'period' | 'all'>('period');
  const rows = view === 'period' ? data.topComponents : data.topComponentsAllTime;
  const max = Math.max(0, ...rows.map((row) => row.downloads));
  return (
    <Card size="3" asChild className={`${styles.card} ${styles.wide}`}>
      <section aria-labelledby="dashboard-top">
        <div className={styles.cardHead}>
          <Heading id="dashboard-top">{t('dashboard.top.title')}</Heading>
          <SegmentedControl.Root
            size="1"
            value={view}
            onValueChange={(next) => setView(next === 'all' ? 'all' : 'period')}
            aria-label={t('dashboard.home.period')}
          >
            <SegmentedControl.Item value="period">
              {t('dashboard.top.period', { period: period(data.days) })}
            </SegmentedControl.Item>
            <SegmentedControl.Item value="all">{t('dashboard.top.allTime')}</SegmentedControl.Item>
          </SegmentedControl.Root>
        </div>
        {rows.length === 0 ? (
          <p className={styles.cardEmpty}>
            {view === 'period' ? t('dashboard.top.empty') : t('dashboard.top.emptyAllTime')}
          </p>
        ) : (
          <div className={styles.tableFrame}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th className={styles.rankCell}>#</th>
                  <th>{t('dashboard.top.component')}</th>
                  <th>{t('dashboard.top.repository')}</th>
                  <th>{t('dashboard.top.downloads')}</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((row, index) => (
                  <tr key={row.id}>
                    <td className={styles.rankCell}>
                      <span className={styles.rank}>{index + 1}</span>
                    </td>
                    <td>
                      <span className={styles.nameLine}>
                        <Link to={componentLink(row)} className={styles.name}>
                          {componentName(row)}
                        </Link>
                        {row.version && <span className={styles.version}>{row.version}</span>}
                      </span>
                    </td>
                    <td>
                      <span className={styles.repositoryCell}>
                        <span className={styles.muted}>{row.repository}</span>
                        <FormatName format={row.format} />
                      </span>
                    </td>
                    <td>
                      <span className={styles.downloadsCell}>
                        <Meter value={row.downloads} max={max} />
                        <span className={styles.count}>{formatNumber(language, row.downloads)}</span>
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        <Link to="/search" className={styles.more}>
          {t('dashboard.top.search')}
          <ArrowRight size={14} aria-hidden="true" />
        </Link>
      </section>
    </Card>
  );
}

function RepositoryRows({
  rows,
  value,
  label,
}: {
  rows: RepositoryUsage[];
  value: (row: RepositoryUsage) => number;
  label: (row: RepositoryUsage) => string;
}) {
  const max = Math.max(0, ...rows.map(value));
  return (
    <ul className={styles.ranking}>
      {rows.map((row) => (
        <li key={row.name}>
          <span className={styles.rankingLine}>
            <span className={styles.repositoryCell}>
              <Link to={browseUrl(row.name)} className={styles.name}>
                {row.name}
              </Link>
              <FormatName format={row.format} />
            </span>
            <span className={styles.count}>{label(row)}</span>
          </span>
          <Meter value={value(row)} max={max} />
        </li>
      ))}
    </ul>
  );
}

function RepositoriesCard({ data }: { data: Usage }) {
  const t = useT();
  const { language } = useI18n();
  const [order, setOrder] = useState<'downloads' | 'size'>('downloads');
  const rows = order === 'downloads' ? data.topRepositoriesByDownloads : data.topRepositoriesByStorage;
  return (
    <Card size="3" asChild className={`${styles.card} ${styles.narrow}`}>
      <section aria-labelledby="dashboard-repositories">
        <div className={styles.cardHead}>
          <Heading id="dashboard-repositories">{t('dashboard.repositories.title')}</Heading>
          <SegmentedControl.Root
            size="1"
            value={order}
            onValueChange={(next) => setOrder(next === 'size' ? 'size' : 'downloads')}
            aria-label={t('dashboard.repositories.order')}
          >
            <SegmentedControl.Item value="downloads">
              {t('dashboard.repositories.byDownloads')}
            </SegmentedControl.Item>
            <SegmentedControl.Item value="size">{t('dashboard.repositories.bySize')}</SegmentedControl.Item>
          </SegmentedControl.Root>
        </div>
        {rows.length === 0 ? (
          <p className={styles.cardEmpty}>
            {order === 'downloads'
              ? t('dashboard.repositories.emptyDownloads')
              : t('dashboard.repositories.emptySize')}
          </p>
        ) : order === 'downloads' ? (
          <RepositoryRows
            rows={rows}
            value={(row) => row.downloads}
            label={(row) => formatNumber(language, row.downloads)}
          />
        ) : (
          <RepositoryRows
            rows={rows}
            value={(row) => row.storageBytes}
            label={(row) => formatBytes(language, row.storageBytes)}
          />
        )}
      </section>
    </Card>
  );
}

function RecentCard({ data }: { data: Usage }) {
  const t = useT();
  const rows = data.recentComponents.slice(0, RECENT);
  return (
    <Card size="3" asChild className={`${styles.card} ${styles.wide}`}>
      <section aria-labelledby="dashboard-recent">
        <Heading id="dashboard-recent">{t('dashboard.recent.title')}</Heading>
        {rows.length === 0 ? (
          <p className={styles.cardEmpty}>{t('dashboard.recent.empty')}</p>
        ) : (
          <ul className={styles.recent}>
            {rows.map((row) => (
              <li key={row.id}>
                <FormatLogo format={row.format} />
                <span className={styles.recentText}>
                  <span className={styles.nameLine}>
                    <Link to={componentLink(row)} className={styles.name}>
                      {componentName(row)}
                    </Link>
                    {row.version && <span className={styles.version}>{row.version}</span>}
                  </span>
                  <span className={styles.caption}>
                    {[row.repository, row.createdBy].filter(Boolean).join(' · ')}
                  </span>
                </span>
                <span className={styles.when}>
                  <RelativeTime value={row.created} />
                </span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </Card>
  );
}

function UnusedCard({ data }: { data: Usage }) {
  const t = useT();
  const { language } = useI18n();
  const period = usePeriod();
  const unused = data.unused;
  const max = Math.max(0, ...unused.repositories.map((row) => row.bytes));
  return (
    <Card size="3" asChild className={`${styles.card} ${styles.narrow}`}>
      <section aria-labelledby="dashboard-unused" className={styles.unused}>
        <div>
          <Heading id="dashboard-unused">
            {t('dashboard.unused.title', { period: period(data.unusedDays) })}
          </Heading>
          <p className={styles.unusedLine}>
            <span className={styles.unusedCount}>{formatNumber(language, unused.components)}</span>
            <span className={styles.muted}>
              {t('dashboard.unused.components', { count: unused.components })}
              {' · '}
              <strong className={styles.warning}>{formatBytes(language, unused.bytes)}</strong>
            </span>
          </p>
          <div className={styles.caption}>
            {t('dashboard.unused.never', {
              count: unused.neverDownloadedComponents,
              size: formatBytes(language, unused.neverDownloadedBytes),
            })}
          </div>
        </div>
        {unused.repositories.length === 0 ? (
          <p className={styles.muted}>{t('dashboard.unused.empty', { period: period(data.unusedDays) })}</p>
        ) : (
          <ul className={styles.ranking}>
            {unused.repositories.map((row) => (
              <li key={row.name}>
                <span className={styles.rankingLine}>
                  <span className={styles.repositoryCell}>
                    <Link to={browseUrl(row.name)} className={styles.name}>
                      {row.name}
                    </Link>
                    <FormatName format={row.format} />
                  </span>
                  <span className={styles.count}>{formatBytes(language, row.bytes)}</span>
                </span>
                <Meter value={row.bytes} max={max} />
                <span className={styles.caption}>
                  {t('dashboard.unused.count', { count: row.components })}
                </span>
              </li>
            ))}
          </ul>
        )}
        <Button asChild size="2" className={styles.cleanup}>
          <Link to={CLEANUP_PATH}>{t('dashboard.unused.cleanup')}</Link>
        </Button>
      </section>
    </Card>
  );
}

function Controls({
  data,
  days,
  repository,
  onDays,
  onRepository,
}: {
  data: Usage | undefined;
  days: Period;
  repository: string;
  onDays: (days: Period) => void;
  onRepository: (repository: string) => void;
}) {
  const t = useT();
  const period = usePeriod();
  return (
    <div className={styles.controls}>
      <SegmentedControl.Root
        size="2"
        value={String(days)}
        onValueChange={(next) => onDays((PERIODS.find((entry) => String(entry) === next) ?? 30) as Period)}
        aria-label={t('dashboard.home.period')}
        className={styles.periods}
      >
        {PERIODS.map((entry) => (
          <SegmentedControl.Item key={entry} value={String(entry)}>
            {period(entry)}
          </SegmentedControl.Item>
        ))}
      </SegmentedControl.Root>
      <Select.Root
        size="2"
        value={repository || ALL}
        onValueChange={(next) => onRepository(next === ALL ? '' : next)}
      >
        <Select.Trigger className={styles.repositorySelect} aria-label={t('dashboard.home.repository')} />
        <Select.Content position="popper">
          <Select.Item value={ALL}>{t('dashboard.home.allRepositories')}</Select.Item>
          {data && data.repositories.length > 0 && <Select.Separator />}
          {data?.repositories.map((entry) => (
            <Select.Item key={entry.name} value={entry.name}>
              {entry.name}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
    </div>
  );
}

function Loading() {
  const t = useT();
  return (
    <div className={styles.board} role="status" aria-busy="true" aria-label={t('dashboard.home.loading')}>
      <div className={styles.tilesFrame}>
        <div className={styles.tiles}>
          {[0, 1, 2, 3, 4, 5].map((index) => (
            <Skeleton key={index} height="110px" />
          ))}
        </div>
      </div>
      <Skeleton height="386px" />
    </div>
  );
}

function Board({ data }: { data: Usage }) {
  return (
    <div className={styles.board}>
      <Tiles data={data} />
      <ChartCard data={data} />
      <div className={styles.row}>
        <TopCard data={data} />
        <RepositoriesCard data={data} />
      </div>
      <div className={styles.row}>
        <RecentCard data={data} />
        <UnusedCard data={data} />
      </div>
    </div>
  );
}

export function DashboardPage() {
  const t = useT();
  const period = usePeriod();
  const [days, setDays] = useState<Period>(30);
  const [repository, setRepository] = useState('');
  const query = useApi<Usage>(withQuery(USAGE_PATH, { days, repository }), {
    placeholderData: keepPreviousData,
  });
  const data = query.data;
  const description = repository
    ? t('dashboard.home.subtitleRepository', { repository, period: period(days) })
    : t('dashboard.home.subtitle', { period: period(days) });
  return (
    <Page
      size="large"
      gutter="large"
      className={styles.overview}
      title={t('dashboard.home.title')}
      documentTitle={t('dashboard.home.documentTitle')}
      description={description}
      actionsAlign="end"
      actions={
        <Controls
          data={data}
          days={days}
          repository={repository}
          onDays={setDays}
          onRepository={setRepository}
        />
      }
    >
      <div className={styles.content}>
        {query.isError && !data ? (
          <ErrorState error={query.error} title={t('dashboard.home.error')} onRetry={() => query.refetch()} />
        ) : !data ? (
          <Loading />
        ) : data.repositories.length === 0 ? (
          <Card size="3" className={styles.card}>
            <p className={styles.cardEmpty}>{t('dashboard.home.noRepositories')}</p>
          </Card>
        ) : (
          <Board data={data} />
        )}
      </div>
    </Page>
  );
}

export default DashboardPage;
