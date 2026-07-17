
import Axios from 'axios';
import {assign} from 'xstate';

import {ExtJS, FormUtils, ValidationUtils, APIConstants} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {ERROR, OAUTH2_CONFIGURATION} = UIStrings;

const OAUTH2_API_URL = APIConstants.REST.INTERNAL.OAUTH2;

// Ensure JSON values for these fields
function parseString(val) {
  if(val) {
    const result = JSON.parse(val);
    if (typeof result === 'object') {
      return result;
    }
  }
  return {};
}

function stringifyObject(val) {
  if (val && typeof val === 'object') {
    try {
      return JSON.stringify(val, null, 2);
    } catch {
      return '';
    }
  }
  return val || '';
}

export default FormUtils.buildFormMachine({
  id: 'OAuth2ConfigurationForm'
}).withConfig({
  actions: {
    setData: assign({
      data: (_, {data}) => mapOAuth2Response(data),
      pristineData: (_, {data}) => mapOAuth2Response(data)
    }),

    logSaveSuccess: () => ExtJS.showSuccessMessage(OAUTH2_CONFIGURATION.MESSAGES.SAVE_SUCCESS),

    validate: assign({
      validationErrors: ({data}) => {
        const optionalUrl = (value) => ValidationUtils.isBlank(value) ? null : ValidationUtils.validateIsUrl(value);
        const hasManualEndpoints = !ValidationUtils.isBlank(data?.idpAuthorizationUrl) &&
            !ValidationUtils.isBlank(data?.idpTokenUrl) &&
            (!ValidationUtils.isBlank(data?.idpJwksUrl) || !ValidationUtils.isBlank(data?.idpJwks));
        return {
          clientId: ValidationUtils.isBlank(data?.clientId) ? ERROR.FIELD_REQUIRED : null,
          issuerUrl: ValidationUtils.isBlank(data?.issuerUrl)
              ? (hasManualEndpoints ? null : ERROR.FIELD_REQUIRED)
              : ValidationUtils.validateIsUrl(data?.issuerUrl),
          idpAuthorizationUrl: optionalUrl(data?.idpAuthorizationUrl),
          idpLogoutUrl: optionalUrl(data?.idpLogoutUrl),
          idpTokenUrl: optionalUrl(data?.idpTokenUrl),
          idpJwksUrl: optionalUrl(data?.idpJwksUrl)
        };
      }
    })
  },
  services:
      {
        fetchData: () => Axios.get(OAUTH2_API_URL).then(response => {
          const data = response.data;
          // Convert JSON fields back to string for the form

          return {
            ...response,
            data: {
              ...data,
              exactMatchClaims: stringifyObject(data.exactMatchClaims),
              authorizationCustomParams: stringifyObject(data.authorizationCustomParams),
              tokenRequestCustomParams: stringifyObject(data.tokenRequestCustomParams)
            }
          };
        }),
        saveData: ({data}) => {
          const {
            issuerUrl,
            idpJwksUrl,
            idpJwsAlgorithm,
            usernameClaim,
            firstNameClaim,
            lastNameClaim,
            emailClaim,
            groupsClaim,
            exactMatchClaims,
            clientId,
            idpAuthorizationUrl,
            idpLogoutUrl,
            idpTokenUrl,
            authorizationCustomParams,
            tokenRequestCustomParams
          } = data;
          return Axios.put(OAUTH2_API_URL, {
            ...data,
            issuerUrl: issuerUrl?.trim(),
            idpJwksUrl: idpJwksUrl?.trim(),
            idpJwsAlgorithm: idpJwsAlgorithm?.trim(),
            usernameClaim: usernameClaim?.trim(),
            firstNameClaim: firstNameClaim?.trim(),
            lastNameClaim: lastNameClaim?.trim(),
            emailClaim: emailClaim?.trim(),
            groupsClaim: groupsClaim?.trim(),
            clientId: clientId?.trim(),
            idpAuthorizationUrl: idpAuthorizationUrl?.trim(),
            idpLogoutUrl: idpLogoutUrl?.trim(),
            idpTokenUrl: idpTokenUrl?.trim(),
            exactMatchClaims: parseString(exactMatchClaims),
            authorizationCustomParams: parseString(authorizationCustomParams),
            tokenRequestCustomParams: parseString(tokenRequestCustomParams)
          });
        }
      }
});

/**
 * Returns the OAuth2 configuration data object
 */
function mapOAuth2Response({data}) {
  return data || {
    issuerUrl: '',
    idpJwksUrl: '',
    idpJwsAlgorithm: '',
    idpJwks: '',
    usernameClaim: 'preferred_username',
    firstNameClaim: 'given_name',
    lastNameClaim: 'family_name',
    emailClaim: 'email',
    groupsClaim: '',
    exactMatchClaims: {},
    clientId: '',
    clientSecret: '',
    idpAuthorizationUrl: '',
    idpLogoutUrl: '',
    idpTokenUrl: '',
    authorizationCustomParams: {},
    tokenRequestCustomParams: {},
    useTrustStore: false
  };
}
