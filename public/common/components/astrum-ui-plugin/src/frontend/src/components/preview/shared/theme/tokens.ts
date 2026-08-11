

/**
 * Design tokens as typed JS constants.
 *
 * Maps semantic names to CSS variable references. Use in inline styles
 * or dynamic style calculations. For static styles, use the CSS variables
 * directly in SCSS.
 *
 * @example
 * import { colors, spacing } from '@nosc/theme/tokens';
 * <Box style={{ color: colors.textPrimary, padding: spacing.md }}>
 */

export const colors = {
  textPrimary: 'var(--color-text, var(--gray-12))',
  textSecondary: 'var(--color-text-secondary, var(--gray-11))',
  textMuted: 'var(--gray-9)',
  background: 'var(--color-background, var(--gray-1))',
  surface: 'var(--color-surface, var(--gray-2))',
  border: 'var(--color-border, var(--gray-6))',
  accent: 'var(--accent-9)',
  accentSubtle: 'var(--accent-3)',

  success: 'var(--green-9)',
  successSubtle: 'var(--green-3)',
  warning: 'var(--amber-9)',
  warningSubtle: 'var(--amber-3)',
  error: 'var(--red-9)',
  errorSubtle: 'var(--red-3)',
  info: 'var(--blue-9)',
  infoSubtle: 'var(--blue-3)',
} as const;

export const spacing = {
  xs: 'var(--space-1)',
  sm: 'var(--space-2)',
  md: 'var(--space-3)',
  lg: 'var(--space-4)',
  xl: 'var(--space-5)',
  xxl: 'var(--space-6)',
} as const;

export const radii = {
  sm: 'var(--radius-1)',
  md: 'var(--radius-2)',
  lg: 'var(--radius-3)',
  full: '9999px',
} as const;

export const fontSizes = {
  xs: '11px',
  sm: '13px',
  md: '14px',
  lg: '16px',
  xl: '20px',
  xxl: '24px',
} as const;

export type ColorToken = keyof typeof colors;
export type SpacingToken = keyof typeof spacing;
export type RadiusToken = keyof typeof radii;
export type FontSizeToken = keyof typeof fontSizes;
