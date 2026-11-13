import { useT } from '../../../i18n';
import { FileSize } from '../../../ui';
import type { RepositoryUsage } from '../api';
import { usageLevel } from '../model/quota';
import { Count } from './bits';
import styles from './UsageBar.module.css';

export function UsageBar({ usage }: { usage: RepositoryUsage }) {
  const t = useT();
  const limit = usage.quota?.limitBytes ?? null;
  const percent = usage.percentUsed;
  const figures = (
    <dl className={styles.figures}>
      <div>
        <dt>{t('repositories.usage.stored')}</dt>
        <dd>
          <FileSize bytes={usage.bytes} />
        </dd>
      </div>
      <div>
        <dt>{t('repositories.usage.assets')}</dt>
        <dd>
          <Count value={usage.assets} />
        </dd>
      </div>
      {limit !== null && (
        <div>
          <dt>{t('repositories.usage.limit')}</dt>
          <dd>
            <FileSize bytes={limit} />
          </dd>
        </div>
      )}
    </dl>
  );
  if (usage.quota === null || limit === null || percent === null) {
    return (
      <div className={styles.usage}>
        {figures}
        <p className={styles.note}>{t('repositories.usage.noQuota')}</p>
      </div>
    );
  }
  const level = usageLevel(percent, usage.quota.warnPercent);
  const text = t('repositories.usage.percent', { percent: percent.toFixed(1) });
  return (
    <div className={styles.usage}>
      {figures}
      <div
        className={styles.track}
        role="progressbar"
        aria-label={t('repositories.usage.meter')}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.min(percent, 100)}
        aria-valuetext={text}
      >
        <div className={styles.fill} data-level={level} style={{ width: `${Math.min(percent, 100)}%` }} />
      </div>
      <p className={styles.note} data-level={level}>
        {text}
      </p>
      {level === 'warning' && (
        <p className={styles.note} data-level={level}>
          {t('repositories.usage.warning')}
        </p>
      )}
      {level === 'full' && (
        <p className={styles.note} data-level={level}>
          {t('repositories.usage.full')}
        </p>
      )}
    </div>
  );
}
