import { DropdownMenu, Text } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import {
  Bell,
  CircleQuestionMark,
  ExternalLink,
  KeyRound,
  Languages,
  LogOut,
  Moon,
  Package,
  RefreshCw,
  Search,
  Sun,
  UserRound,
} from 'lucide-react';
import type { ButtonHTMLAttributes, ReactNode, Ref } from 'react';
import { Link, useLocation, useNavigate } from 'react-router';
import { INTERNAL, useApi } from '../api';
import { useVisibleSettings } from '../app/registry';
import { loginPath } from '../auth/returnTo';
import { useSession } from '../auth/state';
import { useSignOut } from '../auth/useSignOut';
import { LANGUAGES, type Language, useI18n } from '../i18n';
import { Tooltip } from '../ui';
import styles from './Shell.module.css';
import { useTheme } from './theme';

const LINKS = 'https://links.globalart.dev/products/astrum';
const UTM = 'utm_medium=product&utm_source=astrum_repo';

export function helpLink(path: string, campaign: string): string {
  return `${LINKS}/${path}?${UTM}&utm_campaign=${campaign}`;
}

export function documentationUrl(version: string): string {
  const [major = '0', minor = '0'] = version.split(/[.-]/);
  return helpLink(`docs/${major}.${minor}`, 'menu-docs');
}

export const DOCUMENTATION_URL = documentationUrl('3.0.0');

export const STATUS_TILE = 'metric-health';
export const STATUS_NEED = 'astrum:metrics:read';
export const STATUS_CHECKS = `${INTERNAL}/ui/status-check`;
const STATUS_POLL = 60_000;

const LANGUAGE_NAMES: Record<Language, string> = {
  en: 'shell.language.en',
  ru: 'shell.language.ru',
  de: 'shell.language.de',
  es: 'shell.language.es',
  fr: 'shell.language.fr',
  it: 'shell.language.it',
  ja: 'shell.language.ja',
  pl: 'shell.language.pl',
  pt: 'shell.language.pt',
  'sr-Latn': 'shell.language.srLatn',
  tr: 'shell.language.tr',
  zh: 'shell.language.zh',
};

type HeaderButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  label: string;
  ref?: Ref<HTMLButtonElement>;
};

export function HeaderButton({ label, children, className, ref, ...rest }: HeaderButtonProps) {
  return (
    <button
      ref={ref}
      type="button"
      aria-label={label}
      className={className ? `${styles.headerButton} ${className}` : styles.headerButton}
      {...rest}
    >
      {children}
    </button>
  );
}

function WithTooltip({ label, children }: { label: string; children: ReactNode }) {
  return <Tooltip content={label}>{children}</Tooltip>;
}

export function RefreshButton() {
  const { t } = useI18n();
  const client = useQueryClient();
  const label = t('shell.header.refresh');
  return (
    <WithTooltip label={label}>
      <HeaderButton label={label} onClick={() => void client.invalidateQueries()}>
        <RefreshCw size={18} aria-hidden="true" />
      </HeaderButton>
    </WithTooltip>
  );
}

export function SearchButton({ onClick }: { onClick(): void }) {
  const { t } = useI18n();
  return (
    <HeaderButton label={t('shell.search.open')} aria-haspopup="dialog" onClick={onClick}>
      <Search size={18} aria-hidden="true" />
    </HeaderButton>
  );
}

export function ThemeToggle() {
  const { t } = useI18n();
  const { appearance, toggle } = useTheme();
  const label = appearance === 'dark' ? t('shell.theme.toLight') : t('shell.theme.toDark');
  const Icon = appearance === 'dark' ? Sun : Moon;
  return (
    <WithTooltip label={label}>
      <HeaderButton label={label} onClick={toggle}>
        <Icon size={18} aria-hidden="true" />
      </HeaderButton>
    </WithTooltip>
  );
}

export function LanguageMenu() {
  const { t, language, setLanguage } = useI18n();
  const label = t('shell.language.label');
  return (
    <DropdownMenu.Root>
      <WithTooltip label={label}>
        <DropdownMenu.Trigger>
          <HeaderButton label={label}>
            <Languages size={18} aria-hidden="true" />
          </HeaderButton>
        </DropdownMenu.Trigger>
      </WithTooltip>
      <DropdownMenu.Content align="end">
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

export function StatusBell() {
  const { t } = useI18n();
  const { can } = useSession();
  const navigate = useNavigate();
  const health = useVisibleSettings().find((tile) => tile.id === STATUS_TILE);
  const allowed = can(STATUS_NEED);
  const checks = useApi<Record<string, { healthy: boolean }>>(allowed && health ? STATUS_CHECKS : null, {
    refetchInterval: STATUS_POLL,
    staleTime: STATUS_POLL / 2,
  });
  if (!allowed) {
    return null;
  }
  const failing = Object.values(checks.data ?? {}).filter((check) => !check.healthy).length;
  const label =
    failing > 0
      ? `${t('shell.status.label')}: ${t('shell.status.failing', { count: failing })}`
      : t('shell.status.label');
  return (
    <WithTooltip label={label}>
      <HeaderButton
        label={label}
        className={styles.statusButton}
        aria-disabled={health ? undefined : true}
        onClick={() => {
          if (health) {
            navigate(health.to);
          }
        }}
      >
        <Bell size={18} aria-hidden="true" />
        {failing > 0 && <span className={styles.statusDot} data-testid="status-dot" />}
      </HeaderButton>
    </WithTooltip>
  );
}

const HELP_LINKS: Array<Array<{ id: string; label: string; path: string; campaign: string }>> = [
  [
    {
      id: 'releaseNotes',
      label: 'shell.help.releaseNotes',
      path: 'release-notes',
      campaign: 'menu-release-notes',
    },
    { id: 'knowledgeBase', label: 'shell.help.knowledgeBase', path: 'kb', campaign: 'menu-knowledge' },
    { id: 'guides', label: 'shell.help.guides', path: 'guides', campaign: 'menu-guides' },
  ],
  [
    { id: 'community', label: 'shell.help.community', path: 'community', campaign: 'menu-community' },
    { id: 'issues', label: 'shell.help.issues', path: 'issues', campaign: 'menu-issuetracker' },
  ],
];

function ExternalItem({ href, children }: { href: string; children: ReactNode }) {
  return (
    <DropdownMenu.Item asChild>
      <a href={href} target="_blank" rel="noopener noreferrer" className={styles.external}>
        <span>{children}</span>
        <ExternalLink size={14} aria-hidden="true" />
      </a>
    </DropdownMenu.Item>
  );
}

export function HelpMenu() {
  const { t } = useI18n();
  const { state } = useSession();
  const label = t('shell.help.label');
  return (
    <DropdownMenu.Root>
      <WithTooltip label={label}>
        <DropdownMenu.Trigger>
          <HeaderButton label={label}>
            <CircleQuestionMark size={18} aria-hidden="true" />
          </HeaderButton>
        </DropdownMenu.Trigger>
      </WithTooltip>
      <DropdownMenu.Content align="end" className={styles.spaced}>
        <ExternalItem href={documentationUrl(state.version)}>{t('shell.help.documentation')}</ExternalItem>
        {HELP_LINKS[0]?.map((link) => (
          <ExternalItem key={link.id} href={helpLink(link.path, link.campaign)}>
            {t(link.label)}
          </ExternalItem>
        ))}
        <DropdownMenu.Separator />
        {HELP_LINKS[1]?.map((link) => (
          <ExternalItem key={link.id} href={helpLink(link.path, link.campaign)}>
            {t(link.label)}
          </ExternalItem>
        ))}
        <DropdownMenu.Separator />
        <DropdownMenu.Label>
          <Text size="1" className={styles.versionLabel}>
            {t('shell.help.version', { version: versionOf(state) })}
          </Text>
        </DropdownMenu.Label>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}

export function versionOf(state: { version: string; revision: string | null }): string {
  return state.revision ? `${state.version} (${state.revision})` : state.version;
}

export function initials(userId: string): string {
  return userId.trim().slice(0, 2).toUpperCase();
}

export function UserMenu() {
  const { t } = useI18n();
  const { user, state } = useSession();
  const location = useLocation();
  const navigate = useNavigate();
  const signOut = useSignOut();
  if (!user) {
    const label = t('shell.user.signIn');
    return (
      <WithTooltip label={label}>
        <Link
          to={loginPath(location.pathname + location.search)}
          className={styles.headerButton}
          aria-label={label}
        >
          <UserRound size={18} aria-hidden="true" />
        </Link>
      </WithTooltip>
    );
  }
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <button
          type="button"
          className={styles.avatar}
          aria-label={t('shell.user.menu', { name: user.name || user.userId })}
        >
          <span className={styles.avatarCircle}>{initials(user.userId)}</span>
        </button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end" className={`${styles.userMenu} ${styles.spaced}`}>
        <DropdownMenu.Label className={styles.userLabel}>
          <span className={styles.avatarSmall} aria-hidden="true">
            {initials(user.userId)}
          </span>
          <Text size="2" weight="bold" className={styles.userId}>
            {user.userId}
          </Text>
        </DropdownMenu.Label>
        <DropdownMenu.Separator />
        <DropdownMenu.Item onSelect={() => navigate('/user/account')}>
          <UserRound size={16} aria-hidden="true" />
          {t('shell.user.account')}
        </DropdownMenu.Item>
        <DropdownMenu.Item onSelect={() => navigate('/user/access-tokens')}>
          <KeyRound size={16} aria-hidden="true" />
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
          <LogOut size={16} aria-hidden="true" />
          {t('shell.user.signOut')}
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}
