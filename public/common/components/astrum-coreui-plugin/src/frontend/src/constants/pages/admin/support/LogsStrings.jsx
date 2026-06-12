
import React from 'react';
import { faTerminal } from '@fortawesome/free-solid-svg-icons';

export default {
  LOGS: {
    MENU: {
      text: 'Logs',
      description: 'View the current log contents',
      icon: faTerminal
    },

    LIST: {
      FILTER_PLACEHOLDER: 'Filter by File Name',
      LOGGER_NAME_LABEL: 'Logger Name',
      FILE_NAME_LABEL: 'File Name',
      SIZE_LABEL: 'Size',
      LAST_MODIFIED_LABEL: 'Last Modified'
    },

    VIEW: {
      TITLE: (log) => `Viewing ${log}`,
      DOWNLOAD: 'Download',
      MARK: {
        LABEL: 'Marker to insert into log',
        INSERT: 'Insert'
      },
      REFRESH: {
        RATE_LABEL: 'Refresh Rate',
        SIZE_LABEL: 'Size'
      }
    },

    REFRESH: {
      MANUAL_ITEM: 'Manual',
      TWENTY_SECONDS_ITEM: 'Every 20 seconds',
      MINUTE_ITEM: 'Every minute',
      TWO_MINUTES_ITEM: 'Every 2 minutes',
      FIVE_MINUTES_ITEM: 'Every 5 minutes'
    },
    SIZE: {
      LAST25KB_ITEM: 'Last 25KB',
      LAST50KB_ITEM: 'Last 50KB',
      LAST100KB_ITEM: 'Last 100KB'
    }
  }
};