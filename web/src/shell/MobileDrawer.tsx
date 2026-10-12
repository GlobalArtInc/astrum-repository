import { Flex, IconButton, Theme } from '@radix-ui/themes';
import { X } from 'lucide-react';
import { Dialog } from 'radix-ui';
import { useT } from '../i18n';
import { HelpMenu, LanguageMenu, ThemeMenu } from './HeaderMenus';
import { Logo } from './Logo';
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
                <span>
                  <span className="visually-hidden">{t('shell.menu.label')}</span>
                  <Logo />
                </span>
              </Dialog.Title>
              <Dialog.Close asChild>
                <IconButton variant="ghost" color="gray" size="2" aria-label={t('shell.menu.close')}>
                  <X size={18} aria-hidden="true" />
                </IconButton>
              </Dialog.Close>
            </div>
            <nav aria-label={t('shell.menu.label')} className={styles.drawerBody}>
              <MenuItems onNavigate={() => onOpenChange(false)} />
            </nav>
            <div className={styles.drawerFoot}>
              <Flex gap="2">
                <ThemeMenu size="2" />
                <LanguageMenu size="2" />
                <HelpMenu size="2" />
              </Flex>
              <span className={styles.version}>{t('shell.menu.version', { version })}</span>
            </div>
          </Dialog.Content>
        </Theme>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
