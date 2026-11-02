import type { ReactNode } from 'react';
import { formatNumber, useI18n, useT } from '../../../i18n';
import { CopyButton, DateTime, FileSize } from '../../../ui';
import type { AssetDetails } from '../types';
import styles from './AssetSummary.module.css';

export interface AssetSummaryProps {
  details: AssetDetails;
}

interface TileProps {
  label: string;
  children: ReactNode;
}

function Tile({ label, children }: TileProps) {
  return (
    <div className={styles.tile}>
      <dt className={styles.label}>{label}</dt>
      <dd className={styles.value}>{children}</dd>
    </div>
  );
}

function shortDigest(hex: string): string {
  return `${hex.slice(0, 10)}…${hex.slice(-4)}`;
}

export function AssetSummary({ details }: AssetSummaryProps) {
  const t = useT();
  const { language } = useI18n();
  const docker = details.format === 'docker';
  const image = details.image;
  const sha256 = details.checksum.sha256;
  const hex = docker && sha256 ? sha256 : null;
  const size =
    docker && typeof image?.size === 'number' ? (
      <>
        <FileSize bytes={image.size} />
        {typeof image.layers === 'number' && ` · ${t('browse.details.layers', { count: image.layers })}`}
      </>
    ) : (
      <FileSize bytes={details.fileSize} />
    );
  return (
    <section className={styles.summary} aria-label={t('browse.summary.label')}>
      <dl className={styles.grid}>
        <Tile label={docker ? t('browse.summary.pushed') : t('browse.summary.uploaded')}>
          <DateTime value={details.blobUpdated} />
        </Tile>
        <Tile label={t('browse.summary.size')}>{size}</Tile>
        {hex && (
          <Tile label={t('browse.summary.digest')}>
            <span className={styles.digestRow}>
              <span className={styles.digest} title={`sha256:${hex}`}>
                {shortDigest(hex)}
              </span>
              <CopyButton text={`sha256:${hex}`} label={t('browse.summary.copyDigest')} />
            </span>
          </Tile>
        )}
        <Tile label={t('browse.summary.pulls')}>{formatNumber(language, details.downloadCount)}</Tile>
      </dl>
    </section>
  );
}
