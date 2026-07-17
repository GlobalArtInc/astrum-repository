
import {assign} from 'xstate';
import Axios from 'axios';
import {isEmpty, filter, mergeDeepRight} from 'ramda';

import {
  ExtJS,
  FormUtils,
  ValidationUtils,
  APIConstants,
  FormFieldsFactory,
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import {
  EMPTY_DATA,
  URL,
  FIELDS,
  modifyFormFields,
  convertActionsToArray,
  convertActionsToObject,
  normalizeWildCardFormat,
  TYPES,
} from './PrivilegesHelper';


const {REST: {INTERNAL}} = APIConstants;
const {PRIVILEGES: {MESSAGES: LABELS}} = UIStrings;

const {singlePrivilegeUrl, updatePrivilegeUrl, createPrivilegeUrl} = URL;

const isEdit = (name) => ValidationUtils.notBlank(name);
const actionFieldName = FIELDS.ACTIONS.NAME;

function validateActionSelected(data) {
  if (data.hasOwnProperty(actionFieldName)) {
    const actions = data[actionFieldName];
    return isEmpty(filter(v => v === true, actions)) ? LABELS.NO_ACTION_ERROR : undefined;
  }
}

export default FormUtils.buildFormMachine({
  id: 'PrivilegesFormMachine',
  config: (config) =>
      mergeDeepRight(config, {
        states: {
          loaded: {
            on: {
              SET_TYPE: {
                target: 'loaded',
                actions: ['clearSaveError', 'resetData']
              },
            },
          },
        },
      })
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data, types}) => ({
        name: ValidationUtils.validateNotBlank(data.name) || ValidationUtils.validateName(data.name),
        type: ValidationUtils.validateNotBlank(data.type),
        ...FormFieldsFactory.getValidations(data, types),
        actionSelected: validateActionSelected(data)
      })
    }),
    setData: assign((_, {data: [types, privilege]}) => {
      const typesMap = modifyFormFields(types.data);
      const data = convertActionsToObject(privilege.data, typesMap);
      if(data.type === TYPES.REPOSITORY_CONTENT_SELECTOR) {
        if(data.format !== '*' && data.repository === '*') {
          data.repository = `*-${data.format}`;
          data.format = '*';
        }
      }

      return {
        types: typesMap,
        data,
        pristineData: data,
      };
    }),
    resetData: assign({
      data: ({types}, {privilegeType}) => ({
        ...EMPTY_DATA,
        ...FormFieldsFactory.defaultValues(privilegeType, types),
        type: privilegeType,
      }),
    }),
    onDeleteError: (_, event) => ExtJS.showErrorMessage(event.data?.response?.data),
    logDeleteSuccess: ({data}) => ExtJS.showSuccessMessage(LABELS.DELETE_SUCCESS(data.name)),
  },
  services: {
    fetchData: ({pristineData: {name}}) => {
      return Axios.all([
        Axios.get(INTERNAL.PRIVILEGES_TYPES),
        isEdit(name)
            ? Axios.get(singlePrivilegeUrl(name))
            : Promise.resolve({data: EMPTY_DATA}),
      ]);
    },
    saveData: ({data, pristineData: {name}, types}) => {
      const requestData = convertActionsToArray(data, types);
      const normalizedData = normalizeWildCardFormat(requestData);
      if (isEdit(name)) {
        return Axios.put(updatePrivilegeUrl(data.type, name), normalizedData);
      } else {
        return Axios.post(createPrivilegeUrl(data.type), normalizedData);
      }
    },
    confirmDelete: ({data}) => ExtJS.requestConfirmation({
      title: LABELS.CONFIRM_DELETE.TITLE,
      message: LABELS.CONFIRM_DELETE.MESSAGE(data.name),
      yesButtonText: LABELS.CONFIRM_DELETE.YES,
      noButtonText: LABELS.CONFIRM_DELETE.NO
    }),
    delete: ({data}) => Axios.delete(singlePrivilegeUrl(data.name)),
  }
});
