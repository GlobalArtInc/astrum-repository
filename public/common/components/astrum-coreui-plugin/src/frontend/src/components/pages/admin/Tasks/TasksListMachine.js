
import {assign} from 'xstate';

import {ListMachineUtils, ExtAPIUtils, APIConstants, ExtJS} from '@globalart/astrum-ui-plugin';

const {EXT: {TASK: {ACTION, METHODS}}} = APIConstants;

export default ListMachineUtils.buildListMachine({
  id: 'TasksListMachine',
  sortableFields: ['name', 'typeName', 'statusDescription', 'schedule', 'nextRun', 'lastRun', 'lastRunResult'],
  sortField: 'name',
}).withConfig({
  actions: {
    setData: assign({
      data: (_, event) => event.data,
      pristineData: (_, event) => event.data,
    }),
    filterData: assign({
      data: ({filter, pristineData}) => pristineData.filter((task) => {
        return ListMachineUtils.hasAnyMatches([
            task.name,
            task.typeName,
            task.statusDescription,
            task.schedule,
            task.nextRun,
            task.lastRun,
            task.lastRunResult,
        ], filter);
      }
      )}),
  },
  services: {
    fetchData: async () => {
      await ExtJS.waitForPermissions();
      const response = await ExtAPIUtils.extAPIRequest(ACTION, METHODS.READ);
      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    },
  },
});
