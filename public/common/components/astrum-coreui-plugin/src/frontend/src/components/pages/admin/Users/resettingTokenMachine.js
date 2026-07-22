
import {sendParent} from 'xstate';
import {mergeDeepRight} from 'ramda';
import Axios from 'axios';
import {ExtJS, FormUtils} from '@globalart/astrum-ui-plugin';
import {URL} from './UsersHelper';
import UIStrings from '../../../../constants/UIStrings';

const {
  USERS: {TOKEN: LABELS},
} = UIStrings;

export default FormUtils.buildFormMachine({
  id: 'resettingTokenMachine',
  initial: 'saving',

  stateAfterSave: 'done',
  config: (config) =>
    mergeDeepRight(config, {
      states: {
        done: {
          type: 'final',
        },
      },
      on: {
        CANCEL: {
          actions: sendParent('CANCEL'),
        },
      },
    }),
}).withConfig({
  actions: {
    logSaveSuccess: ({data}) =>
      ExtJS.showSuccessMessage(LABELS.SAVE_SUCCESS(data.name)),
  },
  services: {
    saveData: ({data}) =>
      Axios.delete(URL.resetTokenUrl(data.userId, data.source)),
  },
});
