

import React from 'react';
import { Box, Flex, Text, Select } from '@radix-ui/themes';
import { Loader2, Download, Copy, RefreshCw, Info, ExternalLink, ChevronsDownUp, ChevronsUpDown } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsAlert, SettingsButton } from '../../../../shared/form';
import { PageHeader } from '../../../../shared';
import { SystemInfoSection } from './SystemInfoSection';
import { NodeSelector } from './NodeSelector';
import { useSystemInfo } from './useSystemInfo';
import { SystemInfoPageProps, formatSectionTitle } from './types';

import './SystemInfoPage.scss';
import SystemInfoStrings from '../../../../constants/pages/settings/support/SystemInfoStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * SystemInfoPage - System Information page for Preview UI
 *
 * Displays detailed system information including Astrum status,
 * configuration, and various system properties.
 */
export function SystemInfoPage({ className }: SystemInfoPageProps) {
  const {
    systemInfo,
    nodes,
    selectedNode,
    isHAMode,
    isLoading,
    isRefreshing,
    error,
    expandedSections,
    sections,
    sectionRefs,
    handleNodeSelect,
    handleRefresh,
    handleDownload,
    handleCopy,
    handleExpandAll,
    handleCollapseAll,
    handleSectionToggle,
    handleJumpToSection,
    clearError,
  } = useSystemInfo();

  const canRead = ExtJS.checkPermission('astrum:atlas:read');

  // Loading state
  if (isLoading) {
    return (
      <Box className={`system-info-page ${className || ''}`.trim()}>
        <Flex align="center" justify="center" className="system-info-page__loading">
          <Loader2 size={24} className="system-info-page__spinner" />
          <Text size="2">{SystemInfoStrings.PAGE.LOADING_SYSTEM_INFORMATION}</Text>
        </Flex>
      </Box>
    );
  }

  // No permission state
  if (!canRead) {
    return (
      <Box className={`system-info-page ${className || ''}`.trim()}>
        <PageHeader
          title={SystemInfoStrings.PAGE.SYSTEM_INFORMATION}
          description={SystemInfoStrings.PAGE.VIEW_DETAILED_SYSTEM_SERVER_INFORMATION}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#preview/admin/settings'; } },
            { label: SystemInfoStrings.PAGE.SYSTEM_INFORMATION },
          ]}
        />

        <SettingsAlert type="warning">
          {SystemInfoStrings.PAGE.YOU_DO_NOT_HAVE_PERMISSION}
        </SettingsAlert>
      </Box>
    );
  }

  return (
    <Box className={`system-info-page ${className || ''}`.trim()}>
      {/* Header */}
      <PageHeader
        title={SystemInfoStrings.PAGE.SYSTEM_INFORMATION}
        description={SystemInfoStrings.PAGE.VIEW_DETAILED_SYSTEM_SERVER_INFORMATION}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => { window.location.hash = '#preview/admin/settings'; } },
          { label: SystemInfoStrings.PAGE.SYSTEM_INFORMATION },
        ]}
        actions={
          <Flex gap="2" className="system-info-page__actions">
            <SettingsButton
              variant="ghost"
              onClick={handleRefresh}
              disabled={isRefreshing || isLoading}
              aria-label={CommonStrings.ACTIONS.REFRESH}
              icon={RefreshCw}
              className={isRefreshing ? 'system-info-page__spinning' : ''}
              data-analytics-id="as-system-info-refresh"
            >
              {CommonStrings.ACTIONS.REFRESH}
            </SettingsButton>
            <SettingsButton
              variant="ghost"
              onClick={handleCopy}
              disabled={!systemInfo || isLoading}
              aria-label={SystemInfoStrings.PAGE.COPY_CLIPBOARD_ARIA}
              icon={Copy}
              data-analytics-id="as-system-info-copy"
            >
              {SystemInfoStrings.PAGE.COPY}
            </SettingsButton>
            <SettingsButton
              variant="ghost"
              onClick={handleDownload}
              disabled={!systemInfo || isLoading}
              aria-label={CommonStrings.ACTIONS.DOWNLOAD}
              icon={Download}
              data-analytics-id="as-system-info-download"
            >
              {CommonStrings.ACTIONS.DOWNLOAD}
            </SettingsButton>
          </Flex>
        }
      />

      {/* Error Alert */}
      {error && (
        <Box className="system-info-page__alerts">
          <SettingsAlert type="error" onClose={clearError}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* HA Node Selector */}
      {isHAMode && nodes.length > 1 && (
        <NodeSelector
          nodes={nodes}
          selectedNode={selectedNode}
          onNodeSelect={handleNodeSelect}
        />
      )}

      {/* Navigation Bar */}
      {sections.length > 0 && (
        <Flex
          align="center"
          justify="between"
          className="system-info-page__nav"
          data-testid="system-info-nav"
        >
          {/* Jump to Section Dropdown */}
          <Flex align="center" gap="2">
            <Text size="2" weight="medium">{SystemInfoStrings.PAGE.JUMP_TO}</Text>
            <Select.Root
              value=""
              onValueChange={handleJumpToSection}
            >
              <Select.Trigger
                placeholder={SystemInfoStrings.PAGE.SELECT_SECTION_PLACEHOLDER}
                data-testid="system-info-jump-to"
              >
                <Flex align="center" gap="2">
                  <ChevronsUpDown size={14} />
                  <span>{SystemInfoStrings.PAGE.SELECT_SECTION}</span>
                </Flex>
              </Select.Trigger>
              <Select.Content>
                {sections.map(([key]) => (
                  <Select.Item key={key} value={key}>
                    {formatSectionTitle(key)}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
          </Flex>

          {/* Expand/Collapse Buttons */}
          <Flex gap="2">
            <SettingsButton
              variant="ghost"
              onClick={handleExpandAll}
              aria-label={SystemInfoStrings.PAGE.EXPAND_ALL_SECTIONS_ARIA}
              icon={ChevronsDownUp}
              data-testid="system-info-expand-all"
            >
              {SystemInfoStrings.PAGE.EXPAND_ALL}
            </SettingsButton>
            <SettingsButton
              variant="ghost"
              onClick={handleCollapseAll}
              aria-label={SystemInfoStrings.PAGE.COLLAPSE_ALL_SECTIONS_ARIA}
              icon={ChevronsUpDown}
              data-testid="system-info-collapse-all"
            >
              {SystemInfoStrings.PAGE.COLLAPSE_ALL}
            </SettingsButton>
          </Flex>
        </Flex>
      )}

      {/* Content */}
      <Box className="system-info-page__content">
        {sections.length === 0 ? (
          <Text size="2" className="system-info-page__empty">
            {SystemInfoStrings.PAGE.NO_SYSTEM_INFORMATION_AVAILABLE}
          </Text>
        ) : (
          sections.map(([key, data]) => (
            <SystemInfoSection
              key={key}
              ref={(el) => { sectionRefs.current[key] = el; }}
              title={formatSectionTitle(key)}
              data={data}
              open={expandedSections.has(key)}
              onToggle={(isOpen) => handleSectionToggle(key, isOpen)}
            />
          ))
        )}
      </Box>

      {/* Help Section */}
      <Box className="system-info-page__help">
        <Flex align="center" gap="2" className="system-info-page__help-header">
          <Info size={16} />
          <Text size="2" weight="medium">{SystemInfoStrings.PAGE.ABOUT_SYSTEM_INFORMATION}</Text>
        </Flex>
        <Text size="2" className="system-info-page__help-text">
          {SystemInfoStrings.PAGE.PAGE_DISPLAYS_DETAILED_INFORMATION_ABOUT}
        </Text>
        <Text size="2" className="system-info-page__help-text">
          {SystemInfoStrings.PAGE.SEE_DOCUMENTATION(
            <a
              href="https://help.globalart.dev/en/system-information.html"
              target="_blank"
              rel="noopener noreferrer"
              className="system-info-page__help-link"
            >
              {SystemInfoStrings.PAGE.DOCUMENTATION_LINK}
              <ExternalLink size={12} aria-hidden="true" />
            </a>
          )}
        </Text>
      </Box>
    </Box>
  );
}

export default SystemInfoPage;
