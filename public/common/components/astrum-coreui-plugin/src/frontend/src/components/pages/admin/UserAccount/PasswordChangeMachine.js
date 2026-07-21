
import {assign} from 'xstate';
import {ExtJS, FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';
import Axios from "axios";

import UIStrings from '../../../../constants/UIStrings';

const EMPTY_DATA = {
  passwordCurrent: '',
  passwordNew: '',
  passwordNewConfirm: ''
};

export default FormUtils.buildFormMachine({
  id: 'passwordChange',
  initial: 'loaded',
  config: (config) => ({
    ...config,
    context: {
      ...config.context,
      data: EMPTY_DATA,
      pristineData: EMPTY_DATA
    }
  })
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data: {passwordCurrent, passwordNew, passwordNewConfirm}}) => {
        let passwordNewError = [];
        if (ValidationUtils.isBlank(passwordNew)) {
          passwordNewError.push(UIStrings.ERROR.FIELD_REQUIRED);
        }
        if (passwordCurrent === passwordNew) {
          passwordNewError.push(UIStrings.USER_ACCOUNT.MESSAGES.PASSWORD_MUST_DIFFER_ERROR);
        }

        let passwordNewConfirmError = [];
        if (ValidationUtils.isBlank(passwordNewConfirm)) {
          passwordNewConfirmError.push(UIStrings.ERROR.FIELD_REQUIRED);
        }
        if (passwordNew !== passwordNewConfirm) {
          passwordNewConfirmError.push(UIStrings.ERROR.PASSWORD_NO_MATCH_ERROR);
        }

        return {
          passwordCurrent: ValidationUtils.isBlank(passwordCurrent) ? UIStrings.ERROR.FIELD_REQUIRED : null,
          passwordNew: passwordNewError,
          passwordNewConfirm: passwordNewConfirmError
        };
      }
    }),
    logSaveSuccess: () => ExtJS.showSuccessMessage(UIStrings.USER_ACCOUNT.MESSAGES.PASSWORD_CHANGE_SUCCESS),
    logSaveError: ({saveError, saveErrors = {}}) => {
      let error = saveError ||
          saveErrors['*'] ||
          UIStrings.USER_ACCOUNT.MESSAGES.PASSWORD_CHANGE_ERROR

      ExtJS.showErrorMessage(error);
    },
    onSaveSuccess: assign({
      data: () => EMPTY_DATA,
      pristineData: () => EMPTY_DATA
    })
  },
  services: {
    saveData: ({data: {passwordCurrent, passwordNew}}, {userId}) =>
        ExtJS.fetchAuthenticationToken(userId, passwordCurrent).then((result) =>
            Axios.put(`/service/rest/internal/ui/user/${userId}/password`, {
              authToken: result.data,
              password: passwordNew
            }))
  }
});
