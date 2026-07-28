
import {assign} from 'xstate';
import Axios from 'axios';
import ListMachineUtils from '../../../interface/ListMachineUtils';

export default ListMachineUtils.buildListMachine({
  id: 'CleanupPoliciesListMachine',
  sortableFields: ['name', 'format', 'notes']
}).withConfig({
  actions: {
    filterData: assign({
      data: ({filter, pristineData}) => pristineData.filter(
          ({name, format, notes}) => name.toLowerCase().indexOf(filter.toLowerCase()) !== -1 ||
              format.toLowerCase().indexOf(filter.toLowerCase()) !== -1 ||
              notes.toLowerCase().indexOf(filter.toLowerCase()) !== -1)
    })
  },
  services: {
    fetchData: () => Axios.get('service/rest/internal/cleanup-policies')
  }
});
