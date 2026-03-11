import { File } from 'lucide-react';
import styles from './FormatLogo.module.css';
import { FORMAT_LOGOS } from './formatLogos';

const DARK_LOGOS = new Set(['cargo', 'helm', 'nuget', 'p2']);

export function FormatLogo({ format }: { format: string }) {
  const logo = FORMAT_LOGOS[format];
  return (
    <span className={styles.tile} aria-hidden="true">
      <span className={styles.plate}>
        {logo ? (
          <svg
            aria-hidden="true"
            className={styles.logo}
            data-dark={DARK_LOGOS.has(format) ? '' : undefined}
            width="16.5"
            height="16.5"
            viewBox="0 0 24 24"
            fill={logo.color}
          >
            <path d={logo.path} />
          </svg>
        ) : (
          <File size={16} className={styles.fallback} />
        )}
      </span>
    </span>
  );
}
