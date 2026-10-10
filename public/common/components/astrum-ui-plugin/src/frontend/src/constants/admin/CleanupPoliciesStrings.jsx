
import React from 'react';
import {AsTextLink} from '@globalart/astrum-react-shared-components';
import { faBroom } from '@fortawesome/free-solid-svg-icons';

export default {
  CLEANUP_POLICIES: {
    MENU: {
      text: 'Cleanup Policies',
      description: 'Manage component removal configuration',
      icon: faBroom
    },

    CREATE_TITLE: 'Create Cleanup Policy',
    EDIT_TITLE: 'Edit Cleanup Policy',
    SUB_TITLE: 'Define Cleanup Policy',
    DESCRIPTION: <>
      <b>Cleanup policies</b> allow you to automatically delete unused
      components and free up storage space in your Astrum repository
      instance. You can schedule cleanup policies to run at specific
      intervals and define <b>cleanup criteria</b> for selecting
      components to delete.
    </>,

    HELP_TITLE: 'What is a cleanup policy?',
    HELP_TEXT: <>
      Cleanup policies can be used to remove content from your repositories. These policies will execute
      at the configured frequency. Once created, a cleanup policy must be assigned to a repository from{' '}
      <AsTextLink href="#admin/repository/repositories">the repository configuration screen</AsTextLink>.
      For more information, check{' '}
      <AsTextLink external href="http://links.globalart.dev/products/astrum/docs/cleanup-policy">
        the documentation
      </AsTextLink>.
    </>,
    EMPTY_MESSAGE: 'No cleanup policies were found',
    CREATE_BUTTON: 'Create Cleanup Policy',
    FILTER_PLACEHOLDER: 'Filter',

    NAME_LABEL: 'Name',
    FORMAT_LABEL: 'Format',
    DESCRIPTION_LABEL: 'Description',
    CRITERIA_LABEL: 'Cleanup Criteria',
    CRITERIA_DESCRIPTION: 'Remove all components that match all selected criteria',
    LAST_UPDATED_CHECKBOX_TITLE: (enabled) => `${enabled ? 'Disable' : 'Enable'} Component Age Criterion`,
    LAST_UPDATED_LABEL: 'Component Age',
    LAST_DOWNLOADED_CHECKBOX_TITLE: (enabled) => `${enabled ? 'Disable' : 'Enable'} Component Usage Criterion`,
    LAST_DOWNLOADED_LABEL: 'Component Usage',
    RELEASE_TYPE_CHECKBOX_TITLE: (enabled) => `${enabled ? 'Disable' : 'Enable'} Release Type Criterion`,
    RELEASE_TYPE_LABEL: 'Release Type',
    ASSET_NAME_CHECKBOX_TITLE: (enabled) => `${enabled ? 'Disable' : 'Enable'} Asset Name Matcher Criterion`,
    ASSET_NAME_LABEL: 'Asset Name Matcher',
    FORMAT_SELECT: 'Select a format...',
    RELEASE_TYPE_SELECT: 'Remove components that are of the following release type:',
    REPOSITORY_SELECT: 'Select a repository...',

    NAME_DESCRIPTION: 'Use a unique name for the cleanup policy',
    FORMAT_DESCRIPTION: 'The format that this cleanup policy can be applied to',
    RELEASE_TYPE_DESCRIPTION: 'Remove components that are of the following release type:',
    ASSET_NAME_DESCRIPTION: 'Remove components that have at least one asset name matching the following regular expression pattern:',

    LAST_UPDATED_SUB_LABEL: 'Components published over “x” days ago (e.g 1-9999)',
    LAST_DOWNLOADED_SUB_LABEL: 'Components downloaded in “x” amount of days (e.g 1-9999)',

    PLACEHOLDER: 'e.g 100 days',

    RELEASE_TYPE: {
      RELEASES_AND_SNAPSHOT: {
        id: '',
        label: 'Releases & Pre-Releases/Snapshots'
      },
      RELEASES: {
        id: 'RELEASES',
        label: 'Releases'
      },
      PRERELEASES: {
        id: 'PRERELEASES',
        label: 'Pre-Releases/Snapshots'
      },
    },

    EXCLUSION_CRITERIA: {
      LABEL: 'Except, do not remove any component that meets the following criterion:',
      ALERT: 'This option is only applicable to releases',
      ADDITIONAL_CRITERIA_ALERT: 'Select at least one other criterion to enable this option.',
      NORMALIZED_VERSION_ALERT: 'Exclusion criteria will populate once post-upgrade tasks complete',
      VERSION_LABEL: 'Number of Versions',
      SUB_LABEL: (sortBy, labels) => {
        const label = labels[sortBy.toUpperCase()] ? labels[sortBy.toUpperCase()].label : labels.VERSION.label;
        return `Keep the latest "x" number of versions by ${label}.`
      },
      SORT_BY: {
        VERSION: {
          id: 'version',
          label: 'version number',
          format: 'maven2'
        },
        DATE: {
          id: 'date',
          label: 'component age',
          format: 'docker'
        },
      },
    },

    PREVIEW: {
      TITLE: 'Cleanup policy preview',
      REPOSITORY_LABEL: 'Preview Repository',
      REPOSITORY_DESCRIPTION: 'Select a repository to preview what might get cleaned up if this policy was applied',
      INVALID_QUERY: 'The filter is invalid so results could not be retrieved.',
      BUTTON: 'Preview',
      RESULTS: 'Preview Results',
      NAME_COLUMN: 'Name',
      GROUP_COLUMN: 'Group',
      VERSION_COLUMN: 'Version',
      EMPTY: 'No assets in repository matched the criteria',
      COMPONENT_COUNT: (actual, total) => {
        if (total === 0) {
          return 'Criteria matched no components.';
        }
        else if (total > 0) {
          return `Component count (matching criteria) viewing ${actual} out of ${total}.`;
        }
        return `Component count (matching criteria) viewing first ${actual}`;
      },
      SAMPLE_WARNING: 'Results may only be a sample of what will be deleted using the current criteria.',
      BUTTON_TOOLTIP: 'Please select a repository and at least one cleanup criterion',
    },

    DRY_RUN: {
      BUTTON: 'Generate CSV Report',
      REPOSITORY_DESCRIPTION: 'Export a spreadsheet listing which components would be deleted today based on selected cleanup policy settings. Your export will list component namespaces, names, versions and paths.',
      REPOSITORY_SELECT: 'Select a repository',
      TITLE: 'Preview Cleanup Policy Results',
      BUTTON_TOOLTIP: 'Please select a repository and at least one cleanup criterion',
    },

    MESSAGES: {
      NO_CRITERIA_ERROR: 'At least one criterion must be selected',
      SAVE_ERROR: 'An error occurred while saving the cleanup policy',
      DELETE_ERROR: (name) => `Cleanup policy ${name} is in use and cannot be deleted`,

      CONFIRM_DELETE: {
        TITLE: 'Delete cleanup policy',
        MESSAGE: (inUseCount) => inUseCount ?
            `This Cleanup Policy is used by ${inUseCount} repositories` :
            'This Cleanup Policy is not used by any repository',
        YES: 'Delete',
        NO: 'Cancel'
      }
    }
  }
};
