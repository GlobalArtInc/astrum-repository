
import {assign} from 'xstate';
import Axios from 'axios';
import {ListMachineUtils} from '@globalart/astrum-ui-plugin';

export default ListMachineUtils.buildListMachine({
  id: 'ContentSelectorsListMachine',
  sortableFields: ['name', 'type', 'description']
}).withConfig({
  actions: {
    filterData: assign({
      data: ({filter, data, pristineData}, _) => pristineData.filter(
          ({name, description}) => name.toLowerCase().indexOf(filter.toLowerCase()) !== -1 ||
              description.toLowerCase().indexOf(filter.toLowerCase()) !== -1)
    })
  },
  services: {
    fetchData: () => Axios.get('service/rest/v1/security/content-selectors')
  }
});
