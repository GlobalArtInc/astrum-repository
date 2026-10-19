import { Button, Spinner } from '@radix-ui/themes';
import {
  Ban,
  Check,
  ChevronDown,
  ChevronsUpDown,
  ChevronUp,
  CircleHelp,
  ExternalLink,
  Eye,
  Pencil,
  Plus,
  Search,
} from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';
import { useApi } from '../../api';
import { useT } from '../../i18n';
import { ErrorState, Page, Tooltip } from '../../ui';
import { useCollator } from '../security/shared/AdminList';
import { MODES } from './MatcherEditor';
import {
  CREATE_PAGE,
  DOCS,
  type Mode,
  matchesFilter,
  PREVIEW_PAGE,
  type RoutingRuleDetails,
  ruleHref,
  UI_RULES,
} from './model';
import styles from './Routing.module.css';

type Column = 'name' | 'description' | 'mode' | 'repositories';

export function ModeLabel({ mode }: { mode: Mode }) {
  const t = useT();
  return (
    <span className={`${styles.mode} ${mode === 'BLOCK' ? styles.modeBlock : styles.modeAllow}`}>
      {mode === 'BLOCK' ? <Ban size={14} aria-hidden="true" /> : <Check size={14} aria-hidden="true" />}
      {t(MODES[mode])}
    </span>
  );
}

export default function RoutingRulesPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const [params, setParams] = useSearchParams();
  const filter = params.get('q') ?? '';
  const rules = useApi<RoutingRuleDetails[]>(`${UI_RULES}?includeRepositoryNames=true`);
  const [sort, setSort] = useState<{ column: Column; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const rows = useMemo(() => {
    const value = (rule: RoutingRuleDetails): string | number => {
      switch (sort.column) {
        case 'description':
          return rule.description ?? '';
        case 'mode':
          return rule.mode;
        case 'repositories':
          return rule.assignedRepositoryCount;
        default:
          return rule.name;
      }
    };
    const sign = sort.direction === 'asc' ? 1 : -1;
    return (rules.data ?? [])
      .filter((rule) => matchesFilter(rule, filter))
      .sort((left, right) => {
        const a = value(left);
        const b = value(right);
        return (
          sign *
          (typeof a === 'number' && typeof b === 'number' ? a - b : collator.compare(String(a), String(b)))
        );
      });
  }, [rules.data, filter, sort, collator]);
  const search = (value: string) =>
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        if (value) {
          next.set('q', value);
        } else {
          next.delete('q');
        }
        return next;
      },
      { replace: true },
    );
  const header = (column: Column, label: string, className?: string) => {
    const sorted = sort.column === column ? sort.direction : undefined;
    return (
      <th
        className={className}
        aria-sort={sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : undefined}
      >
        <button
          type="button"
          className={styles.sort}
          onClick={() =>
            setSort(
              sort.column === column
                ? { column, direction: sort.direction === 'asc' ? 'desc' : 'asc' }
                : { column, direction: 'asc' },
            )
          }
        >
          {label}
          {sorted === 'asc' && <ChevronUp size={14} aria-hidden="true" className={styles.sortIcon} />}
          {sorted === 'desc' && <ChevronDown size={14} aria-hidden="true" className={styles.sortIcon} />}
          {!sorted && <ChevronsUpDown size={14} aria-hidden="true" className={styles.sortIcon} />}
        </button>
      </th>
    );
  };
  const all = rules.data ?? [];
  return (
    <Page
      className={styles.page}
      title={t('routing.list.title')}
      description={t('routing.list.intro')}
      breadcrumbs={[
        { label: t('routing.crumbs.settings'), to: '/settings' },
        { label: t('routing.list.title') },
      ]}
      actions={
        <div className={styles.actions}>
          <Button
            asChild
            size="3"
            variant="outline"
            color="gray"
            className={`${styles.bigButton} ${styles.outlineButton}`}
          >
            <Link to={PREVIEW_PAGE}>
              <Eye size={16} aria-hidden="true" />
              {t('routing.list.preview')}
            </Link>
          </Button>
          <Button asChild size="3" className={styles.bigButton}>
            <Link to={CREATE_PAGE}>
              <Plus size={16} aria-hidden="true" />
              {t('routing.list.create')}
            </Link>
          </Button>
        </div>
      }
    >
      <div className={styles.stack}>
        <div className={styles.filter}>
          <Search size={16} aria-hidden="true" className={styles.filterIcon} />
          <input
            type="text"
            className={styles.filterInput}
            value={filter}
            placeholder={t('routing.list.filter')}
            aria-label={t('routing.list.filter')}
            onChange={(event) => search(event.target.value)}
          />
        </div>
        {rules.error ? (
          <ErrorState error={rules.error} onRetry={() => void rules.refetch()} />
        ) : (
          <div className={styles.listCard}>
            <div className={styles.tableScroll}>
              <table className={styles.table} aria-label={t('routing.list.title')}>
                <thead>
                  <tr>
                    {header('name', t('routing.list.name'))}
                    {header('description', t('routing.list.description'))}
                    {header('mode', t('routing.list.mode'), styles.modeColumn)}
                    {header('repositories', t('routing.list.repositories'), styles.countColumn)}
                    <th className={styles.editColumn}>
                      <span className="visually-hidden">{t('routing.list.edit', { name: '' })}</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {rows.map((rule) => (
                    <tr
                      key={rule.name}
                      className={styles.row}
                      tabIndex={0}
                      aria-label={t('routing.list.open', { name: rule.name })}
                      onClick={() => navigate(ruleHref(rule.name))}
                      onKeyDown={(event) => {
                        if (event.key === 'Enter' && event.target === event.currentTarget) {
                          navigate(ruleHref(rule.name));
                        }
                      }}
                    >
                      <td className={styles.nameCell}>{rule.name}</td>
                      <td className={styles.descriptionCell}>{rule.description || '—'}</td>
                      <td>
                        <ModeLabel mode={rule.mode} />
                      </td>
                      <td className={styles.countCell}>{rule.assignedRepositoryCount}</td>
                      <td className={styles.editCell}>
                        <Tooltip content={t('routing.list.edit', { name: rule.name })}>
                          <Link
                            className={styles.edit}
                            to={ruleHref(rule.name)}
                            aria-label={t('routing.list.edit', { name: rule.name })}
                            onClick={(event) => event.stopPropagation()}
                          >
                            <Pencil size={16} aria-hidden="true" />
                          </Link>
                        </Tooltip>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            {rules.isPending && (
              <div className={styles.empty} aria-busy="true">
                <Spinner size="3" />
              </div>
            )}
            {!rules.isPending && rows.length === 0 && (
              <div className={styles.empty}>
                <span className={styles.emptyTitle}>
                  {all.length === 0 ? t('routing.list.empty') : t('routing.list.noMatches')}
                </span>
                <span>
                  {all.length === 0 ? t('routing.list.emptyText') : t('routing.list.noMatchesText')}
                </span>
              </div>
            )}
            <section className={styles.what} aria-labelledby="routing-what">
              <h2 id="routing-what" className={styles.whatTitle}>
                <CircleHelp size={14} aria-hidden="true" />
                {t('routing.list.whatTitle')}
              </h2>
              <p className={styles.whatText}>{t('routing.list.whatText')}</p>
              <a className={styles.link} href={DOCS} target="_blank" rel="noreferrer">
                {t('routing.list.docs')}
                <ExternalLink size={12} aria-hidden="true" />
              </a>
            </section>
          </div>
        )}
      </div>
    </Page>
  );
}
