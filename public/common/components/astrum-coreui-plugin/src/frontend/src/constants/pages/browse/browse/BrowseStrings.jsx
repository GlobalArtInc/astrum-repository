
import { faDatabase } from '@fortawesome/free-solid-svg-icons';

export default {
  BROWSE: {
    MENU: {
      text: 'Browse',
      description: 'Browse assets and components',
      icon: faDatabase
    },
    TREE_TITLE: 'View Assets/Components for',
    TREE_EMPTY_MESSAGE: 'No component/assets found in repository',
    LIST: {
      COLUMNS: {
        NAME: 'Name',
        TYPE: 'Type',
        FORMAT: 'Format',
        STATUS: 'Status',
        URL: 'URL',
        HEALTH_CHECK: 'Health Check',
        IQ_POLICY_VIOLATIONS: 'Firewall Report',
      },
      FILTER_PLACEHOLDER: 'Filter by name',
      EMPTY_MESSAGE: 'There are no repositories available',
      COPY_URL_TITLE: 'Copy URL to Clipboard',
      URL_COPIED_MESSAGE: 'URL Copied to Clipboard',
      URL_COPY_ERROR_MESSAGE: 'Failed to copy URL to clipboard'
    }
  }
};
