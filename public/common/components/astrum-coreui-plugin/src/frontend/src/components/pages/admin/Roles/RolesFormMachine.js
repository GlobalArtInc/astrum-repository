
import {assign, spawn} from 'xstate';
import Axios from 'axios';
import {mergeDeepRight} from 'ramda';

import {ExtJS, FormUtils, ValidationUtils, ExtAPIUtils} from '@globalart/astrum-ui-plugin';

import ExternalRolesComboboxMachine from './ExternalRolesComboboxMachine';

import UIStrings from '../../../../constants/UIStrings';
import {TYPES, EMPTY_DATA, URL} from './RolesHelper';

const {ROLES: {MESSAGES: LABELS}} = UIStrings;
const {rolesUrl, privilegesUrl, sourcesApi, defaultRolesUrl, singleRoleUrl} = URL;

const validateId = (id) => {
  if (ValidationUtils.isBlank(id)) {
    return UIStrings.ERROR.FIELD_REQUIRED;
  } else {
    return ValidationUtils.validateLeadingOrTrailingSpace(id)
  }
}

export default FormUtils.buildFormMachine({
  id: 'RolesFormMachine',
  config: (config) =>
      mergeDeepRight(config, {
        context: {
          privilegesListFilter: '',
          rolesListFilter: ''
        },
        states: {
          loaded: {
            on: {
              SET_ROLE_TYPE: {
                target: 'loaded',
                actions: ['clearSaveError', 'resetData', 'setRoleType']
              },
              SET_EXTERNAL_ROLE_TYPE: {
                target: 'loaded',
                actions: [
                    'clearSaveError',
                    'resetData',
                    'setExternalRoleType',
                    'initExternalRolesActor',
                    'sendDataToActor',
                    'sendLdapDataToActor'
                ],
              },
              UPDATE_ROLES: {
                target: 'loaded',
                actions: ['updateRoles']
              },
              UPDATE_PRIVILEGES: {
                target: 'loaded',
                actions: ['updatePrivileges']
              },
              SET_PRIVILEGES_LIST_FILTER: {
                target: 'loaded',
                actions: ['updatePrivilegesListFilter']
              },
              SET_ROLES_LIST_FILTER: {
                target: 'loaded',
                actions: ['updateRolesListFilter']
              }
            }
          },
        }
      })
}).withConfig({
  actions: {
    initExternalRolesActor: assign({
      externalRolesRef: ({externalRolesRef}) =>
          externalRolesRef || spawn(ExternalRolesComboboxMachine, 'externalRolesCombobox'),
    }),
    sendDataToActor: ({externalRoleType, externalRolesRef}) => externalRolesRef.send(
        {type: 'UPDATE_TYPE', externalRoleType}, {to: 'externalRolesCombobox'},
    ),
    sendLdapDataToActor: ({externalRolesRef, ldapQueryCharacterLimit}) => externalRolesRef.send(
      {type: 'UPDATE_LDAP_LIMIT', ldapQueryCharacterLimit}, {to: 'externalRolesCombobox'},
  ),
    validate: assign({
      validationErrors: ({data}) => ({
        id: validateId(data?.id),
        name: ValidationUtils.validateNotBlank(data.name),
      })
    }),
    setData: assign({
      roles: (_, {data: [roles]}) => roles.data,
      privileges: (_, {data: [, privileges]}) => privileges.data,
      sources: (_, {data: [, , sources]}) => sources.data,
      data: (_, {data: [, , , role]}) => role.data,
      pristineData: (_, {data: [, , , role]}) => role.data,
    }),
    setRoleType: assign({
      roleType: (_, {roleType}) => roleType
    }),
    setExternalRoleType: assign({
      roleType: TYPES.EXTERNAL,
      externalRoleType: (_, {externalRoleType}) => externalRoleType,
    }),
    resetData: assign({
      data: () => EMPTY_DATA,
      roleType: '',
      externalRoleType: '',
    }),
    updateRoles: assign({
      data: ({data}, {newRoles}) => ({...data, roles: newRoles}),
    }),
    updatePrivileges: assign({
      data: ({data}, {newPrivileges}) => ({...data, privileges: newPrivileges}),
    }),
    updatePrivilegesListFilter: assign({
      privilegesListFilter: (_, {privilegesListFilter}) => privilegesListFilter
    }),
    updateRolesListFilter: assign({
      rolesListFilter: (_, {rolesListFilter}) => rolesListFilter
    }),
    onDeleteError: (_, event) => ExtJS.showErrorMessage(event.data?.response?.data),
    logDeleteSuccess: ({data}) => ExtJS.showSuccessMessage(LABELS.DELETE_SUCCESS(data.name)),
  },
  services: {
    fetchData: ({pristineData: {id}, isCreate}) => {
      return Axios.all([
        Axios.get(defaultRolesUrl),
        Axios.get(privilegesUrl),
        isCreate
            ? ExtAPIUtils.extAPIRequest(sourcesApi.action, sourcesApi.method).then(v => v.data.result)
            : Promise.resolve({data: []} ),
        isCreate || !id
            ? Promise.resolve({data: EMPTY_DATA})
            : Axios.get(singleRoleUrl(id)),
      ]);
    },
    saveData: ({data, isCreate}) => {
      if (isCreate) {
        return Axios.post(rolesUrl, data);
      } else {
        return Axios.put(singleRoleUrl(data.id), data);
      }
    },
    confirmDelete: ({data}) => ExtJS.requestConfirmation({
      title: LABELS.CONFIRM_DELETE.TITLE,
      message: LABELS.CONFIRM_DELETE.MESSAGE(data.name),
      yesButtonText: LABELS.CONFIRM_DELETE.YES,
      noButtonText: LABELS.CONFIRM_DELETE.NO
    }),
    delete: ({data}) => Axios.delete(singleRoleUrl(data.id)),
  }
});
