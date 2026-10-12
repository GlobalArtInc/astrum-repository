import styles from './Shell.module.css';

export function Sparkle({ size = 28 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" aria-hidden="true" focusable="false">
      <path
        fill="var(--astrum-brand-mark)"
        d="M14 4c.9 6.3 3.7 9.1 10 10-6.3.9-9.1 3.7-10 10-.9-6.3-3.7-9.1-10-10 6.3-.9 9.1-3.7 10-10Z"
      />
      <path
        fill="var(--astrum-brand-mark)"
        opacity="0.85"
        d="M25 2c.4 2.5 1.5 3.6 4 4-2.5.4-3.6 1.5-4 4-.4-2.5-1.5-3.6-4-4 2.5-.4 3.6-1.5 4-4Z"
      />
    </svg>
  );
}

export function Logo() {
  return (
    <span className={styles.logo} aria-hidden="true">
      <Sparkle />
      <span className={styles.wordmark}>
        astrum <span className={styles.wordmarkAccent}>repository</span>
      </span>
    </span>
  );
}
