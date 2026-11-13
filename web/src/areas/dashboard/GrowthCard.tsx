import { Card } from '@radix-ui/themes';
import { Link } from 'react-router';
import { formatBytes, useI18n, useT } from '../../i18n';
import { editAddress, type UsageSeries } from '../repositories/api';
import styles from './Dashboard.module.css';
import { growthRanking } from './growth';

export function GrowthCard({ series }: { series: UsageSeries | undefined }) {
  const t = useT();
  const { language } = useI18n();
  if (!series) {
    return null;
  }
  const rows = growthRanking(series);
  return (
    <Card size="3" asChild className={styles.card}>
      <section aria-labelledby="dashboard-growth">
        <h2 id="dashboard-growth" className={styles.cardTitle}>
          {t('dashboard.growth.title')}
        </h2>
        {rows.length === 0 ? (
          <p className={styles.cardEmpty}>{t('dashboard.growth.empty')}</p>
        ) : (
          <ul className={styles.ranking}>
            {rows.map((row) => (
              <li key={row.name}>
                <span className={styles.rankingLine}>
                  <span className={styles.repositoryCell}>
                    <Link to={editAddress(row.name)} className={styles.name}>
                      {row.name}
                    </Link>
                  </span>
                  <span className={styles.count}>
                    {t('dashboard.growth.delta', { size: formatBytes(language, row.growth) })}
                  </span>
                </span>
                <span className={styles.caption}>
                  {t('dashboard.growth.range', {
                    from: formatBytes(language, row.first),
                    to: formatBytes(language, row.last),
                  })}
                </span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </Card>
  );
}
