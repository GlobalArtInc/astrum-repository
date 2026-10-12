import { Button, Heading, Text } from '@radix-ui/themes';
import { BookOpen, Ghost, House } from 'lucide-react';
import { Link } from 'react-router';
import { useSession } from '../auth/state';
import { useT } from '../i18n';
import { useDocumentTitle } from '../ui/hooks';
import { documentationUrl } from './HeaderMenus';
import styles from './NotFoundPage.module.css';

export function NotFoundPage() {
  const t = useT();
  const { state } = useSession();
  useDocumentTitle(t('shell.notFound.title'));
  return (
    <div className={styles.page}>
      <div className={styles.content}>
        <div className={styles.ghost} aria-hidden="true">
          <Ghost size={80} strokeWidth={1.5} />
        </div>
        <div className={styles.code} aria-hidden="true">
          404
        </div>
        <Heading as="h1" size="6" className={styles.title}>
          {t('shell.notFound.title')}
        </Heading>
        <div className={styles.divider} aria-hidden="true" />
        <Text as="p" size="3" className={styles.message}>
          {t('shell.notFound.description')}
        </Text>
        <div className={styles.actions}>
          <Text as="p" size="2" className={styles.next}>
            {t('shell.notFound.next')}
          </Text>
          <div className={styles.buttons}>
            <Button asChild size="3" className={styles.solid}>
              <Link to="/">
                <House size={16} aria-hidden="true" />
                {t('shell.notFound.home')}
              </Link>
            </Button>
            <Button asChild size="3" variant="soft" className={styles.soft}>
              <a href={documentationUrl(state.version)} target="_blank" rel="noopener noreferrer">
                <BookOpen size={16} aria-hidden="true" />
                {t('shell.notFound.docs')}
              </a>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
