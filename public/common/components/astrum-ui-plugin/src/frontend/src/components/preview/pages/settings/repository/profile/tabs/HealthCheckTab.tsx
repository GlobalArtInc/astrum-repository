

import React, { useState, useCallback, useEffect } from 'react';
import { Box, Flex, IconButton, Tooltip } from '@radix-ui/themes';
import { Maximize2, Minimize2 } from 'lucide-react';
import { SecurityReportPage } from '../../../../../shared/security/SecurityReportPage';

import './HealthCheckTab.scss';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';

export interface HealthCheckTabProps {
  repositoryName: string;
}

/**
 * HealthCheckTab - Full Health Check report in a tab, with fullscreen toggle.
 * Reuses SecurityReportPage content; same pattern as Structure tab.
 */
export function HealthCheckTab({ repositoryName }: HealthCheckTabProps): JSX.Element {
  const [isFullscreen, setIsFullscreen] = useState(false);

  const toggleFullscreen = useCallback(() => {
    setIsFullscreen((prev) => !prev);
  }, []);

  useEffect(() => {
    if (isFullscreen) {
      window.dispatchEvent(new CustomEvent('as-sidebar-toggle', { detail: { open: true } }));
    } else {
      window.dispatchEvent(new CustomEvent('as-sidebar-toggle', { detail: { open: false } }));
    }
  }, [isFullscreen]);

  return (
    <Box
      className={`health-check-tab-container ${isFullscreen ? 'health-check-tab-container--fullscreen' : ''}`}
    >
      <Flex gap="3" align="center" className="health-check-tab-container__toolbar">
        <Tooltip content={isFullscreen ? RepositoryProfileStrings.HEALTH_CHECK_TAB.EXIT_FULLSCREEN : RepositoryProfileStrings.HEALTH_CHECK_TAB.FULLSCREEN_MODE}>
          <IconButton variant="soft" size="2" onClick={toggleFullscreen} aria-label={isFullscreen ? RepositoryProfileStrings.HEALTH_CHECK_TAB.EXIT_FULLSCREEN_2 : RepositoryProfileStrings.HEALTH_CHECK_TAB.ENTER_FULLSCREEN}>
            {isFullscreen ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
          </IconButton>
        </Tooltip>
      </Flex>
      <Box className="health-check-tab-container__content">
        <SecurityReportPage repositoryName={repositoryName} reportType="health-check" embedded />
      </Box>
    </Box>
  );
}

export default HealthCheckTab;
