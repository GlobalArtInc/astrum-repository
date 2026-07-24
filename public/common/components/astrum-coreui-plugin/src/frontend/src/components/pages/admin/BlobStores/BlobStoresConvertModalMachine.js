

import Axios from 'axios';
import {assign} from 'xstate';

import {FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import {URLs} from './BlobStoresHelper';

export default FormUtils.buildFormMachine({
  id: 'BlobStoresConvertModalMachine',
  initial: 'loaded',
}).withConfig({
  actions: {
    validate: assign({
      validationErrors: ({data}) => ({
        newName: ValidationUtils.validateNameField(data.newName)
      }),
    }),
  },
  services: {
    saveData: ({data, pristineData}) =>
        Axios.post(URLs.convertToGroupBlobStoreUrl(pristineData.name, data.newName)),
  }
});
