import { useId } from 'react';
import styles from './Shell.module.css';

const GRADIENT_FROM = '#8B5CF6';
const GRADIENT_TO = '#3B82F6';

export function Mark({ size = 28 }: { size?: number }) {
  const id = useId();
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 61 61"
      fill="none"
      aria-hidden="true"
      focusable="false"
      className={styles.mark}
    >
      <defs>
        <linearGradient id={id} x1="3" y1="8" x2="53" y2="58" gradientUnits="userSpaceOnUse">
          <stop stopColor={GRADIENT_FROM} />
          <stop offset="1" stopColor={GRADIENT_TO} />
        </linearGradient>
      </defs>
      <path
        d="M28 8C30.3 23.57 37.42 30.7 53 33C37.42 35.3 30.3 42.42 28 58C25.7 42.42 18.57 35.3 3 33C18.57 30.7 25.7 23.57 28 8Z"
        fill={`url(#${id})`}
      />
      <path
        d="M50 3.5C50.75 8.53 52.48 10.25 57.5 11C52.48 11.75 50.75 13.47 50 18.5C49.25 13.47 47.52 11.75 42.5 11C47.52 10.25 49.25 8.53 50 3.5Z"
        fill="var(--astrum-logo-accent)"
      />
    </svg>
  );
}

export const Sparkle = Mark;

export function Wordmark({ height = 28 }: { height?: number }) {
  const id = useId();
  return (
    <svg
      width={(height * 300) / 37}
      height={height}
      viewBox="0 0 300 37"
      fill="none"
      aria-hidden="true"
      focusable="false"
      className={styles.wordmark}
    >
      <defs>
        <linearGradient id={id} x1="1" y1="3" x2="34" y2="36" gradientUnits="userSpaceOnUse">
          <stop stopColor={GRADIENT_FROM} />
          <stop offset="1" stopColor={GRADIENT_TO} />
        </linearGradient>
      </defs>
      <path
        d="M17.5 3C19.02 13.28 23.72 17.98 34 19.5C23.72 21.02 19.02 25.72 17.5 36C15.98 25.72 11.28 21.02 1 19.5C11.28 17.98 15.98 13.28 17.5 3Z"
        fill={`url(#${id})`}
      />
      <path
        d="M33.5 1C33.95 4.01 34.98 5.05 38 5.5C34.98 5.95 33.95 6.99 33.5 10C33.05 6.99 32.02 5.95 29 5.5C32.02 5.05 33.05 4.01 33.5 1Z"
        fill="var(--astrum-logo-accent)"
      />
      <text
        x="45"
        y="27.5"
        fontFamily="Inter, 'Helvetica Neue', Helvetica, Arial, sans-serif"
        fontSize="27"
        fontWeight="700"
        letterSpacing="-0.4"
        fill="var(--astrum-logo-text)"
      >
        astrum{' '}
        <tspan fontWeight="600" fill="var(--astrum-logo-accent)">
          repository
        </tspan>
      </text>
    </svg>
  );
}

export function Logo({ compact = false }: { compact?: boolean }) {
  return (
    <span className={styles.logo} data-compact={compact || undefined} aria-hidden="true">
      <span className={styles.logoFull}>
        <Wordmark height={compact ? 24 : 28} />
      </span>
      <span className={styles.logoMark}>
        <Mark size={28} />
      </span>
    </span>
  );
}
