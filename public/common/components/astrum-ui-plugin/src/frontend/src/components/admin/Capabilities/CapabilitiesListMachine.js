
import { assign } from 'xstate';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';
import ListMachineUtils from '../../../interface/ListMachineUtils';
import APIConstants from '../../../constants/APIConstants';

const { ACTION, METHODS } = APIConstants.EXT.CAPABILITY;

export default ListMachineUtils.buildListMachine({
  id: 'CapabilitiesListMachine',
  sortField: 'typeName',
  sortDirection: 'asc',
  sortableFields: ['typeName', 'state', 'category', 'repository', 'description', 'notes'],
}).withConfig({
  actions: {
    setData: assign({
      data: (_, { data }) => data.map(item => ({
        ...item,
        category: item.tags?.Category || '',
        repository: item.tags?.Repository || ''
      })),
      pristineData: (_, { data }) => data.map(item => ({
        ...item,
        category: item.tags?.Category || '',
        repository: item.tags?.Repository || ''
      })),
    }),
    filterData: assign({
      data: ({ filter, pristineData }) => {
        if (!filter) {
          return pristineData;
        }
        const lowerFilter = filter.toLowerCase();
        return pristineData.filter(capability =>
          ListMachineUtils.hasAnyMatches([
            capability.typeName,
            capability.description,
            capability.notes,
            capability.tags?.Category,
            capability.tags?.Repository,
            capability.state
          ], lowerFilter)
        );
      }
    }),
  },
  services: {
    fetchData: () => ExtAPIUtils.extAPIRequest(ACTION, METHODS.READ).then(ExtAPIUtils.checkForErrorAndExtract),
  },
  on: {
    RETRY: {
      target: 'loading',
    },
  },
});
