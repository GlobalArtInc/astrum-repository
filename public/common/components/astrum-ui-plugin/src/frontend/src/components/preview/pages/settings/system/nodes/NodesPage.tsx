

import React from 'react';
import { Box, Text } from '@radix-ui/themes';
import { Server, ExternalLink } from 'lucide-react';

import { ExtJS } from '../../../../../../interface/ExtJS';
import { PageHeader } from '../../../../shared';
import { SettingsFormSection, SettingsAlert } from '../../../../shared/form';
import { NodesList } from './NodesList';
import { NodesPageProps } from './types';

import './NodesPage.scss';

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
        title="Nodes"
        description="View cluster nodes in this Astrum Repository instance"
        breadcrumbs={[
          { label: 'Settings', onClick: () => navigateTo('#preview/admin/settings') },
          { label: 'Nodes' }
        ]}
      />

      {/* Content */}
      <Box className="nodes-page__content">
        {/* Single-node informational banner */}
        {!isClustered && (
          <Box className="nodes-page__banner">
            <SettingsAlert type="info" data-testid="single-node-banner">
              This instance is running in single-node mode. High Availability cluster nodes are not expected. See the{' '}
              <a
                href={HA_DOCUMENTATION_URL}
                target="_blank"
                rel="noopener noreferrer"
              >
                high availability documentation
                <ExternalLink size={12} aria-hidden="true" />
              </a>
              {' '}for more information.
            </SettingsAlert>
          </Box>
        )}

        {/* Nodes List Section */}
        <SettingsFormSection title="Nodes">
          <Text size="2" className="nodes-page__section-description">
            Nodes participating in this Astrum Repository instance.
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
