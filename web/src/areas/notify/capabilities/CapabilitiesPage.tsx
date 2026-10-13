import { Badge, Button, Card, Table, TextField } from '@radix-ui/themes';
import {
  ChevronRight,
  CircleCheck,
  CircleHelp,
  ExternalLink,
  Link2,
  Palette,
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
import {
  CheckList,
  EmptyState,
  ErrorState,
  FilterLayout,
  FilterSection,
  Page,
  ResetFiltersButton,
} from '../../../ui';
import styles from '../Notify.module.css';
import { CAPABILITIES, type Capability, CREATE, categoryLabel, LIST_PAGE, stateLabel } from './model';

export const TYPE_ICONS: Record<string, typeof Puzzle> = {
  baseurl: Link2,
  'rapture.settings': SlidersHorizontal,
  'rapture.branding': Palette,
};

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
  const canCreate = useCan(CREATE);
  const capabilities = useApi<Capability[]>(CAPABILITIES);
  const [query, setQuery] = useState('');
  const [states, setStates] = useState<string[]>([]);
  const [categories, setCategories] = useState<string[]>([]);
  const [sheet, setSheet] = useState(false);
  const all = capabilities.data ?? [];
  const stateCounts = useMemo(() => count(all, stateOf), [all]);
  const categoryCounts = useMemo(() => count(all, categoryOf), [all]);
  const shown = all
    .filter((capability) => states.length === 0 || states.includes(stateOf(capability)))
    .filter((capability) => categories.length === 0 || categories.includes(categoryOf(capability)))
    .filter((capability) => {
      const needle = query.trim().toLowerCase();
      return (
        needle === '' ||
        [capability.typeName, capability.description ?? '', capability.notes].some((text) =>
          text.toLowerCase().includes(needle),
        )
      );
    })
    .sort((left, right) => left.typeName.localeCompare(right.typeName));
  const reset = () => {
    setStates([]);
    setCategories([]);
  };
  const filters = (
    <div className={styles.filters}>
      <ResetFiltersButton onClick={reset} />
      <FilterSection title={t('notify.capabilities.state')}>
        <CheckList
          options={[...stateCounts.entries()].map(([value, number]) => ({
            value,
            count: number,
            label:
              value === 'active'
                ? t('notify.capabilities.states.active')
                : t('notify.capabilities.states.disabled'),
          }))}
          selected={states}
          onChange={setStates}
        />
      </FilterSection>
      <FilterSection title={t('notify.capabilities.category')}>
        <CheckList
          options={[...categoryCounts.entries()]
            .map(([value, number]) => ({ value, count: number, label: categoryLabel(t, value) }))
            .sort((left, right) => left.label.localeCompare(right.label))}
          selected={categories}
          onChange={setCategories}
        />
      </FilterSection>
      <p className={styles.filtersNote}>{t('notify.capabilities.filtersNote')}</p>
    </div>
  );
  return (
    <Page
      title={t('notify.capabilities.title')}
      description={t('notify.capabilities.description')}
      breadcrumbs={[
        { label: t('notify.crumbs.settings'), to: '/settings' },
        { label: t('notify.capabilities.title') },
      ]}
      actions={
        canCreate ? (
          <Button asChild>
            <Link to={`${LIST_PAGE}/create`}>
              <Plus size={16} aria-hidden="true" />
              {t('notify.capabilities.create')}
            </Link>
          </Button>
        ) : null
      }
    >
      <FilterLayout
        label={t('notify.capabilities.filters')}
        contentLabel={t('notify.capabilities.listLabel')}
        width={260}
        gap={16}
        filters={filters}
        sheetOpen={sheet}
        onSheetOpenChange={setSheet}
        onResetAll={() => {
          reset();
          setQuery('');
        }}
      >
        <div className={styles.listHead}>
          <TextField.Root
            className={styles.search}
            value={query}
            placeholder={t('notify.capabilities.search')}
            aria-label={t('notify.capabilities.search')}
            onChange={(event) => setQuery(event.target.value)}
          >
            <TextField.Slot>
              <Search size={14} aria-hidden="true" />
            </TextField.Slot>
          </TextField.Root>
          <span className={styles.muted}>
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
          <Card className={styles.card}>
            <div className={styles.scroll}>
              <Table.Root className={styles.table} aria-label={t('notify.capabilities.listLabel')}>
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeaderCell>{t('notify.capabilities.columns.type')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{t('notify.capabilities.state')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{t('notify.capabilities.category')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>
                      {t('notify.capabilities.columns.description')}
                    </Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{t('notify.capabilities.columns.notes')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell />
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {shown.map((capability) => {
                    const Icon = TYPE_ICONS[capability.type] ?? Puzzle;
                    const to = `${LIST_PAGE}/${encodeURIComponent(capability.id)}`;
                    return (
                      <Table.Row
                        key={capability.id}
                        align="center"
                        className={styles.clickable}
                        onClick={() => navigate(to)}
                      >
                        <Table.RowHeaderCell>
                          <Link
                            to={to}
                            className={styles.typeLink}
                            onClick={(event) => event.stopPropagation()}
                          >
                            <Icon size={16} aria-hidden="true" className={styles.typeIcon} />
                            {capability.typeName}
                          </Link>
                        </Table.RowHeaderCell>
                        <Table.Cell>
                          <span className={styles.state}>
                            {capability.active && (
                              <CircleCheck size={16} color="var(--green-11)" aria-hidden="true" />
                            )}
                            <Badge color={capability.active ? 'green' : 'gray'}>
                              {stateLabel(t, capability)}
                            </Badge>
                          </span>
                        </Table.Cell>
                        <Table.Cell>
                          <Badge variant="outline" color="iris">
                            {categoryLabel(t, capability.tags.Category)}
                          </Badge>
                        </Table.Cell>
                        <Table.Cell className={styles.ellipsis}>
                          {capability.description ??
                            (capability.active ? t('notify.capabilities.states.active') : '')}
                        </Table.Cell>
                        <Table.Cell>{capability.notes}</Table.Cell>
                        <Table.Cell>
                          <ChevronRight size={16} aria-hidden="true" className={styles.muted} />
                        </Table.Cell>
                      </Table.Row>
                    );
                  })}
                </Table.Body>
              </Table.Root>
            </div>
          </Card>
        )}
        <Card className={styles.about}>
          <p className={styles.aboutTitle}>
            <CircleHelp size={14} aria-hidden="true" />
            {t('notify.capabilities.aboutTitle')}
          </p>
          <p className={styles.muted}>{t('notify.capabilities.aboutText')}</p>
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
      </FilterLayout>
    </Page>
  );
}
