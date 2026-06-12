
import { faInfo } from '@fortawesome/free-solid-svg-icons';

export default {
  SYSTEM_INFORMATION: {
    MENU: {
      text: 'System Information',
      description: 'General system information',
      icon: faInfo
    },
    ACTIONS: {
      download: 'Download as JSON',
      nodeSelector: 'Node Selector'
    },
    LOAD_ERROR: 'An error occurred while fetching the system information'
  }
};