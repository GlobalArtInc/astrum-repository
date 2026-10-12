import { IconButton, Tooltip } from '@radix-ui/themes';
import { Menu, Search } from 'lucide-react';
import { Link } from 'react-router';
import { useT } from '../i18n';
import { HelpMenu, LanguageMenu, ThemeMenu, UserMenu } from './HeaderMenus';
import { Logo } from './Logo';
import styles from './Shell.module.css';

export function isMac(): boolean {
  return /mac|iphone|ipad/i.test(navigator.platform || navigator.userAgent);
}

export interface HeaderProps {
  compact: boolean;
  collapsed: boolean;
  onToggleMenu(): void;
  onOpenSearch(): void;
}

export function Header({ compact, collapsed, onToggleMenu, onOpenSearch }: HeaderProps) {
  const t = useT();
  const toggleLabel = compact
    ? t('shell.menu.open')
    : collapsed
      ? t('shell.menu.expand')
      : t('shell.menu.collapse');
  return (
    <header className={styles.header}>
      <div className={styles.headerStart}>
        <Tooltip content={toggleLabel}>
          <IconButton
            variant="surface"
            color="gray"
            size="3"
            aria-label={toggleLabel}
            aria-expanded={compact ? undefined : !collapsed}
            onClick={onToggleMenu}
          >
            <Menu size={18} aria-hidden="true" />
          </IconButton>
        </Tooltip>
        <Link to="/" className={styles.home} aria-label={t('shell.header.home')}>
          <Logo />
        </Link>
      </div>
      <div className={styles.headerSearch}>
        <button type="button" className={styles.searchBox} onClick={onOpenSearch} aria-haspopup="dialog">
          <Search size={16} aria-hidden="true" />
          <span className={styles.searchText}>{t('shell.search.placeholder')}</span>
          <kbd className={styles.kbd}>{isMac() ? '⌘ K' : 'Ctrl K'}</kbd>
        </button>
        <IconButton
          className={styles.searchIcon}
          variant="surface"
          color="gray"
          size="3"
          aria-label={t('shell.search.open')}
          aria-haspopup="dialog"
          onClick={onOpenSearch}
        >
          <Search size={18} aria-hidden="true" />
        </IconButton>
      </div>
      <div className={styles.headerEnd}>
        <span className={styles.wide}>
          <ThemeMenu />
        </span>
        <span className={styles.wide}>
          <LanguageMenu />
        </span>
        <span className={styles.wide}>
          <HelpMenu />
        </span>
        <UserMenu />
      </div>
    </header>
  );
}
