import { Menu } from 'lucide-react';
import { Link } from 'react-router';
import { useT } from '../i18n';
import { Tooltip } from '../ui';
import { PHONE, useMediaQuery } from '../ui/hooks';
import {
  HeaderButton,
  HelpMenu,
  LanguageMenu,
  RefreshButton,
  SearchButton,
  StatusBell,
  ThemeToggle,
  UserMenu,
} from './HeaderMenus';
import { Logo } from './Logo';
import styles from './Shell.module.css';
import { SearchDialog, SearchTrigger } from './search';

export { isMac } from './search/SearchPanel';

export interface HeaderProps {
  compact: boolean;
  collapsed: boolean;
  onToggleMenu(): void;
  searchOpen: boolean;
  onSearchOpenChange(open: boolean): void;
}

export function Header({ compact, collapsed, onToggleMenu, searchOpen, onSearchOpenChange }: HeaderProps) {
  const t = useT();
  const phone = useMediaQuery(PHONE);
  const toggleLabel = compact
    ? t('shell.menu.open')
    : collapsed
      ? t('shell.menu.expand')
      : t('shell.menu.collapse');
  return (
    <header className={styles.header} data-compact={compact || undefined}>
      <div className={styles.headerStart}>
        <Tooltip content={toggleLabel}>
          <HeaderButton
            label={toggleLabel}
            aria-expanded={compact ? undefined : !collapsed}
            onClick={onToggleMenu}
          >
            <Menu size={compact ? 20 : 18} aria-hidden="true" />
          </HeaderButton>
        </Tooltip>
        <Link to="/" className={styles.home} aria-label={t('shell.header.home')}>
          <Logo compact={compact} />
        </Link>
      </div>
      <div className={styles.headerSearch}>
        {!compact && <SearchTrigger onOpen={() => onSearchOpenChange(true)} />}
      </div>
      <div className={styles.headerEnd}>
        {compact ? <SearchButton onClick={() => onSearchOpenChange(true)} /> : <RefreshButton />}
        <ThemeToggle />
        {!phone && <LanguageMenu />}
        <StatusBell />
        {!phone && <HelpMenu />}
        <UserMenu />
      </div>
      <SearchDialog open={searchOpen} onOpenChange={onSearchOpenChange} />
    </header>
  );
}
