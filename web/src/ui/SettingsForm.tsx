import type { FormEvent, ReactNode } from 'react';
import { useId } from 'react';
import styles from './SettingsForm.module.css';

export const settingsButton = styles.large;

export interface SettingsFormProps {
  label: string;
  start?: ReactNode;
  end?: ReactNode;
  unsaved?: string | null;
  progress?: { step: number; steps: number; label?: string };
  wide?: boolean;
  onSubmit?(): void;
  children: ReactNode;
}

export function SettingsForm({
  label,
  start,
  end,
  unsaved,
  progress,
  wide,
  onSubmit,
  children,
}: SettingsFormProps) {
  const classes = [styles.form, wide ? styles.wide : '', progress ? styles.withProgress : '']
    .filter(Boolean)
    .join(' ');
  const submit = (event: FormEvent) => {
    event.preventDefault();
    onSubmit?.();
  };
  return (
    <form className={classes} aria-label={label} noValidate onSubmit={submit}>
      {(start || end || unsaved || progress) && (
        <div className={styles.top}>
          <div className={styles.bar}>
            {start && <div className={styles.barStart}>{start}</div>}
            <div className={styles.barEnd}>
              {unsaved && <span className={styles.unsaved}>{unsaved}</span>}
              {end}
            </div>
          </div>
          {progress && (
            <div
              className={styles.progress}
              role="progressbar"
              aria-label={progress.label}
              aria-valuemin={1}
              aria-valuemax={progress.steps}
              aria-valuenow={progress.step}
            >
              <span style={{ width: `${(progress.step / progress.steps) * 100}%` }} />
              {Array.from({ length: progress.steps - 1 }, (_, index) => index + 1).map((position) => (
                <i
                  key={position}
                  className={styles.divider}
                  style={{ left: `${(position / progress.steps) * 100}%` }}
                />
              ))}
            </div>
          )}
        </div>
      )}
      <div className={styles.inner}>{children}</div>
    </form>
  );
}

export function SettingsSection({
  title,
  description,
  icon,
  children,
}: {
  title: string;
  description?: ReactNode;
  icon?: ReactNode;
  children: ReactNode;
}) {
  const id = useId();
  return (
    <section className={styles.section} aria-labelledby={id}>
      <div className={styles.sectionHead}>
        <h2 id={id} className={styles.sectionTitle}>
          {icon}
          {title}
        </h2>
        {description && <p className={styles.sectionText}>{description}</p>}
      </div>
      {children}
    </section>
  );
}

export function SettingsFields({ children }: { children: ReactNode }) {
  return <div className={styles.fields}>{children}</div>;
}
