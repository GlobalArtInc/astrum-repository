
import {assign, Machine} from 'xstate';
import Axios from 'axios';
import {ListMachineUtils} from '@globalart/astrum-ui-plugin';

export default ListMachineUtils.buildListMachine({
  id: 'RoutingRulesListMachine',
  sortableFields: ['name', 'description', 'assignedRepositoryCount'],
  config: (config) => ({
    ...config,
    states: {
      ...config.states,
      preview: {
        type: 'final'
      }
    },
    on: {
      PREVIEW: {
        target: 'preview'
      }
    }
  })
}).withConfig({
  actions: {
    filterData: assign({
      data: ({filter, data, pristineData}, _) => pristineData.filter(({name, description}) =>
          name.toLowerCase().indexOf(filter.toLowerCase()) !== -1 ||
          description.toLowerCase().indexOf(filter.toLowerCase()) !== -1)
    })
  },
  services: {
    fetchData: () => Axios.get('service/rest/internal/ui/routing-rules?includeRepositoryNames=true')
  }
});
