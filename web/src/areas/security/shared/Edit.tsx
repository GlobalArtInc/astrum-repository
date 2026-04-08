import { Check } from 'lucide-react';
import { Fragment, type ReactNode } from 'react';
import styles from './Edit.module.css';

export function EditForm({
  id,
  onSubmit,
  bar,
  children,
  wide,
  tight,
}: {
  id: string;
  onSubmit(event: React.FormEvent<HTMLFormElement>): void;
  bar: ReactNode;
  children: ReactNode;
  wide?: boolean;
  tight?: boolean;
}) {
  return (
    <form
      id={id}
      className={tight ? `${styles.form} ${styles.tight}` : styles.form}
      onSubmit={onSubmit}
      noValidate
    >
      {bar}
      <div className={wide ? `${styles.body} ${styles.wide}` : styles.body}>{children}</div>
    </form>
  );
}

export function EditBar({
  start,
  end,
  progress,
}: {
  start?: ReactNode;
  end?: ReactNode;
  progress?: ReactNode;
}) {
  return (
    <div className={progress ? `${styles.bar} ${styles.wizardBar}` : styles.bar}>
      <div className={styles.barRow}>
        <div className={`${styles.group} ${styles.start}`}>{start}</div>
        <div className={styles.group}>{end}</div>
      </div>
      {progress}
    </div>
  );
}

export function Progress({ total, done, label }: { total: number; done: number; label: string }) {
  return (
    <div
      className={styles.progress}
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={total}
      aria-valuenow={done}
    >
      {Array.from({ length: total }, (_, index) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: segments are positions
        <Fragment key={index}>
          {index > 0 && <span className={styles.divider} aria-hidden="true" />}
          <span className={index < done ? `${styles.segment} ${styles.filled}` : styles.segment} />
        </Fragment>
      ))}
    </div>
  );
}

export function StepTitle({ children }: { children: ReactNode }) {
  return <h2 className={styles.stepTitle}>{children}</h2>;
}

export function FormSection({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <section className={styles.section}>
      {title && <h3 className={styles.sectionTitle}>{title}</h3>}
      {children}
    </section>
  );
}

export function ActionBoxes<V extends string>({
  label,
  help,
  required,
  options,
  value,
  onChange,
  disabled,
  error,
}: {
  label: string;
  help?: string;
  required?: boolean;
  options: { value: V; label: string }[];
  value: readonly V[];
  onChange(value: V[]): void;
  disabled?: boolean;
  error?: string;
}) {
  return (
    <div role="group" aria-label={label} className={styles.boxes}>
      <span className={styles.boxesLabel}>
        {label}
        {required && (
          <span className={styles.required} aria-hidden="true">
            *
          </span>
        )}
      </span>
      <div className={styles.boxRow}>
        {options.map((option) => {
          const checked = value.includes(option.value);
          return (
            <label key={option.value} className={styles.box} data-disabled={disabled || undefined}>
              <span className={checked ? `${styles.mark} ${styles.markOn}` : styles.mark}>
                <input
                  type="checkbox"
                  value={option.value}
                  checked={checked}
                  disabled={disabled}
                  onChange={(event) =>
                    onChange(
                      event.target.checked
                        ? [...value, option.value]
                        : value.filter((item) => item !== option.value),
                    )
                  }
                />
                {checked && <Check size={14} aria-hidden="true" />}
              </span>
              <span>{option.label}</span>
            </label>
          );
        })}
      </div>
      {help && <p className={styles.boxesHelp}>{help}</p>}
      {error && (
        <p className={styles.error} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}

export function ReadOnlyValue({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className={styles.readOnly}>
      <span className={styles.readOnlyLabel}>{label}</span>
      <span className={styles.readOnlyValue}>{value}</span>
    </div>
  );
}
