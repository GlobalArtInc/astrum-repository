
import axios from 'axios';
import {assign} from 'xstate';

import {ListMachineUtils} from '@globalart/astrum-ui-plugin';

export default ListMachineUtils.buildListMachine({
  id: 'LogViewerListMachine',
  sortField: 'fileName',
  sortableFields: ['fileName', 'size', 'lastModified']
}).withConfig({
  actions: {
    filterData: assign({
      data: ({filter, pristineData}) => {
        return pristineData.filter(({fileName}) => fileName.toLowerCase().includes(filter.toLowerCase()));
      }
    })
  },
  services: {
    fetchData: () => axios.get('service/rest/internal/logging/logs')
  }
});



