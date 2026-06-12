
import { faArchive } from '@fortawesome/free-solid-svg-icons';

export default {
  NODES: {
    MENU: {
      text: 'Nodes',
      description: 'See your available nodes',
      icon: faArchive
    },
    HELP: {
      TITLE: 'What are Nodes?',
      TEXT: 'When clustering is enabled, redundant Astrum Repository instances (i.e., nodes) run in active/active mode (i.e., both actively running Astrum Repository simultaneously) within a single cloud region or on-premises data center. This allows you to maintain Astrum Repository availability even if one node becomes unavailable. Only active Nodes will appear here.'
    }
  }
};
