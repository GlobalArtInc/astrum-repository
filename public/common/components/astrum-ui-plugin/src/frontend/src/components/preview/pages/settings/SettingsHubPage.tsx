

import React, { useState, useMemo, useEffect } from 'react';
import {
  Box,
  Container,
  Flex,
  Text,
  TextField,
  IconButton,
  ScrollArea,
} from '@radix-ui/themes';
import { Search, X } from 'lucide-react';

import { ExtJS } from '../../../../interface/ExtJS';
import { isVisible } from '../../../../interface/NavigationUtils';
import { PageHeader } from '../../shared';
import { SETTINGS_SECTIONS } from './settingsConfig';
import SettingsSection from './SettingsSection';
import CommonStrings from '../../constants/CommonStrings';
import SettingsStrings from '../../constants/pages/settings/SettingsStrings';

/**
 * SettingsHubPage - Searchable hub for all admin settings
 *
 * Features:
 * - 36 settings cards organized in 4 sections
 * - Client-side search filtering by label, description, and search terms
 * - Empty sections automatically hidden when searching
 * - Cards navigate to individual settings pages
 * - Cards are a list on phones and tiles on wider screens
 *
 * URL: #preview/admin/settings
 */
export function SettingsHubPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const isCloud = ExtJS.state?.().getValue?.('isCloud', false) ?? false;
  const isProEdition = ExtJS.isProEdition?.() ?? false;
  const user = ExtJS.useUser();
  const isAdmin = !!user?.administrator;

  // Reactive trigger: increments when ExtJS permissions load or change,
  // forcing the useMemo to re-evaluate isVisible() with real permission data.
  const [permissionsVersion, setPermissionsVersion] = useState(0);
  useEffect(() => {
    let unmounted = false;
    let interval: ReturnType<typeof setInterval> | undefined;
    const cleanupRef: { fn?: () => void } = {};

    function subscribe() {
      const app = (window as any).Ext?.getApplication?.();
      if (!app) return false;

      const pc = app.getController?.('Permissions');
      const sc = app.getController?.('State');
      if (!pc) return false;

      if (unmounted) return true;

      const handler = () => { if (!unmounted) setPermissionsVersion((v) => v + 1); };
      pc.on('changed', handler);
      sc?.on('userchanged', handler);
      handler();
      cleanupRef.fn = () => {
        pc.un('changed', handler);
        sc?.un('userchanged', handler);
      };
      return true;
    }

    if (!subscribe()) {
      let retries = 0;
      interval = setInterval(() => {
        if (subscribe() || ++retries >= 100) clearInterval(interval!);
      }, 100);
    }

    return () => {
      unmounted = true;
      if (interval) clearInterval(interval);
      cleanupRef.fn?.();
    };
  }, []);

  // ExtJS.state() is a singleton store that React cannot observe — edition values
  // are read at startup and do not change during a session,
  // so omitting them from the dependency array is intentional.
  const filteredSections = useMemo(() => {
    let sections = SETTINGS_SECTIONS;

    if (isCloud) {
      sections = sections
        .filter((section) => !section.cloudExcluded)
        .map((section) => ({
          ...section,
          cards: section.cards.filter((card) => !card.cloudExcluded),
        }))
        .filter((section) => section.cards.length > 0);
    } else if (!isProEdition) {
      sections = sections
        .map((section) => ({
          ...section,
          cards: section.cards.filter((card) => !card.proOnly),
        }))
        .filter((section) => section.cards.length > 0);
    }

    // Cloud-only cards (e.g. Usage) are hidden on any self-hosted deployment.
    if (!isCloud) {
      sections = sections
        .map((section) => ({
          ...section,
          cards: section.cards.filter((card) => !card.cloudOnly),
        }))
        .filter((section) => section.cards.length > 0);
    }

    if (!isAdmin) {
      sections = sections
        .map((section) => ({
          ...section,
          cards: section.cards.filter((card) => !card.adminOnly),
        }))
        .filter((section) => section.cards.length > 0);
    }

    sections = sections
      .map((section) => ({
        ...section,
        cards: section.cards.filter((card) => isVisible(card.visibilityRequirements)),
      }))
      .filter((section) => section.cards.length > 0);

    if (!searchQuery.trim()) {
      return sections;
    }

    const query = searchQuery.toLowerCase().trim();

    return sections.map((section) => ({
      ...section,
      cards: section.cards.filter((card) =>
        card.label.toLowerCase().includes(query) ||
        card.description.toLowerCase().includes(query) ||
        card.searchTerms?.some((term) => term.toLowerCase().includes(query))
      ),
    })).filter((section) => section.cards.length > 0);
  }, [searchQuery, isCloud, isProEdition, isAdmin, permissionsVersion]);

  const hasResults = filteredSections.length > 0;

  return (
    <ScrollArea>
      <Container size="4" py="4" px={{ initial: '4', md: '0' }}>
        <PageHeader
          title={CommonStrings.LABELS.SETTINGS}
          description={SettingsStrings.HUB.CONFIGURE_SECURITY_REPOSITORIES_SYSTEM_SUPPORT}
        >
          {/* Search bar */}
          <Box mt="4">
            <TextField.Root
              size="3"
              placeholder={SettingsStrings.HUB.SEARCH_SETTINGS_PLACEHOLDER}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            >
              <TextField.Slot side="left">
                <Search size={16} />
              </TextField.Slot>
              {searchQuery && (
                <TextField.Slot side="right">
                  <IconButton
                    variant="ghost"
                    size="1"
                    onClick={() => setSearchQuery('')}
                    aria-label={SettingsStrings.HUB.CLEAR_SEARCH_ARIA}
                  >
                    <X size={14} />
                  </IconButton>
                </TextField.Slot>
              )}
            </TextField.Root>
          </Box>
        </PageHeader>

        {/* Settings sections */}
        <Box mt="6">
          {hasResults ? (
            <Flex direction="column" gap="6">
              {filteredSections.map((section) => (
                <SettingsSection key={section.id} section={section} />
              ))}
            </Flex>
          ) : (
            <EmptyState searchQuery={searchQuery} />
          )}
        </Box>
      </Container>
    </ScrollArea>
  );
}

/**
 * EmptyState - Shown when search returns no results
 */
interface EmptyStateProps {
  searchQuery: string;
}

function EmptyState({ searchQuery }: EmptyStateProps) {
  return (
    <Box py="8" style={{ textAlign: 'center' }}>
      <Flex direction="column" align="center" gap="2">
        <Text size="3" weight="medium" color="gray">
          {SettingsStrings.HUB.NO_SETTINGS_FOUND_MATCHING(searchQuery)}
        </Text>
        <Text size="2" color="gray">
          {SettingsStrings.HUB.TRY_DIFFERENT_SEARCH_TERM_BROWSE}
        </Text>
      </Flex>
    </Box>
  );
}

export default SettingsHubPage;
