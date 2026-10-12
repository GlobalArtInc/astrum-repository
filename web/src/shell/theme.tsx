import { createContext, type ReactNode, useCallback, useContext, useMemo, useState } from 'react';
import { readStored, useMediaQuery, writeStored } from '../ui/hooks';

export type ThemePreference = 'dark' | 'light' | 'system';
export type Appearance = 'dark' | 'light';

export const THEME_KEY = 'astrum-theme-preference';
export const PREFERENCES: ThemePreference[] = ['dark', 'light', 'system'];

export function readPreference(): ThemePreference {
  const stored = readStored(THEME_KEY);
  return PREFERENCES.includes(stored as ThemePreference) ? (stored as ThemePreference) : 'dark';
}

export function resolveAppearance(preference: ThemePreference, prefersLight: boolean): Appearance {
  if (preference === 'system') {
    return prefersLight ? 'light' : 'dark';
  }
  return preference;
}

interface ThemeState {
  preference: ThemePreference;
  appearance: Appearance;
  setPreference(preference: ThemePreference): void;
}

const ThemeContext = createContext<ThemeState | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [preference, setCurrent] = useState<ThemePreference>(readPreference);
  const prefersLight = useMediaQuery('(prefers-color-scheme: light)');
  const setPreference = useCallback((next: ThemePreference) => {
    writeStored(THEME_KEY, next);
    setCurrent(next);
  }, []);
  const value = useMemo(
    () => ({ preference, appearance: resolveAppearance(preference, prefersLight), setPreference }),
    [preference, prefersLight, setPreference],
  );
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeState {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme needs a ThemeProvider');
  }
  return context;
}
