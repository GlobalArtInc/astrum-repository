
import React from "react";
import {AsTextLink} from '@globalart/react-shared-components';
import { faDungeon } from '@fortawesome/free-solid-svg-icons';

export default {
  REALMS: {
    MENU: {
      text: 'Realms',
      description: 'Manage the active security realms and their order',
      icon: faDungeon
    },
    CONFIGURATION: {
      LABEL: 'Realms Configuration',
      SUB_LABEL: 'Active Realms',
      AVAILABLE_TITLE: 'Available',
      SELECTED_TITLE: 'Active',
      EMPTY_LIST: 'There are no configured realms for which you have view permissions.',
    },
    MESSAGES: {
      NO_REALMS_CONFIGURED: 'At least one realm must be selected and should include a user that is able to update the list of realms',
      MULTIPLE_SSO_ENABLED: 'Invalid configuration. Only one SSO realm can be enabled at a time: '
    },
    LOCAL_REALM_REMOVAL_MODAL: {
      HEADER: 'Confirm Removal of Local Realms',
      WARNING: <>
          Warning! Removing local realms will prevent local admin access.
          This can make system recovery more difficult should there be issues with
          other authentication realms. For more information, see the{' '}
          <AsTextLink 
            external 
            href="http://links.globalart.dev/products/astrum/docs/realms"
          >
            Realms help documentation
          </AsTextLink>
        </>,
      CONFIRM_BUTTON: 'Confirm',
      ACKNOWLEDGEMENT: {
        LABEL: 'Acknowledgement',
        STRING: 'I acknowledge',
        PLACEHOLDER: 'Provide acknowledgement',
        VALIDATION_ERROR: 'Invalid acknowledgement',
        get SUBLABEL () {return `Type "${this.STRING}" in order to proceed with this action`}
      }
    },
    SSO_REALMS: {
      OAUTH2: 'OAuth2Realm',
      SAML: 'SamlRealm'
    }
  }
}
