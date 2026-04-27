import { Card, Heading, IconButton, Text, TextField } from '@radix-ui/themes';
import { ChevronRight, Search, Settings, X } from 'lucide-react';
import { useState } from 'react';
import { Link, Navigate, useLocation } from 'react-router';
import { useRegistry, useVisibleSettings } from '../../app/registry';
import { loginPath } from '../../auth/returnTo';
import { useSession } from '../../auth/state';
import { type Translate, useT } from '../../i18n';
import { EmptyState, Page } from '../../ui';
import styles from './SettingsHub.module.css';
import { type ResolvedTile, resolveTile, SECTIONS } from './sections';

export function matchesFilter(t: Translate, tile: ResolvedTile, filter: string): boolean {
  const wanted = filter.trim().toLowerCase();
  return (
    wanted === '' ||
    t(tile.title).toLowerCase().includes(wanted) ||
    t(tile.description).toLowerCase().includes(wanted)
  );
}

function TileCard({ tile }: { tile: ResolvedTile }) {
  const t = useT();
  const Icon = tile.icon;
  return (
    <Card asChild size="1" className={styles.tile}>
      <Link to={tile.to}>
        <div className={styles.body}>
          <span className={styles.icon} aria-hidden="true">
            <Icon size={18} />
          </span>
          <div className={styles.texts}>
            <Text as="div" size="2" weight="medium" className={styles.name}>
              {t(tile.title)}
            </Text>
            <Text as="p" size="2" className={styles.description}>
              {t(tile.description)}
            </Text>
          </div>
          <ChevronRight size={18} className={styles.chevron} aria-hidden="true" />
        </div>
      </Link>
    </Card>
  );
}

export function SettingsHubPage() {
  const t = useT();
  const { user } = useSession();
  const location = useLocation();
  const tiles = useVisibleSettings()
    .map(resolveTile)
    .filter((tile): tile is ResolvedTile => tile !== null);
  const registered = useRegistry().settings.length;
  const [filter, setFilter] = useState('');
  if (tiles.length === 0 && !user) {
    return <Navigate to={loginPath(location.pathname)} replace />;
  }
  const shown = tiles.filter((tile) => matchesFilter(t, tile, filter));
  const sections = SECTIONS.map((section) => ({
    ...section,
    tiles: shown
      .filter((tile) => tile.section === section.id)
      .sort((left, right) => left.order - right.order),
  })).filter((section) => section.tiles.length > 0);
  return (
    <Page title={t('settings.hub.title')} description={t('settings.hub.description')} className={styles.hub}>
      {tiles.length > 0 && (
        <TextField.Root
          size="3"
          value={filter}
          placeholder={t('settings.hub.filter')}
          aria-label={t('settings.hub.filter')}
          onChange={(event) => setFilter(event.target.value)}
          type="search"
          className={styles.search}
        >
          <TextField.Slot>
            <Search size={16} aria-hidden="true" />
          </TextField.Slot>
          {filter !== '' && (
            <TextField.Slot>
              <IconButton
                size="1"
                variant="ghost"
                color="gray"
                aria-label={t('settings.hub.clear')}
                onClick={() => setFilter('')}
              >
                <X size={14} aria-hidden="true" />
              </IconButton>
            </TextField.Slot>
          )}
        </TextField.Root>
      )}
      {registered === 0 ? (
        <EmptyState icon={Settings} title={t('settings.hub.empty')} />
      ) : tiles.length === 0 ? (
        <EmptyState icon={Settings} title={t('settings.hub.none')} description={t('settings.hub.noneText')} />
      ) : sections.length === 0 ? (
        <EmptyState
          icon={Search}
          title={t('settings.hub.noMatches', { query: filter.trim() })}
          description={t('settings.hub.noMatchesText')}
        />
      ) : (
        <div className={styles.sections}>
          {sections.map((section) => (
            <section key={section.id} aria-labelledby={`settings-${section.id}`}>
              <Heading as="h2" size="5" id={`settings-${section.id}`} className={styles.sectionTitle}>
                {t(section.title)}
              </Heading>
              <ul className={styles.grid}>
                {section.tiles.map((tile) => (
                  <li key={tile.id}>
                    <TileCard tile={tile} />
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}
    </Page>
  );
}
