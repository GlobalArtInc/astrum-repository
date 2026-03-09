import { createContext, type ReactNode, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { dictionaries } from './dictionaries';
import { detectLanguage, rememberLanguage } from './language';
import { type Language, splitRich, translate, type Vars } from './translate';

export interface Translate {
  (key: string, vars?: Vars): string;
  rich(key: string, slots: Record<string, ReactNode>): ReactNode[];
}

interface I18n {
  language: Language;
  setLanguage(language: Language): void;
  t: Translate;
}

const I18nContext = createContext<I18n | null>(null);

export function makeTranslate(language: Language): Translate {
  const t = ((key: string, vars?: Vars) => translate(dictionaries, language, key, vars)) as Translate;
  t.rich = (key, slots) => {
    const text = translate(dictionaries, language, key);
    return splitRich(text).map((part) =>
      'slot' in part ? (slots[part.slot] ?? `{${part.slot}}`) : part.text,
    );
  };
  return t;
}

export function I18nProvider({ children, language: initial }: { children: ReactNode; language?: Language }) {
  const [language, setCurrent] = useState<Language>(() => initial ?? detectLanguage());
  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);
  const setLanguage = useCallback((next: Language) => {
    rememberLanguage(next);
    setCurrent(next);
  }, []);
  const value = useMemo(
    () => ({ language, setLanguage, t: makeTranslate(language) }),
    [language, setLanguage],
  );
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18n {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error('useI18n needs an I18nProvider');
  }
  return context;
}

export function useT(): Translate {
  return useI18n().t;
}
