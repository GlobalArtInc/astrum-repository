
import React from 'react';
import {assign} from 'xstate';
import {sortBy, prop} from 'ramda';
import {FormUtils, ExtAPIUtils, APIConstants} from '@globalart/astrum-ui-plugin';

const {EXT: {PROPRIETARY_REPOSITORIES: {ACTION, METHODS}, }} = APIConstants;

export default FormUtils.buildFormMachine({
  id: 'ProprietaryRepositoriesMachine',
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: () => {{}}
    }),
    setData: assign((_, event) => {
      const [selected, possible] = sortBy(prop('tid'), event.data?.data || []);
      const data = selected?.result?.data;
      return {
        data,
        pristineData: data,
        possibleRepos: possible?.result?.data,
      };
    }),
  },
  services: {
    fetchData: () => ExtAPIUtils.extAPIBulkRequest([
      {action: ACTION, method: METHODS.READ},
      {action: ACTION, method: METHODS.POSSIBLE_REPOS},
    ]),
    saveData: ({data}) => ExtAPIUtils.extAPIRequest(ACTION, METHODS.UPDATE, {
      data: [{'enabledRepositories': data.enabledRepositories}],
    }).then(v => v.data.result),
  }
});
