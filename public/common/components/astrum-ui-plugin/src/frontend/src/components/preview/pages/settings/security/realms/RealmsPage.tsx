

import React, { useState, useCallback, useMemo } from 'react';
import { Box, Flex, Text, Heading } from '@radix-ui/themes';
import {
  Loader2,
  GripVertical,
  ChevronRight,
  ChevronLeft,
  ChevronsRight,
  ChevronsLeft,
  Search,
  ArrowUp,
  ArrowDown,
} from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import {
  SettingsForm,
  SettingsAlert,
} from '../../../../shared/form';
import { HelpSection, PageHeader } from '../../../../shared';
import { useRealmsForm } from './useRealmsForm';
import { Realm, RealmsPageProps } from './types';

import './RealmsPage.scss';
import RealmsStrings from '../../../../constants/pages/settings/security/RealmsStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import plural from '../../../../../../interface/plural';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

/**
 * RealmsPage - Security Realms configuration page for Preview UI
 *
 * Allows configuring which security realms are active and their order.
 * Order matters - realms are checked in sequence for authentication.
 * Uses XState form machine for state management.
 */
export function RealmsPage({ className }: RealmsPageProps) {
  // XState form hook handles load, save, dirty tracking, toast, validation
  const realmsForm = useRealmsForm();

  // Local UI state for selections and search
  const [availableSearch, setAvailableSearch] = useState('');
  const [activeSearch, setActiveSearch] = useState('');
  const [availableSelection, setAvailableSelection] = useState<string[]>([]);
  const [activeSelection, setActiveSelection] = useState<string[]>([]);
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);

  const canUpdate = ExtJS.checkPermission('astrum:settings:update');

  // Filtered lists
  const filteredInactive = useMemo(() => {
    if (!availableSearch) return realmsForm.inactiveRealms;
    const search = availableSearch.toLowerCase();
    return realmsForm.inactiveRealms.filter((r) => r.name.toLowerCase().includes(search));
  }, [realmsForm.inactiveRealms, availableSearch]);

  const filteredActive = useMemo(() => {
    if (!activeSearch) return realmsForm.activeRealms;
    const search = activeSearch.toLowerCase();
    return realmsForm.activeRealms.filter((r) => r.name.toLowerCase().includes(search));
  }, [realmsForm.activeRealms, activeSearch]);

  // Handle click on available item
  const handleAvailableClick = useCallback((realm: Realm, e: React.MouseEvent) => {
    const id = realm.id;
    if (e.ctrlKey || e.metaKey) {
      setAvailableSelection((prev) =>
        prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]
      );
    } else {
      setAvailableSelection([id]);
    }
    setActiveSelection([]);
  }, []);

  // Handle click on active item
  const handleActiveClick = useCallback((realm: Realm, e: React.MouseEvent) => {
    const id = realm.id;
    if (e.ctrlKey || e.metaKey) {
      setActiveSelection((prev) =>
        prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]
      );
    } else {
      setActiveSelection([id]);
    }
    setAvailableSelection([]);
  }, []);

  // Move selected to active
  const moveToActive = useCallback(() => {
    if (availableSelection.length === 0) return;
    const realmsToAdd = realmsForm.inactiveRealms.filter((r) => availableSelection.includes(r.id));
    realmsForm.reorder([...realmsForm.activeRealms, ...realmsToAdd]);
    setAvailableSelection([]);
  }, [availableSelection, realmsForm]);

  // Move selected to inactive
  const moveToInactive = useCallback(() => {
    if (activeSelection.length === 0) return;
    realmsForm.reorder(realmsForm.activeRealms.filter((r) => !activeSelection.includes(r.id)));
    setActiveSelection([]);
  }, [activeSelection, realmsForm]);

  // Move all to active
  const moveAllToActive = useCallback(() => {
    realmsForm.reorder(realmsForm.availableRealms);
    setAvailableSelection([]);
  }, [realmsForm]);

  // Move all to inactive
  const moveAllToInactive = useCallback(() => {
    realmsForm.reorder([]);
    setActiveSelection([]);
  }, [realmsForm]);

  // Double click to move single item
  const handleDoubleClickInactive = useCallback((realm: Realm) => {
    realmsForm.addRealm(realm);
    setAvailableSelection([]);
  }, [realmsForm]);

  const handleDoubleClickActive = useCallback((realm: Realm) => {
    realmsForm.removeRealm(realm.id);
    setActiveSelection([]);
  }, [realmsForm]);

  // Move selected item up/down in active list
  const handleMoveUp = useCallback(() => {
    if (activeSelection.length !== 1) return;
    realmsForm.moveUp(activeSelection[0]);
  }, [activeSelection, realmsForm]);

  const handleMoveDown = useCallback(() => {
    if (activeSelection.length !== 1) return;
    realmsForm.moveDown(activeSelection[0]);
  }, [activeSelection, realmsForm]);

  // Drag and drop handlers for reordering
  const handleDragStart = useCallback((e: React.DragEvent, index: number) => {
    setDraggedIndex(index);
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', index.toString());
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent, _index: number) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDrop = useCallback((e: React.DragEvent, dropIndex: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === dropIndex) {
      setDraggedIndex(null);
      return;
    }

    const newRealms = [...realmsForm.activeRealms];
    const [removed] = newRealms.splice(draggedIndex, 1);
    newRealms.splice(dropIndex, 0, removed);
    realmsForm.reorder(newRealms);
    setDraggedIndex(null);
  }, [draggedIndex, realmsForm]);

  const handleDragEnd = useCallback(() => {
    setDraggedIndex(null);
  }, []);

  // Loading state
  if (realmsForm.isLoading) {
    return (
      <Box
        className={`realms-page ${className || ''}`.trim()}
        data-testid="realms-page"
        data-view="edit"
      >
        <Flex align="center" justify="center" className="realms-page__loading">
          <Loader2 size={24} className="realms-page__spinner" />
          <Text size="2">{RealmsStrings.PAGE.LOADING_REALM_CONFIGURATION}</Text>
        </Flex>
      </Box>
    );
  }

  return (
    <Box
      className={`realms-page ${className || ''}`.trim()}
      data-testid="realms-page"
      data-view="edit"
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      {/* Header */}
      <Box mb="4">
        <PageHeader
          title={RealmsStrings.PAGE.REALMS}
          description={RealmsStrings.PAGE.CONFIGURE_ACTIVE_SECURITY_REALMS_THEIR}
        
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: RealmsStrings.PAGE.REALMS }
          ]}
/>
      </Box>

      {/* Alerts */}
      {(realmsForm.saveError || realmsForm.loadError) && (
        <Box className="realms-page__alerts">
          <SettingsAlert type="error">
            {realmsForm.saveError || realmsForm.loadError}
          </SettingsAlert>
        </Box>
      )}

      {/* Form */}
      <SettingsForm
        onSubmit={() => realmsForm.submit()}
        onCancel={() => realmsForm.discard()}
        loading={realmsForm.isSaving}
        pristine={realmsForm.isPristine}
        showActions={canUpdate}
        testId="realms-form"
        submitAnalyticsId="as-realms-save"
        data-loading={realmsForm.isLoading || realmsForm.isSaving ? 'true' : 'false'}
        data-dirty={!realmsForm.isPristine ? 'true' : 'false'}
        data-valid="true"
      >
        {/* Section heading */}
        <Heading as="h2" size="3" mb="3">{RealmsStrings.PAGE.ACTIVE_REALMS}</Heading>

        {/* Transfer List */}
        <Box className="realms-page__transfer-container">
          {/* Available (Inactive) Panel */}
          <Box className="realms-page__panel">
            <Box className="realms-page__panel-header">
              <Text size="2" weight="medium">{RealmsStrings.PAGE.AVAILABLE}</Text>
              <Text size="1" className="realms-page__panel-count">
                {plural(RealmsStrings.PAGE.REALM_COUNT, filteredInactive.length)}
              </Text>
            </Box>
            <Box className="realms-page__search">
              <Search size={14} className="realms-page__search-icon" />
              <input
                type="text"
                placeholder={RealmsStrings.PAGE.FILTER_PLACEHOLDER}
                value={availableSearch}
                onChange={(e) => setAvailableSearch(e.target.value)}
                disabled={!canUpdate}
                className="realms-page__search-input"
              />
            </Box>
            <Box className="realms-page__items" role="listbox" aria-label={RealmsStrings.PAGE.AVAILABLE_REALMS_ARIA}>
              {filteredInactive.map((realm) => (
                <Box
                  key={realm.id}
                  role="option"
                  aria-selected={availableSelection.includes(realm.id)}
                  tabIndex={0}
                  onClick={(e) => canUpdate && handleAvailableClick(realm, e)}
                  onDoubleClick={() => canUpdate && handleDoubleClickInactive(realm)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && canUpdate) handleDoubleClickInactive(realm);
                  }}
                  className={`realms-page__item ${availableSelection.includes(realm.id) ? 'realms-page__item--selected' : ''} ${!canUpdate ? 'realms-page__item--disabled' : ''}`}
                >
                  <Text size="2">{realm.name}</Text>
                </Box>
              ))}
              {filteredInactive.length === 0 && (
                <Box className="realms-page__empty">
                  {availableSearch ? RealmsStrings.PAGE.NO_MATCHES : RealmsStrings.PAGE.NO_AVAILABLE_REALMS}
                </Box>
              )}
            </Box>
          </Box>

          {/* Transfer Controls */}
          <Box className="realms-page__controls">
            <button
              type="button"
              onClick={moveAllToActive}
              disabled={!canUpdate || realmsForm.inactiveRealms.length === 0}
              className="realms-page__control-button"
              aria-label={RealmsStrings.PAGE.ADD_ALL_REALMS_ARIA}
              title={RealmsStrings.PAGE.ADD_ALL}
            >
              <ChevronsRight size={16} />
            </button>
            <button
              type="button"
              onClick={moveToActive}
              disabled={!canUpdate || availableSelection.length === 0}
              className="realms-page__control-button"
              aria-label={RealmsStrings.PAGE.ADD_SELECTED_REALMS_ARIA}
              title={RealmsStrings.PAGE.ADD_SELECTED}
            >
              <ChevronRight size={16} />
            </button>
            <button
              type="button"
              onClick={moveToInactive}
              disabled={!canUpdate || activeSelection.length === 0}
              className="realms-page__control-button"
              aria-label={RealmsStrings.PAGE.REMOVE_SELECTED_REALMS_ARIA}
              title={RealmsStrings.PAGE.REMOVE_SELECTED}
            >
              <ChevronLeft size={16} />
            </button>
            <button
              type="button"
              onClick={moveAllToInactive}
              disabled={!canUpdate || realmsForm.activeRealms.length === 0}
              className="realms-page__control-button"
              aria-label={RealmsStrings.PAGE.REMOVE_ALL_REALMS_ARIA}
              title={RealmsStrings.PAGE.REMOVE_ALL}
            >
              <ChevronsLeft size={16} />
            </button>
          </Box>

          {/* Active Panel */}
          <Box className="realms-page__panel realms-page__panel--active">
            <Box className="realms-page__panel-header">
              <Text size="2" weight="medium">{RealmsStrings.PAGE.ACTIVE}</Text>
              <Text size="1" className="realms-page__panel-count">
                {plural(RealmsStrings.PAGE.REALM_COUNT, realmsForm.activeRealms.length)}
              </Text>
            </Box>
            <Box className="realms-page__search">
              <Search size={14} className="realms-page__search-icon" />
              <input
                type="text"
                placeholder={RealmsStrings.PAGE.FILTER_PLACEHOLDER}
                value={activeSearch}
                onChange={(e) => setActiveSearch(e.target.value)}
                disabled={!canUpdate}
                className="realms-page__search-input"
              />
            </Box>
            <Box className="realms-page__items realms-page__items--sortable" role="listbox" aria-label={RealmsStrings.PAGE.ACTIVE_REALMS_ARIA}>
              {filteredActive.map((realm) => {
                const actualIndex = realmsForm.activeRealms.findIndex((r) => r.id === realm.id);
                return (
                  <Box
                    key={realm.id}
                    role="option"
                    aria-selected={activeSelection.includes(realm.id)}
                    tabIndex={0}
                    draggable={canUpdate}
                    onDragStart={(e) => handleDragStart(e, actualIndex)}
                    onDragOver={(e) => handleDragOver(e, actualIndex)}
                    onDrop={(e) => handleDrop(e, actualIndex)}
                    onDragEnd={handleDragEnd}
                    onClick={(e) => canUpdate && handleActiveClick(realm, e)}
                    onDoubleClick={() => canUpdate && handleDoubleClickActive(realm)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && canUpdate) handleDoubleClickActive(realm);
                    }}
                    className={`realms-page__item realms-page__item--draggable ${activeSelection.includes(realm.id) ? 'realms-page__item--selected' : ''} ${draggedIndex === actualIndex ? 'realms-page__item--dragging' : ''} ${!canUpdate ? 'realms-page__item--disabled' : ''}`}
                  >
                    {canUpdate && (
                      <GripVertical size={14} className="realms-page__drag-handle" />
                    )}
                    <Text size="2">{realm.name}</Text>
                    <Text size="1" className="realms-page__item-order">
                      {actualIndex + 1}
                    </Text>
                  </Box>
                );
              })}
              {filteredActive.length === 0 && (
                <Box className="realms-page__empty">
                  {activeSearch ? RealmsStrings.PAGE.NO_MATCHES : RealmsStrings.PAGE.NO_ACTIVE_REALMS}
                </Box>
              )}
            </Box>
          </Box>

          {/* Reorder Controls */}
          {canUpdate && (
            <Box className="realms-page__reorder-controls">
              <button
                type="button"
                onClick={handleMoveUp}
                disabled={activeSelection.length !== 1 || realmsForm.activeRealms.findIndex((r) => r.id === activeSelection[0]) <= 0}
                className="realms-page__control-button"
                aria-label={RealmsStrings.PAGE.MOVE_UP_ARIA}
                title={RealmsStrings.PAGE.MOVE_UP}
              >
                <ArrowUp size={16} />
              </button>
              <button
                type="button"
                onClick={handleMoveDown}
                disabled={activeSelection.length !== 1 || realmsForm.activeRealms.findIndex((r) => r.id === activeSelection[0]) >= realmsForm.activeRealms.length - 1}
                className="realms-page__control-button"
                aria-label={RealmsStrings.PAGE.MOVE_DOWN_ARIA}
                title={RealmsStrings.PAGE.MOVE_DOWN}
              >
                <ArrowDown size={16} />
              </button>
            </Box>
          )}
        </Box>

        {/* Help Section */}
        <HelpSection
          title={RealmsStrings.PAGE.ABOUT_REALMS}
          content={RealmsStrings.PAGE.SECURITY_REALMS_USED_AUTHENTICATE_USERS}
          docLink={{
            label: CommonStrings.ACTIONS.VIEW_DOCUMENTATION,
            href: 'https://help.globalart.dev/en/realms.html',
          }}
        />
      </SettingsForm>
    </Box>
  );
}

export default RealmsPage;
