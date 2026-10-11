import React from 'react';
import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  NODE_SELECTOR: {
    SELECT_NODE: 'Select Node',
    LOCAL_BADGE: 'local',
  },
  MACHINE: {
    FAILED_LOAD_SYSTEM_INFORMATION: 'Failed to load system information',
    ERROR_OCCURRED: 'An error occurred',
    FAILED_REFRESH_SYSTEM_INFORMATION: 'Failed to refresh system information',
  },
  PAGE: {
    DOCUMENTATION_LINK: 'documentation',
    SEE_DOCUMENTATION: (link) => <>See our {link} for more information.</>,
    LOADING_SYSTEM_INFORMATION: 'Loading system information...',
    SYSTEM_INFORMATION: 'System Information',
    VIEW_DETAILED_SYSTEM_SERVER_INFORMATION: 'View detailed system and server information',
    YOU_DO_NOT_HAVE_PERMISSION: 'You do not have permission to view system information.',
    COPY_CLIPBOARD_ARIA: 'Copy to clipboard',
    COPY: 'Copy',
    JUMP_TO: 'Jump to:',
    SELECT_SECTION_PLACEHOLDER: 'Select section...',
    SELECT_SECTION: 'Select section...',
    EXPAND_ALL_SECTIONS_ARIA: 'Expand all sections',
    EXPAND_ALL: 'Expand All',
    COLLAPSE_ALL_SECTIONS_ARIA: 'Collapse all sections',
    COLLAPSE_ALL: 'Collapse All',
    NO_SYSTEM_INFORMATION_AVAILABLE: 'No system information available',
    ABOUT_SYSTEM_INFORMATION: 'About System Information',
    PAGE_DISPLAYS_DETAILED_INFORMATION_ABOUT: 'This page displays detailed information about your Astrum Repository instance, including version information, system properties, and runtime configuration. Use the Download button to save this information for support requests.',
  },
  SECTION: {
    FILTER_PROPERTIES_PLACEHOLDER: 'Filter properties...',
    NO_MATCHING_PROPERTIES_FOUND: 'No matching properties found',
    NO_PROPERTIES_AVAILABLE: 'No properties available',
  },
  TYPES: {
    YES: 'Yes',
    NO: 'No',
  },
  HOOK: {
    SYSTEM_INFORMATION_REFRESHED: 'System information refreshed',
    SYSTEM_INFORMATION_DOWNLOADED: 'System information downloaded',
    COPIED_CLIPBOARD: 'Copied to clipboard',
    FAILED_COPY_CLIPBOARD: 'Failed to copy to clipboard',
  },
  API: {
    FAILED_LOAD_SYSTEM_INFORMATION: 'Failed to load system information',
    FAILED_LOAD_HA_SYSTEM_INFORMATION: 'Failed to load HA system information',
  },
}, 'PREVIEW.SYSTEM_INFO');
