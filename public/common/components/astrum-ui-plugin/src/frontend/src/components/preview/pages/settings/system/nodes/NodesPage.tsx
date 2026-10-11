

import React from 'react';
import { Box, Text } from '@radix-ui/themes';
import { Server, ExternalLink } from 'lucide-react';

import { ExtJS } from '../../../../../../interface/ExtJS';
import { PageHeader } from '../../../../shared';
import { SettingsFormSection, SettingsAlert } from '../../../../shared/form';
import { NodesList } from './NodesList';
import { NodesPageProps } from './types';

import './NodesPage.scss';
import NodesStrings from '../../../../constants/pages/settings/system/NodesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

const HA_DOCUMENTATION_URL = 'https://links.globalart.dev/products/astrum/high-availability';

/**
 * NodesPage - Main Nodes management page for Preview UI
 *
 * Displays cluster nodes in the system.
 */
export function NodesPage({ className }: NodesPageProps) {
  const isClustered = ExtJS.useState(() => ExtJS.state().getValue('astrum.datastore.clustered.enabled'));

  return (
    <Box className={`nodes-page ${className || ''}`.trim()} data-analytics-id="as-nodes-nav-detail">
      <PageHeader
        icon={Server}
        title={NodesStrings.PAGE.NODES}
        description={NodesStrings.PAGE.VIEW_CLUSTER_NODES_ASTRUM_REPOSITORY}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
          { label: NodesStrings.PAGE.NODES }
        ]}
      />

      {/* Content */}
      <Box className="nodes-page__content">
        {/* Single-node informational banner */}
        {!isClustered && (
          <Box className="nodes-page__banner">
            <SettingsAlert type="info" data-testid="single-node-banner">
              {NodesStrings.PAGE.SINGLE_NODE_MODE(
                <a
                  href={HA_DOCUMENTATION_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  {NodesStrings.PAGE.HIGH_AVAILABILITY_DOCUMENTATION}
                  <ExternalLink size={12} aria-hidden="true" />
                </a>
              )}
            </SettingsAlert>
          </Box>
        )}

        {/* Nodes List Section */}
        <SettingsFormSection title={NodesStrings.PAGE.NODES}>
          <Text size="2" className="nodes-page__section-description">
            {NodesStrings.PAGE.NODES_PARTICIPATING_ASTRUM_REPOSITORY_INSTANCE}
          </Text>
          <Box className="nodes-page__nodes-list">
            <NodesList />
          </Box>
        </SettingsFormSection>
      </Box>
    </Box>
  );
}

export default NodesPage;
