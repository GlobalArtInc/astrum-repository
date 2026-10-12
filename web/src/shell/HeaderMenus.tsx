import { Button, DropdownMenu, IconButton, Text, Tooltip } from '@radix-ui/themes';
import {
  BookOpen,
  CircleHelp,
  KeyRound,
  Languages,
  LogIn,
  LogOut,
  Monitor,
  Moon,
  Package,
  Sun,
  UserRound,
} from 'lucide-react';
import { Link, useLocation, useNavigate } from 'react-router';
import { loginPath } from '../auth/returnTo';
import { useSession } from '../auth/state';
import { useSignOut } from '../auth/useSignOut';
import { LANGUAGES, type Language, useI18n } from '../i18n';
import styles from './Shell.module.css';
import { type ThemePreference, useTheme } from './theme';

export const DOCUMENTATION_URL = 'https://github.com/GlobalArtInc/astrum-repository/tree/main/docs';

const THEME_LABELS: Record<ThemePreference, string> = {
  dark: 'shell.theme.dark',
  light: 'shell.theme.light',
  system: 'shell.theme.system',
};

const LANGUAGE_NAMES: Record<Language, string> = {
  en: 'shell.language.en',
  ru: 'shell.language.ru',
};

type Size = '2' | '3';

export function ThemeMenu({ size = '3' }: { size?: Size }) {
  const { t } = useI18n();
  const { preference, appearance, setPreference } = useTheme();
  const Icon = preference === 'system' ? Monitor : appearance === 'dark' ? Moon : Sun;
  return (
    <DropdownMenu.Root>
      <Tooltip content={t('shell.theme.label')}>
        <DropdownMenu.Trigger>
          <IconButton variant="surface" color="gray" size={size} aria-label={t('shell.theme.label')}>
            <Icon size={18} aria-hidden="true" />
          </IconButton>
        </DropdownMenu.Trigger>
      </Tooltip>
      <DropdownMenu.Content align="end">
        <DropdownMenu.Label>{t('shell.theme.label')}</DropdownMenu.Label>
        <DropdownMenu.RadioGroup
          value={preference}
          onValueChange={(value) => setPreference(value as ThemePreference)}
        >
          {(['dark', 'light', 'system'] as const).map((value) => (
            <DropdownMenu.RadioItem key={value} value={value}>
              {t(THEME_LABELS[value])}
            </DropdownMenu.RadioItem>
          ))}
        </DropdownMenu.RadioGroup>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}

export function LanguageMenu({ size = '3' }: { size?: Size }) {
  const { t, language, setLanguage } = useI18n();
  return (
    <DropdownMenu.Root>
      <Tooltip content={t('shell.language.label')}>
        <DropdownMenu.Trigger>
          <IconButton variant="surface" color="gray" size={size} aria-label={t('shell.language.label')}>
            <Languages size={18} aria-hidden="true" />
          </IconButton>
        </DropdownMenu.Trigger>
      </Tooltip>
      <DropdownMenu.Content align="end">
        <DropdownMenu.Label>{t('shell.language.label')}</DropdownMenu.Label>
        <DropdownMenu.RadioGroup value={language} onValueChange={(value) => setLanguage(value as Language)}>
          {LANGUAGES.map((value) => (
            <DropdownMenu.RadioItem key={value} value={value} lang={value}>
              {t(LANGUAGE_NAMES[value])}
            </DropdownMenu.RadioItem>
          ))}
        </DropdownMenu.RadioGroup>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}

export function HelpMenu({ size = '3' }: { size?: Size }) {
  const { t } = useI18n();
  const { state } = useSession();
  return (
    <DropdownMenu.Root>
      <Tooltip content={t('shell.help.label')}>
        <DropdownMenu.Trigger>
          <IconButton variant="surface" color="gray" size={size} aria-label={t('shell.help.label')}>
            <CircleHelp size={18} aria-hidden="true" />
          </IconButton>
        </DropdownMenu.Trigger>
      </Tooltip>
      <DropdownMenu.Content align="end">
        <DropdownMenu.Item asChild>
          <a href={DOCUMENTATION_URL} target="_blank" rel="noopener noreferrer">
            <BookOpen size={14} aria-hidden="true" />
            {t('shell.help.documentation')}
          </a>
        </DropdownMenu.Item>
        <DropdownMenu.Separator />
        <DropdownMenu.Label>{t('shell.help.version', { version: versionOf(state) })}</DropdownMenu.Label>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}

export function versionOf(state: { version: string; revision: string | null }): string {
  return state.revision ? `${state.version} (${state.revision})` : state.version;
}

export function initials(name: string, userId: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  const letters =
    words.length >= 2 ? `${words[0]?.[0] ?? ''}${words[1]?.[0] ?? ''}` : (words[0] ?? userId).slice(0, 2);
  return letters.toUpperCase();
}

export function UserMenu() {
  const { t } = useI18n();
  const { user, state } = useSession();
  const location = useLocation();
  const navigate = useNavigate();
  const signOut = useSignOut();
  if (!user) {
    return (
      <Button asChild variant="soft" size="3">
        <Link to={loginPath(location.pathname + location.search)}>
          <LogIn size={16} aria-hidden="true" />
          {t('shell.user.signIn')}
        </Link>
      </Button>
    );
  }
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <button
          type="button"
          className={styles.avatar}
          aria-label={t('shell.user.menu', { name: user.name })}
        >
          {initials(user.name, user.userId)}
        </button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end" style={{ minWidth: 220 }}>
        <DropdownMenu.Label style={{ height: 'auto', flexDirection: 'column', alignItems: 'flex-start' }}>
          <Text size="2" weight="bold" style={{ color: 'var(--gray-12)' }}>
            {user.name}
          </Text>
          <Text size="1" color="gray">
            {user.userId}
          </Text>
        </DropdownMenu.Label>
        <DropdownMenu.Separator />
        <DropdownMenu.Item onSelect={() => navigate('/user/account')}>
          <UserRound size={14} aria-hidden="true" />
          {t('shell.user.account')}
        </DropdownMenu.Item>
        <DropdownMenu.Item onSelect={() => navigate('/user/access-tokens')}>
          <KeyRound size={14} aria-hidden="true" />
          {t('shell.user.accessTokens')}
        </DropdownMenu.Item>
        {state.nugetApiKeyRealm && (
          <DropdownMenu.Item onSelect={() => navigate('/user/nuget-api-key')}>
            <Package size={14} aria-hidden="true" />
            {t('shell.user.nugetApiKey')}
          </DropdownMenu.Item>
        )}
        <DropdownMenu.Separator />
        <DropdownMenu.Item onSelect={() => void signOut()}>
          <LogOut size={14} aria-hidden="true" />
          {t('shell.user.signOut')}
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}
