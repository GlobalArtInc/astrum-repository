
import { faMedkit } from '@fortawesome/free-solid-svg-icons';

export default {
  METRIC_HEALTH: {
    MENU: {
      text: 'Status',
      description: 'System status checks',
      detailsDescription: 'View Node specific details',
      icon: faMedkit
    },
    NAME_HEADER: 'Name',
    MESSAGE_HEADER: 'Message',
    ERROR_HEADER: 'Error',
    STATUS_HEADER: 'Status',
    BACK_BUTTON: 'Back to Status Table View',
    EMPTY_NODE_LIST: 'There are no nodes available',
    EMPTY_NODE: 'There is no node available',
    HELP: {
      LABEL: 'What are Nodes?',
      SUB_LABEL:
        'When clustering is enabled, redundant Astrum Repository instances (i.e., nodes) run in active/active mode (i.e., both actively running Astrum Repository simultaneously) within a single cloud region or on-premises data center. This allows you to maintain Astrum Repository availability even if one node becomes unavailable. Node status displayed on this screen is updated at 5-minute intervals.',
    },
  },
};
