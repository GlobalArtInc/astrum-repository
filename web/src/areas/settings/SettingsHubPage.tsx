import { Flex, Heading, Text, TextField } from '@radix-ui/themes';
import { Search, Settings } from 'lucide-react';
import { useState } from 'react';
import { Link, Navigate, useLocation } from 'react-router';
import type { SettingsTile } from '../../app/area';
import { useRegistry, useVisibleSettings } from '../../app/registry';
import { loginPath } from '../../auth/returnTo';
import { useSession } from '../../auth/state';
import { type Translate, useT } from '../../i18n';
import { EmptyState, Page } from '../../ui';
import styles from './SettingsHub.module.css';
import { SECTIONS } from './sections';

export function matchesFilter(t: Translate, tile: SettingsTile, filter: string): boolean {
  const wanted = filter.trim().toLowerCase();
  return (
    wanted === '' ||
    t(tile.title).toLowerCase().includes(wanted) ||
    t(tile.description).toLowerCase().includes(wanted)
  );
}

export function SettingsHubPage() {
  const t = useT();
  const { user } = useSession();
  const location = useLocation();
  const tiles = useVisibleSettings();
  const registered = useRegistry().settings.length;
  const [filter, setFilter] = useState('');
  if (tiles.length === 0 && !user) {
    return <Navigate to={loginPath(location.pathname)} replace />;
  }
  const shown = tiles.filter((tile) => matchesFilter(t, tile, filter));
  const sections = SECTIONS.map((section) => ({
    ...section,
    tiles: shown.filter((tile) => tile.section === section.id),
  })).filter((section) => section.tiles.length > 0);
  return (
    <Page title={t('settings.hub.title')} description={t('settings.hub.description')}>
      {tiles.length > 0 && (
        <TextField.Root
          size="3"
          type="search"
          value={filter}
          placeholder={t('settings.hub.filter')}
          aria-label={t('settings.hub.filter')}
          onChange={(event) => setFilter(event.target.value)}
        >
          <TextField.Slot>
            <Search size={16} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
      )}
      {registered === 0 ? (
        <EmptyState icon={Settings} title={t('settings.hub.empty')} />
      ) : tiles.length === 0 ? (
        <EmptyState icon={Settings} title={t('settings.hub.none')} description={t('settings.hub.noneText')} />
      ) : sections.length === 0 ? (
        <EmptyState icon={Search} title={t('settings.hub.noMatches')} />
      ) : (
        sections.map((section) => (
          <section key={section.id} className={styles.section} aria-labelledby={`settings-${section.id}`}>
            <Heading as="h2" size="5" id={`settings-${section.id}`}>
              {t(section.title)}
            </Heading>
            <ul className={styles.grid}>
              {section.tiles.map((tile) => {
                const Icon = tile.icon;
                return (
                  <li key={tile.id}>
                    <Link to={tile.to} className={styles.tile}>
                      <span className={styles.icon} aria-hidden="true">
                        <Icon size={18} />
                      </span>
                      <Flex direction="column" gap="1">
                        <Text size="3" weight="medium">
                          {t(tile.title)}
                        </Text>
                        <Text size="2" color="gray">
                          {t(tile.description)}
                        </Text>
                      </Flex>
                    </Link>
                  </li>
                );
              })}
            </ul>
          </section>
        ))
      )}
    </Page>
  );
}
