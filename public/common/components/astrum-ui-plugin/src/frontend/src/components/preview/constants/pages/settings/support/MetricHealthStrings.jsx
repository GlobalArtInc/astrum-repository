import React from 'react';
import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  DETAIL: {
    SELECT_HEALTH_CHECK_VIEW_DETAILS: 'Select a health check to view details',
    HEALTHY: 'Healthy',
    UNHEALTHY: 'Unhealthy',
    UNKNOWN: 'Unknown',
    MESSAGE: 'Message',
    ERROR: 'Error',
    DURATION_MS: (duration) => `Duration: ${duration}ms`,
    LAST_CHECKED: (Date) => `Last checked: ${Date}`,
    NO_ADDITIONAL_DETAILS_AVAILABLE_HEALTH: 'No additional details available for this health check.',
  },
  LIST: {
    HEALTHY_COUNT: (healthy) => `${healthy} healthy`,
    UNHEALTHY_COUNT: (unhealthy) => `${unhealthy} unhealthy`,
    UNKNOWN_COUNT: (unknown) => `${unknown} unknown`,
    FILTER_HEALTH_CHECKS_PLACEHOLDER: 'Filter health checks...',
    NO_MATCHING_HEALTH_CHECKS_FOUND: 'No matching health checks found',
    NO_HEALTH_CHECKS_AVAILABLE: 'No health checks available',
  },
  PAGE: {
    DOCUMENTATION_LINK: 'documentation',
    SEE_DOCUMENTATION: (link) => <>See our {link} for more information.</>,
    HEALTH_CHECKS_REFRESHED: 'Health checks refreshed',
    HEALTH_CHECK_DATA_DOWNLOADED: 'Health check data downloaded',
    LOADING_HEALTH_CHECKS: 'Loading health checks...',
    METRIC_HEALTH: 'Metric Health',
    VIEW_SYSTEM_HEALTH_CHECKS_DIAGNOSTICS: 'View system health checks and diagnostics',
    YOU_DO_NOT_HAVE_PERMISSION: 'You do not have permission to view metric health.',
    SELECT_NODE_VIEW_HEALTH_CHECKS: 'Select a node to view health checks',
    NO_CLUSTER_NODES_AVAILABLE: 'No cluster nodes available',
    HEALTH_CHECKS: (selectedNode) => `Health checks for ${selectedNode}`,
    NO_HEALTH_CHECKS_AVAILABLE: 'No health checks available',
    ABOUT_STATUS: 'About Status',
    PAGE_DISPLAYS_STATUS_VARIOUS_HEALTH: 'This page displays the status of various health checks that monitor critical system components. Red indicators show components that may require attention.',
  },
  API: {
    FAILED_LOAD_METRIC_HEALTH_DATA: 'Failed to load metric health data',
    FAILED_LOAD_NODE_METRIC_HEALTH: 'Failed to load node metric health data',
  },
}, 'PREVIEW.METRIC_HEALTH');
