import { Button, TextField } from '@radix-ui/themes';
import { CircleCheck, CircleX, Clock, Download, RefreshCw, Search } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { useApi } from '../../api';
import { useI18n } from '../../i18n';
import { formatDateTime } from '../../i18n/format';
import { ErrorState, Page, useToast } from '../../ui';
import { HelpCard } from '../security/shared/AdminList';
import { CHECKS, DOCS, HEALTH_PAGE, type HealthCheck, type HealthChecks, saveJson } from './api';
import styles from './Support.module.css';

export function orderChecks(checks: HealthChecks): Array<[string, HealthCheck]> {
  return Object.entries(checks).sort(
    ([leftName, left], [rightName, right]) =>
      Number(left.healthy) - Number(right.healthy) || leftName.localeCompare(rightName),
  );
}

function StatusIcon({ healthy, size = 16 }: { healthy: boolean; size?: number }) {
  return healthy ? (
    <CircleCheck size={size} className={styles.ok} aria-hidden="true" />
  ) : (
    <CircleX size={size} className={styles.bad} aria-hidden="true" />
  );
}

export default function MetricHealthPage() {
  const { t, language } = useI18n();
  const toast = useToast();
  const navigate = useNavigate();
  const { check: chosen } = useParams();
  const checks = useApi<HealthChecks>(CHECKS);
  const [filter, setFilter] = useState('');
  const ordered = useMemo(() => orderChecks(checks.data ?? {}), [checks.data]);
  const shown = ordered.filter(([name]) => name.toLowerCase().includes(filter.trim().toLowerCase()));
  const selected = ordered.find(([name]) => name === chosen) ?? ordered[0];
  const healthy = ordered.filter(([, check]) => check.healthy).length;
  const crumbs = [
    { label: t('support.crumbs.settings'), to: '/settings' },
    { label: t('support.health.title'), to: selected ? HEALTH_PAGE : undefined },
    ...(selected ? [{ label: selected[0] }] : []),
  ];
  return (
    <Page
      gutter="large"
      className={`${styles.wide} ${styles.healthPage}`}
      title={t('support.health.title')}
      description={t('support.health.intro')}
      breadcrumbs={crumbs}
      actions={
        <div className={styles.actions}>
          <Button
            variant="ghost"
            size="3"
            className={styles.ghostAction}
            onClick={async () => {
              await checks.refetch();
              toast.success(t('support.health.refreshed'));
            }}
          >
            <RefreshCw size={16} aria-hidden="true" />
            {t('support.common.refresh')}
          </Button>
          <Button
            variant="ghost"
            size="3"
            className={styles.ghostAction}
            disabled={!checks.data}
            onClick={() => {
              saveJson('metric-health.json', checks.data);
              toast.success(t('support.health.downloaded'));
            }}
          >
            <Download size={16} aria-hidden="true" />
            {t('support.common.download')}
          </Button>
        </div>
      }
    >
      <div className={styles.stack}>
        {checks.error ? (
          <ErrorState error={checks.error} onRetry={() => void checks.refetch()} />
        ) : (
          <div className={styles.health}>
            <section className={styles.checkList} aria-label={t('support.health.title')}>
              <div className={styles.counters}>
                <span>
                  <StatusIcon healthy size={14} />
                  {t('support.health.healthyCount', { count: healthy })}
                </span>
                <span>
                  <StatusIcon healthy={false} size={14} />
                  {t('support.health.unhealthyCount', { count: ordered.length - healthy })}
                </span>
              </div>
              <div className={styles.checkFilter}>
                <TextField.Root
                  size="1"
                  value={filter}
                  placeholder={t('support.health.filter')}
                  aria-label={t('support.health.filter')}
                  onChange={(event) => setFilter(event.target.value)}
                  className={styles.checkFilterInput}
                >
                  <TextField.Slot>
                    <Search size={14} aria-hidden="true" />
                  </TextField.Slot>
                </TextField.Root>
              </div>
              <ul className={styles.checks}>
                {shown.map(([name, check]) => (
                  <li key={name}>
                    <button
                      type="button"
                      className={styles.checkItem}
                      aria-current={selected?.[0] === name || undefined}
                      onClick={() => navigate(`${HEALTH_PAGE}/${encodeURIComponent(name)}`)}
                    >
                      <StatusIcon healthy={check.healthy} />
                      <span>{name}</span>
                    </button>
                  </li>
                ))}
                {!checks.isPending && shown.length === 0 && (
                  <li className={styles.muted}>
                    {ordered.length === 0 ? t('support.health.empty') : t('support.health.noMatches')}
                  </li>
                )}
              </ul>
            </section>
            <section className={styles.checkDetail} aria-live="polite">
              {selected ? (
                <>
                  <div className={styles.detailHeader}>
                    <StatusIcon healthy={selected[1].healthy} size={22} />
                    <h2 className={styles.detailTitle}>{selected[0]}</h2>
                    <span className={selected[1].healthy ? styles.okBadge : styles.badBadge}>
                      {selected[1].healthy ? t('support.health.healthy') : t('support.health.unhealthy')}
                    </span>
                  </div>
                  <div className={styles.detailBody}>
                    {selected[1].message && (
                      <>
                        <h3 className={styles.detailLabel}>{t('support.health.message')}</h3>
                        <p className={styles.message}>{selected[1].message}</p>
                      </>
                    )}
                    {selected[1].details !== null && selected[1].details !== undefined && (
                      <>
                        <h3 className={styles.detailLabel}>{t('support.health.details')}</h3>
                        <pre className={styles.message}>{JSON.stringify(selected[1].details, null, 2)}</pre>
                      </>
                    )}
                    <p className={styles.duration}>
                      <Clock size={14} aria-hidden="true" />
                      {t('support.health.duration', { ms: selected[1].duration })}
                    </p>
                    <p className={styles.lastChecked}>
                      {t('support.health.lastChecked', {
                        time: formatDateTime(language, selected[1].timestamp),
                      })}
                    </p>
                  </div>
                </>
              ) : (
                <p className={styles.muted}>{t('support.health.select')}</p>
              )}
            </section>
          </div>
        )}
        <HelpCard title={t('support.health.helpTitle')} link={DOCS} linkLabel={t('support.common.docsMore')}>
          {t('support.health.helpText')}
        </HelpCard>
      </div>
    </Page>
  );
}
