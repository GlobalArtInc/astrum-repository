
import Axios from 'axios';

import {ExtJS, TokenMachine} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

export default TokenMachine.withConfig({
      actions: {
        showResetSuccess: () => {
          ExtJS.showSuccessMessage(UIStrings.NUGET_API_KEY.MESSAGES.RESET_SUCCESS)
        },
        showResetError: () => {
          ExtJS.showErrorMessage(UIStrings.NUGET_API_KEY.MESSAGES.RESET_ERROR)
        },
        showAccessError: () => {
          ExtJS.showErrorMessage(UIStrings.NUGET_API_KEY.MESSAGES.ACCESS_ERROR);
        }
      },
      services: {
        resetToken: () => ExtJS.requestAuthenticationToken(UIStrings.NUGET_API_KEY.AUTH_INSTRUCTIONS)
            .then(authToken =>
                Axios.delete(`/service/rest/internal/nuget-api-key?authToken=${btoa(authToken)}`)),
        accessToken: () => ExtJS.requestAuthenticationToken(UIStrings.NUGET_API_KEY.AUTH_INSTRUCTIONS)
            .then(authToken =>
                Axios.get(`/service/rest/internal/nuget-api-key?authToken=${btoa(authToken)}`))
      }
    },
);
