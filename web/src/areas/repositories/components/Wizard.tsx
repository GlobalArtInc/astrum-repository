import { Button } from '@radix-ui/themes';
import { ArrowLeft, File } from 'lucide-react';
import type { ReactNode } from 'react';
import { useT } from '../../../i18n';
import { FORMAT_LOGOS } from '../../../ui/formatLogos';
import styles from './Wizard.module.css';

const DARK_LOGOS = new Set(['cargo', 'helm', 'nuget', 'p2']);

export function Logo({ format, size = 24 }: { format: string; size?: number }) {
  const logo = FORMAT_LOGOS[format];
  return logo ? (
    <svg
      aria-hidden="true"
      className={styles.logo}
      data-dark={DARK_LOGOS.has(format) ? '' : undefined}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill={logo.color}
    >
      <path d={logo.path} />
    </svg>
  ) : (
    <File size={size} aria-hidden="true" className={styles.fallback} />
  );
}

export function WizardBar({
  step,
  steps = 5,
  onCancel,
  onBack,
  next,
}: {
  step: number;
  steps?: number;
  onCancel(): void;
  onBack?(): void;
  next?: ReactNode;
}) {
  const t = useT();
  return (
    <div className={styles.bar}>
      <div className={styles.buttons}>
        <Button variant="outline" onClick={onCancel}>
          {t('repositories.create.cancel')}
        </Button>
        <span className={styles.right}>
          {onBack && (
            <Button variant="soft" onClick={onBack}>
              <ArrowLeft size={16} aria-hidden="true" />
              {t('repositories.create.back')}
            </Button>
          )}
          {next}
        </span>
      </div>
      <div
        className={styles.progress}
        role="progressbar"
        aria-valuemin={1}
        aria-valuemax={steps}
        aria-valuenow={step}
        aria-label={t('repositories.create.progress', { step, steps })}
      >
        {Array.from({ length: steps }, (_, index) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: the segments never move
          <span key={index} className={styles.segment} data-done={index < step || undefined} />
        ))}
      </div>
    </div>
  );
}

export function StepTitle({ title, count }: { title: string; count?: number }) {
  return (
    <h2 className={styles.stepTitle}>
      {title}
      {count !== undefined && <span className={styles.stepCount}>{count}</span>}
    </h2>
  );
}
