
import Axios from 'axios';
import {assign} from 'xstate';
import {APIConstants, ExtAPIUtils, ExtJS, ListMachineUtils} from '@globalart/astrum-ui-plugin';

import { filterReposByUiUpload } from '../BrowseUtils';

const {EXT: {UPLOAD: {ACTION, METHODS}, REPOSITORY}} = APIConstants;

export default ListMachineUtils.buildListMachine({
  id: 'UploadListMachine',
  sortableFields: ['name', 'format']
}).withConfig({
  actions: {
    setData: assign((_, {data: [repositories, definitions]}) => ({
      pristineData: filterReposByUiUpload(definitions, repositories)
    })),
    filterData: assign({
      data: ({filter, pristineData}, _) => pristineData.filter(
        ({name, format}) => ListMachineUtils.hasAnyMatches([name, format], filter)
      )
    })
  },
  services: {
    fetchData: async () => {
      await ExtJS.waitForPermissions();
      return Axios.all([
        ExtAPIUtils.extAPIRequest(REPOSITORY.ACTION, REPOSITORY.METHODS.READ_REFERENCES, {})
            .then(ExtAPIUtils.checkForErrorAndExtract),
        ExtAPIUtils.extAPIRequest(ACTION, METHODS.GET_UPLOAD_DEFINITIONS).then(ExtAPIUtils.checkForErrorAndExtract),
      ])
    },
  }
})
