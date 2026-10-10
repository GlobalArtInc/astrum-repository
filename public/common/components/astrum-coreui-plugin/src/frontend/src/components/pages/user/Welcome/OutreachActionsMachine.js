
import {assign, Machine} from 'xstate';

const SHOW_CONNECT_ACTION_KEY = 'as-welcome-show-connect-action';
const SHOW_CONNECT_ACTION_DEFAULT = true;

export default Machine({
  id: 'OutreachActionsMachine',
  initial: 'readingShowConnectAction',
  context: {
    showConnectAction: SHOW_CONNECT_ACTION_DEFAULT,
    url: '',
  },

  states: {
    idle: {
      on: {
        REDIRECT: {
          target: 'redirecting',
          actions: ['setUrl'],
        },
        OPEN: {
          target: 'idle',
          actions: ['open'],
        },
        OPEN_CONNECT_MODAL: {
          target: 'showingConnectModal',
        },
      },
    },
    redirecting: {
      type: 'final',
      invoke: {
        src: 'redirect',
      },
    },
    readingShowConnectAction: {
      invoke: {
        src: 'readShowConnectAction',
      },
    },
    savingShowConnectAction: {
      invoke: {
        src: 'saveShowConnectAction',
      },
    },
    showingConnectModal: {
      on: {
        CLOSE_MODAL: {
          target: 'savingShowConnectAction',
        }
      }
    },
  },
  on: {
    SET_DATA: {
      target: 'idle',
      actions: ['setShowConnectAction'],
    }
  },
}, {
  actions: {
    setShowConnectAction: assign({
      showConnectAction: (_, event) => event.showConnectAction,
    }),
    setUrl: assign({
      url: (_, event) => event.url,
    }),
    open: (_, event) => window.open(event.url, '_blank'),
  },
  services: {
    readShowConnectAction: () => callback => {
      const showConnectAction = localStorage.getItem(SHOW_CONNECT_ACTION_KEY);
      callback({
        type: 'SET_DATA',
        showConnectAction: showConnectAction ? JSON.parse(showConnectAction) : SHOW_CONNECT_ACTION_DEFAULT,
      });
    },
    saveShowConnectAction: () => callback => {
      localStorage.setItem(SHOW_CONNECT_ACTION_KEY, JSON.stringify(false));
      callback({ type: 'SET_DATA', showConnectAction: false});
    },
    redirect: ({url}) => {
      window.location.hash = url;
    },
  }
});
