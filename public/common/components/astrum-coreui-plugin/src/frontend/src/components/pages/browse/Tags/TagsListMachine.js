
import {assign} from 'xstate';
import {APIConstants, ExtAPIUtils, ExtJS, ListMachineUtils} from '@globalart/astrum-ui-plugin';

const {EXT: {TAGS_LIST: {ACTION, METHODS}}} = APIConstants;

export default ListMachineUtils.buildListMachine({
  id: 'TagsListMachine',
  sortField : 'id',
  sortableFields: ['id', 'firstCreatedTime', 'lastUpdatedTime'],
}).withConfig({
  actions: {
    filterData: assign({
      data: ({filter, pristineData}, _) => pristineData.filter(
        ({id}) => ListMachineUtils.hasAnyMatches([id], filter)
      )
    })
  },
  services: {
    fetchData: async () => {
      await ExtJS.waitForPermissions();
      const tagListData = await ExtAPIUtils.extAPIRequest(ACTION, METHODS.READ_TAGS);
      ExtAPIUtils.checkForError(tagListData);
      return tagListData.data.result;
    }
  }
});
