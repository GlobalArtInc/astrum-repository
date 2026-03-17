import { Theme } from '@radix-ui/themes';
import { X } from 'lucide-react';
import { Dialog } from 'radix-ui';
import { useT } from '../i18n';
import { HeaderButton, HelpMenu, LanguageMenu } from './HeaderMenus';
import { Wordmark } from './Logo';
import styles from './Shell.module.css';
import { MenuItems } from './SideMenu';

export function MobileDrawer({
  open,
  onOpenChange,
  version,
}: {
  open: boolean;
  onOpenChange(open: boolean): void;
  version: string;
}) {
  const t = useT();
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Theme>
          <Dialog.Overlay className={styles.scrim} />
          <Dialog.Content className={styles.drawer} aria-describedby={undefined}>
            <div className={styles.drawerHead}>
              <Dialog.Title asChild>
                <span className={styles.drawerTitle}>
                  <span className="visually-hidden">{t('shell.menu.label')}</span>
                  <Wordmark height={24} />
                </span>
              </Dialog.Title>
              <Dialog.Close asChild>
                <HeaderButton label={t('shell.menu.close')} className={styles.ghost}>
                  <X size={20} aria-hidden="true" />
                </HeaderButton>
              </Dialog.Close>
            </div>
            <nav aria-label={t('shell.menu.label')} className={styles.drawerBody}>
              <MenuItems onNavigate={() => onOpenChange(false)} />
            </nav>
            <div className={styles.drawerFoot}>
              <div className={styles.drawerTools}>
                <LanguageMenu />
                <HelpMenu />
              </div>
              <span className={styles.version}>{t('shell.menu.version', { version })}</span>
            </div>
          </Dialog.Content>
        </Theme>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
