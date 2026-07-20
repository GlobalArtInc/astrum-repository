
import {assign} from 'xstate';
import Axios from 'axios';
import {mergeDeepRight} from 'ramda';
import {FormUtils, ExtJS, APIConstants} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {REALMS: {MESSAGES}} = UIStrings;

const NX_AUTHENTICATING_REALM = 'AstrumAuthenticatingRealm';
const NX_AUTHORIZING_REALM = 'AstrumAuthorizingRealm';
const SSO_REALMS = [UIStrings.REALMS.SSO_REALMS.SAML, UIStrings.REALMS.SSO_REALMS.OAUTH2];

export default FormUtils.buildFormMachine({
  id: 'RealmsMachine',
  config: (config) => mergeDeepRight(config, {
    states: {
      loaded: {
        on: {
          CHECK_LOCAL_REALM_REMOVAL: [
            {
              target: 'showLocalRealmRemovalModal',
              cond: 'isLocalRealmRemoval'
            },
            {
              target: 'saving',
              cond: 'canSave'
            }
          ]
        }
      },
      showLocalRealmRemovalModal: {
        on: {
          SAVE: {
            target: 'saving',
            cond: 'canSave'
          },
          CLOSE: {
            target: 'loaded'
          }
        }
      }
    }
  })
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data}) => ({
        active: data.active.length ? null : MESSAGES.NO_REALMS_CONFIGURED,
        multipleSsoEnabled: SSO_REALMS.filter(realm => data.active.includes(realm)).length > 1
            ? MESSAGES.MULTIPLE_SSO_ENABLED + SSO_REALMS
              .filter(realm => data.active.includes(realm))
              .map(realmId => data.available.find(r => r.id === realmId)?.name || realmId)
              .join(', ')
            : null
      }),
    }),
    setData: assign((_, event) => {
      const response = event.data || [];
      const available = response[0]?.data || [];
      const active = response[1]?.data || [];
      const data = {
        available,
        active,
      };

      return {
        data,
        pristineData: data,
      };
    }),
  },
  guards: {
    isLocalRealmRemoval: ({data, pristineData}) => 
      (pristineData.active?.includes(NX_AUTHENTICATING_REALM) && 
        !data.active?.includes(NX_AUTHENTICATING_REALM)) ||
      (pristineData.active?.includes(NX_AUTHORIZING_REALM) && 
        !data.active?.includes(NX_AUTHORIZING_REALM))
  },
  services: {
    fetchData: async () => {
      await ExtJS.waitForPermissions();
      return Axios.all([
        Axios.get(APIConstants.REST.PUBLIC.AVAILABLE_REALMS),
        Axios.get(APIConstants.REST.PUBLIC.ACTIVE_REALMS),
      ]);
    },
    saveData: ({ data }) =>
      Axios.put(APIConstants.REST.PUBLIC.ACTIVE_REALMS, data.active),
  },
});
