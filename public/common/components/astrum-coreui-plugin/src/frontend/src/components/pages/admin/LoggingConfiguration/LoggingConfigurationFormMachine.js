
import {assign} from 'xstate';
import Axios from 'axios';
import { mergeDeepRight } from 'ramda';
import {ExtJS, FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const loggerUrl = (name) => `/service/rest/internal/ui/loggingConfiguration/${name}`;

export default FormUtils.buildFormMachine({
  id: 'LoggingConfigurationFormMachine',
  config: (config) => mergeDeepRight(config,{
    states: {
      loaded: {
        on: {
          SAVE: [
            {
              target: 'saving',
              cond: 'isEdit'
            },
            {
              target: 'confirmSave'
            }
          ],
          CANCEL: {
            actions: ['onCancel', 'clearDirtyFlag']
          },

          RESET: {
            target: 'confirmReset',
            cond: 'isEdit'
          }
        }
      },
      confirmReset: {
        invoke: {
          src: 'confirmReset',
          onDone: 'resetting',
          onError: 'loaded'
        }
      },
      confirmSave: {
        invoke: {
          src: 'confirmSave',
          onDone: 'saving',
          onError: 'loaded'
        }
      },
      resetting: {
        invoke: {
          src: 'reset',
          onDone: {
            actions: ['onReset', 'clearDirtyFlag']
          },
          onError: {
            target: 'resetError',
            actions: ['setResetError']
          }
        }
      },
      resetError: {
        on: {
          SAVE: {
            target: 'confirmReset',
            actions: ['clearResetError']
          }
        }
      }
    }
  })
}).withConfig({
  actions: {
    clearResetError: assign({resetError: null}),
    setResetError: assign({resetError: (_, event) => event?.data?.message}),

    validate: assign({
      validationErrors: ({data}) => ({
        name: ValidationUtils.validateLoggerName(data.name)
      })
    })
  },
  guards: {
    isEdit: ({pristineData}) => ValidationUtils.notBlank(pristineData.name)
  },
  services: {
    fetchData: ({pristineData}) => {
      if (ValidationUtils.notBlank(pristineData.name)) { // Edit logging configuration
        return Axios.get(loggerUrl(pristineData.name));
      }
      else { // New Logging Configuration
        return Promise.resolve({
          data: {
            name: '',
            level: 'INFO'
          }
        });
      }
    },
    confirmReset: () => ExtJS.requestConfirmation({
      title: UIStrings.LOGGING.CONFIRM_RESET.TITLE,
      message: UIStrings.LOGGING.CONFIRM_RESET.MESSAGE,
      yesButtonText: UIStrings.LOGGING.CONFIRM_RESET.CONFIRM_BUTTON,
      noButtonText: UIStrings.SETTINGS.CANCEL_BUTTON_LABEL
    }),
    confirmSave: async ({data}) => {
      const response = await Axios.get(loggerUrl(data.name));
      if (response.data.override) {
        return await ExtJS.requestConfirmation({
          title: UIStrings.LOGGING.CONFIRM_UPDATE.TITLE,
          message: UIStrings.LOGGING.CONFIRM_UPDATE.MESSAGE({name: data.name, level: data.level}),
          yesButtonText: UIStrings.LOGGING.CONFIRM_UPDATE.CONFIRM_BUTTON,
          noButtonText: UIStrings.SETTINGS.CANCEL_BUTTON_LABEL
        });
      }
      return Promise.resolve();
    },
    reset: ({data}) => Axios.post(`${loggerUrl(data.name)}/reset`),
    saveData: ({data}) => Axios.put(loggerUrl(data.name), data)
  }
});
