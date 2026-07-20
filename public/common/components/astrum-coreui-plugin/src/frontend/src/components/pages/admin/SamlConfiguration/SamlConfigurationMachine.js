
import Axios from 'axios';
import {assign} from 'xstate';

import {ExtJS, FormUtils, ValidationUtils, APIConstants} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {ERROR, SAML_CONFIGURATION} = UIStrings;

const SAML_API_URL = APIConstants.REST.INTERNAL.SAML; 

export default FormUtils.buildFormMachine({
  id: 'SamlConfigurationForm'
}).withConfig({
  actions: {
    setData: assign({
      data: (_, {data}) => mapSamlResponse(data),
      pristineData: (_, {data}) => mapSamlResponse(data)
    }),

    logSaveSuccess: () => ExtJS.showSuccessMessage(SAML_CONFIGURATION.MESSAGES.SAVE_SUCCESS),

    validate: assign({
      validationErrors: ({data}) => ({
        idpMetadata: ValidationUtils.isBlank(data?.idpMetadata) ? ERROR.FIELD_REQUIRED : null,
        entityIdUri: ValidationUtils.notUri(data?.entityIdUri) ? SAML_CONFIGURATION.FIELDS.entityIdUriValidationError : null,
        usernameAttr: ValidationUtils.isBlank(data?.usernameAttr) ? ERROR.FIELD_REQUIRED : null
      })
    })
  },
  services:
      {
        fetchData: () => Axios.get(SAML_API_URL),
        saveData: ({data}) => {
          const {
            validateResponseSignature,
            validateAssertionSignature,
            usernameAttr,
            firstNameAttr,
            lastNameAttr,
            emailAttr,
            roleAttr
          } = data;
          return Axios.put(SAML_API_URL, {
            ...data,
            validateResponseSignature: validateResponseSignature === 'default' ? null : data.validateResponseSignature,
            validateAssertionSignature: validateAssertionSignature === 'default' ? null : data.validateAssertionSignature,
            usernameAttr: usernameAttr.trim(),
            firstNameAttr: firstNameAttr.trim(),
            lastNameAttr: lastNameAttr.trim(),
            emailAttr: emailAttr.trim(),
            roleAttr: roleAttr.trim()
          });
        }
      }
});

/**
 * Returns a modified data object where Boolean properties validateResponseSignature and validateAssertionSignature are
 * handled as strings
 */
function mapSamlResponse({data}) {
  ['validateResponseSignature', 'validateAssertionSignature'].forEach(function(item) {
    data[item] = data[item] === null ? 'default' : data[item].toString();
  });
  return data;
}
