
import {clone, filter, find, indexBy, keys, prop, propEq} from 'ramda';
import {APIConstants} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {
  PRIVILEGES: { FORM: LABELS },
} = UIStrings;
const {
  REST: {
    PUBLIC: { PRIVILEGES: privilegesUrl },
  },
} = APIConstants;

export const TYPES = {
  SCRIPT: 'script',
  REPOSITORY_ADMIN: 'repository-admin',
  REPOSITORY_CONTENT_SELECTOR: 'repository-content-selector',
  REPOSITORY_VIEW: 'repository-view',
  APPLICATION: 'application',
  WILDCARD: 'wildcard',
};

export const FIELDS = {
  NAME: {
    NAME: 'name',
  },
  SCRIPT_NAME: {
    NAME: 'scriptName',
    LABEL: 'Script Name',
  },
  CONTENT_SELECTOR: {
    NAME: 'contentSelector',
    LABEL: 'Content Selector',
  },
  PATTERN: {
    NAME: 'pattern',
  },
  REPOSITORY: {
    NAME: 'repository',
    LABEL: 'Repository',
  },
  ACTIONS: {
    NAME: 'actions',
    LABEL: 'Actions',
  },
  FORMAT: {
    NAME: 'format',
    LABEL: 'Format',
  },
};

export const ACTIONS = {
  DISASSOCIATE: 'DISASSOCIATE',
  ADD: 'ADD',
  READ: 'READ',
  BROWSE: 'BROWSE',
  DELETE: 'DELETE',
  ASSOCIATE: 'ASSOCIATE',
  ALL: 'ALL',
  EDIT: 'EDIT',
  RUN: 'RUN',
};

export const EMPTY_DATA = {
  type: '',
  name: '',
  description: '',
};

const singlePrivilegeUrl = name => `${privilegesUrl}/${encodeURIComponent(name)}`;
const createPrivilegeUrl = type => `${privilegesUrl}/${encodeURIComponent(type)}`;
const updatePrivilegeUrl = (type, name) => `${privilegesUrl}/${encodeURIComponent(type)}/${encodeURIComponent(name)}`;

export const URL = {privilegesUrl, singlePrivilegeUrl, updatePrivilegeUrl, createPrivilegeUrl};

function areAllActionsSelected(typeDefinition, selectedActions) {
  if (typeDefinition) {
    const actionsField = typeDefinition.formFields?.find(field => field.id === FIELDS.ACTIONS.NAME);
    const availableActions = actionsField?.attributes?.options || [];

    // If all available actions are selected, send "ALL" (more concise)
    const allSelected = availableActions.length > 0 && availableActions.every(action =>
        selectedActions.includes(action)
    );

    return allSelected;
  }

  return false;
}

export const convertActionsToArray = (data, types = {}) => {
  const clonedData = clone(data);
  const fieldName = FIELDS.ACTIONS.NAME;
  const actionsObj = clonedData[fieldName];

  if (actionsObj == null) {
    return clonedData;
  }

  const selectedActions = keys(filter(v => v === true, actionsObj));
  const privilegeType = clonedData.type;
  const typeDefinition = types[privilegeType];

  // Check if all available actions are selected, then we can send "ALL"
  if (areAllActionsSelected(typeDefinition, selectedActions)) {
    clonedData[fieldName] = [ACTIONS.ALL];
    return clonedData;
  }

  const actionsArr = selectedActions.map(e => {
    return e === 'create' ? ACTIONS.ADD : e === 'update' ? ACTIONS.EDIT : e;
  });

  clonedData[fieldName] = actionsArr;
  return clonedData;
};

export const convertActionsToObject = (data, types = {}) => {
  const clonedData = clone(data);
  const fieldName = FIELDS.ACTIONS.NAME;
  const actions = clonedData[fieldName];

  if (actions == null) {
    return clonedData;
  }

  if (actions.length === 0) {
    clonedData[fieldName] = {};
    return clonedData;
  }

  // Check if actions contain "ALL" or "*"
  const hasAllAction = actions.some(action =>
    action === ACTIONS.ALL || action === '*' || action.toUpperCase() === 'ALL'
  );

  if (hasAllAction) {
    // Get available actions for this privilege type from form field definition
    const privilegeType = clonedData.type;
    const typeDefinition = types[privilegeType];

    if (typeDefinition) {
      const actionsField = typeDefinition.formFields?.find(field => field.id === fieldName);
      const availableActions = actionsField?.attributes?.options || [];

      // Expand "ALL" to all available actions
      const actionsObj = {};
      availableActions.forEach(action => {
        actionsObj[action] = true;
      });

      clonedData[fieldName] = actionsObj;
      return clonedData;
    }
  }

  const actionsObj = Object.assign(...actions.map(e => {
    if (clonedData.type === TYPES.APPLICATION) {
      return e === ACTIONS.ADD ? {'create': true} : e === ACTIONS.EDIT ? {'update': true} : {[e.toLowerCase()]: true};
    } else {
      return {[e.toLowerCase()]: true};
    }
  }));

  clonedData[fieldName] = actionsObj;
  return clonedData;
};

const renameScriptId = types => {
  const type = types[TYPES.SCRIPT];
  if (!type) return;
  const field = find(propEq('id', FIELDS.NAME.NAME))(type.formFields);
  if (!field) return;
  field.id = FIELDS.SCRIPT_NAME.NAME;
};

const renamePatternHelpText = types => {
  const type = types[TYPES.WILDCARD];
  if (!type) return;
  const field = find(propEq('id', FIELDS.PATTERN.NAME))(type.formFields);
  if (!field) return;
  field.helpText = LABELS.PRIVILEGE_STRING.SUB_LABEL;
};

export const modifyFormFields = typesArr => {
  const types = indexBy(prop('id'), typesArr || []);
  renameScriptId(types);
  // Removed addFormatFieldForRepoSelectorType since backend doesn't provide format field for repository-content-selector
  renamePatternHelpText(types);

  return types;
};

export const normalizeWildCardFormat = data => {
  const clonedData = clone(data);
  if (clonedData.repository) {
    const [repository, format] = clonedData.repository.split('-');
    if (repository === '*' && format) {
      clonedData.format = format;
      clonedData.repository = repository;
    }
  }
  return clonedData;
};
