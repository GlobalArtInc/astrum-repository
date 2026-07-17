
import {assign} from 'xstate';
import Axios from 'axios';

import {ExtJS, ListMachineUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

export default ListMachineUtils.buildListMachine({
  id: 'LoggingConfigurationListMachine',
  sortableFields: ['name', 'level'],
  config: (config) => ({
    ...config,
    states: {
      ...config.states,
      loading: {
        ...config.states.loading,
        states: {
          ...config.states.loading.states,
          resetting: {
            invoke: {
              src: 'reset',
              onDone: 'fetch',
              onError: {
                target: 'fetch',
                actions: ['showResetError']
              }
            }
          }
        }
      },
      loaded: {
        ...config.states.loaded,
        on: {
          ...config.states.loaded.on,
          RESET: 'confirmReset'
        }
      },
      confirmReset: {
        invoke: {
          src: 'confirmReset',
          onDone: '#loading.resetting',
          onError: 'loaded'
        }
      }
    }
  })
}).withConfig({
  actions: {
    filterData: assign({
      data: ({filter, data, pristineData}, _) => pristineData.filter(
          ({name}) => name.toLowerCase().indexOf(filter.toLowerCase()) !== -1
      )
    }),
    sortData: assign({
      data: ListMachineUtils.sortDataByFieldAndDirection({useLowerCaseSorting: false})
    }),
    showResetError: (_, {data}) => {
      console.error(data.response);
      ExtJS.showErrorMessage(UIStrings.LOGGING.MESSAGES.RESET_ERROR)
    }
  },
  services: {
    fetchData: () => Axios.get('service/rest/internal/ui/loggingConfiguration'),
    confirmReset: () => ExtJS.requestConfirmation({
      title: UIStrings.LOGGING.CONFIRM_RESET_ALL.TITLE,
      message: UIStrings.LOGGING.CONFIRM_RESET_ALL.MESSAGE,
      yesButtonText: UIStrings.LOGGING.CONFIRM_RESET_ALL.CONFIRM_BUTTON,
      noButtonText: UIStrings.SETTINGS.CANCEL_BUTTON_LABEL
    }),
    reset: () => Axios.post('service/rest/internal/ui/loggingConfiguration/reset')
  }
});
