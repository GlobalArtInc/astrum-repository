import { Badge, Button, Card, Table } from '@radix-ui/themes';
import {
  ChevronRight,
  CircleCheckBig,
  CircleHelp,
  ExternalLink,
  Pencil,
  Plus,
  Puzzle,
  Search,
  SlidersHorizontal,
} from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { useApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { EmptyState, ErrorState, Page, Tooltip } from '../../../ui';
import { PHONE, useMediaQuery } from '../../../ui/hooks';
import { settingsButton } from '../../../ui/SettingsForm';
import styles from './Capabilities.module.css';
import { type FilterGroup, FilterSheet, FilterSidebar } from './CapabilityFilters';
import {
  CAPABILITIES,
  type Capability,
  CREATE,
  categoryIcon,
  categoryLabel,
  LIST_PAGE,
  stateLabel,
} from './model';

const stateOf = (capability: Capability) => (capability.active ? 'active' : 'disabled');
const categoryOf = (capability: Capability) => capability.tags.Category ?? '';

function count<T>(items: T[], key: (item: T) => string): Map<string, number> {
  const counts = new Map<string, number>();
  for (const item of items) {
    counts.set(key(item), (counts.get(key(item)) ?? 0) + 1);
  }
  return counts;
}

export default function CapabilitiesPage() {
  const t = useT();
  const navigate = useNavigate();
  const phone = useMediaQuery(PHONE);
  const canCreate = useCan(CREATE);
  const capabilities = useApi<Capability[]>(CAPABILITIES);
  const [query, setQuery] = useState('');
  const [states, setStates] = useState<string[]>([]);
  const [categories, setCategories] = useState<string[]>([]);
  const [sheet, setSheet] = useState(false);
  const all = capabilities.data ?? [];
  const stateCounts = useMemo(() => count(all, stateOf), [all]);
  const categoryCounts = useMemo(() => count(all, categoryOf), [all]);
  const needle = query.trim().toLowerCase();
  const shown = all
    .filter((capability) => states.length === 0 || states.includes(stateOf(capability)))
    .filter((capability) => categories.length === 0 || categories.includes(categoryOf(capability)))
    .filter(
      (capability) =>
        needle === '' ||
        [capability.typeName, capability.description ?? '', capability.notes].some((text) =>
          text.toLowerCase().includes(needle),
        ),
    )
    .sort((left, right) => left.typeName.localeCompare(right.typeName));
  const groups: FilterGroup[] = [
    {
      id: 'state',
      title: t('notify.capabilities.state'),
      options: [...stateCounts.entries()].map(([value, number]) => ({
        value,
        count: number,
        label:
          value === 'active'
            ? t('notify.capabilities.states.active')
            : t('notify.capabilities.states.disabled'),
      })),
      selected: states,
      onChange: setStates,
    },
    {
      id: 'category',
      title: t('notify.capabilities.category'),
      options: [...categoryCounts.entries()]
        .map(([value, number]) => ({ value, count: number, label: categoryLabel(t, value) }))
        .sort((left, right) => left.label.localeCompare(right.label)),
      selected: categories,
      onChange: setCategories,
    },
  ];
  return (
    <Page
      title={t('notify.capabilities.title')}
      description={t('notify.capabilities.description')}
      breadcrumbs={[
        { label: t('notify.crumbs.settings'), to: '/settings' },
        { label: t('notify.capabilities.title') },
      ]}
      className={styles.page}
      actions={
        canCreate ? (
          <Button asChild size="3" className={settingsButton}>
            <Link to={`${LIST_PAGE}/create`}>
              <Plus size={16} aria-hidden="true" />
              {t('notify.capabilities.create')}
            </Link>
          </Button>
        ) : null
      }
    >
      <div className={styles.layout}>
        {!phone && <FilterSidebar groups={groups} label={t('notify.capabilities.filters')} />}
        <div className={styles.main}>
          {phone && (
            <Button
              type="button"
              size="3"
              variant="outline"
              color="gray"
              className={styles.filtersButton}
              onClick={() => setSheet(true)}
            >
              <SlidersHorizontal size={16} aria-hidden="true" />
              {t('notify.capabilities.filters')}
            </Button>
          )}
          <div className={styles.listHead}>
            <div className={styles.search}>
              <Search size={16} aria-hidden="true" />
              <input
                type="search"
                value={query}
                placeholder={t('notify.capabilities.search')}
                aria-label={t('notify.capabilities.search')}
                onChange={(event) => setQuery(event.target.value)}
              />
            </div>
            <span className={styles.shown}>
              {t('notify.capabilities.shown', { shown: shown.length, total: all.length })}
            </span>
          </div>
          {capabilities.error ? (
            <ErrorState error={capabilities.error} onRetry={() => void capabilities.refetch()} />
          ) : shown.length === 0 && !capabilities.isPending ? (
            <Card>
              <EmptyState
                icon={Puzzle}
                title={all.length === 0 ? t('notify.capabilities.none') : t('notify.capabilities.noMatch')}
              />
            </Card>
          ) : (
            <Table.Root
              variant="surface"
              className={styles.table}
              aria-label={t('notify.capabilities.listLabel')}
            >
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell>{t('notify.capabilities.columns.type')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('notify.capabilities.state')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('notify.capabilities.category')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>
                    {t('notify.capabilities.columns.description')}
                  </Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('notify.capabilities.columns.notes')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell className={styles.iconColumn} />
                  <Table.ColumnHeaderCell className={styles.iconColumn} />
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {shown.map((capability) => {
                  const Icon = categoryIcon(capability.tags.Category);
                  const to = `${LIST_PAGE}/${encodeURIComponent(capability.id)}`;
                  const description =
                    capability.description ??
                    (capability.active ? t('notify.capabilities.states.active') : '');
                  return (
                    <Table.Row
                      key={capability.id}
                      align="center"
                      className={styles.row}
                      onClick={() => navigate(to)}
                    >
                      <Table.Cell className={styles.typeCell}>
                        <Link
                          to={to}
                          className={styles.typeLink}
                          onClick={(event) => event.stopPropagation()}
                        >
                          <Icon size={16} aria-hidden="true" className={styles.typeIcon} />
                          {capability.typeName}
                        </Link>
                      </Table.Cell>
                      <Table.Cell>
                        <span className={styles.state}>
                          {capability.active && (
                            <CircleCheckBig size={16} color="var(--green-11)" aria-hidden="true" />
                          )}
                          <Badge color={capability.active ? 'green' : 'gray'}>
                            {stateLabel(t, capability)}
                          </Badge>
                        </span>
                      </Table.Cell>
                      <Table.Cell>
                        <Badge variant="outline">{categoryLabel(t, capability.tags.Category)}</Badge>
                      </Table.Cell>
                      <Table.Cell className={styles.description}>
                        {description && (
                          <Tooltip content={description}>
                            <span>{description}</span>
                          </Tooltip>
                        )}
                      </Table.Cell>
                      <Table.Cell className={styles.notes}>{capability.notes}</Table.Cell>
                      <Table.Cell className={styles.iconColumn}>
                        <Pencil size={16} aria-hidden="true" className={styles.edit} />
                      </Table.Cell>
                      <Table.Cell className={styles.iconColumn}>
                        <ChevronRight size={16} aria-hidden="true" className={styles.arrow} />
                      </Table.Cell>
                    </Table.Row>
                  );
                })}
              </Table.Body>
            </Table.Root>
          )}
          <Card className={styles.about}>
            <p className={styles.aboutTitle}>
              <CircleHelp size={16} aria-hidden="true" />
              {t('notify.capabilities.aboutTitle')}
            </p>
            <p className={styles.aboutText}>{t('notify.capabilities.aboutText')}</p>
            <a
              className={styles.docs}
              href="https://links.globalart.dev/astrum/capabilities"
              target="_blank"
              rel="noreferrer"
            >
              {t('notify.capabilities.docs')}
              <ExternalLink size={12} aria-hidden="true" />
            </a>
          </Card>
        </div>
      </div>
      {phone && <FilterSheet groups={groups} open={sheet} onOpenChange={setSheet} />}
    </Page>
  );
}
