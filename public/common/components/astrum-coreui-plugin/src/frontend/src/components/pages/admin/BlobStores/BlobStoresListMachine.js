
import Axios from 'axios';
import {assign} from 'xstate';
import {ListMachineUtils, ExtJS} from '@globalart/astrum-ui-plugin';

const BLOB_STORES_URL = 'service/rest/internal/ui/blobstores';

export default ListMachineUtils.buildListMachine({
  id: 'BlobStoresListMachine',
  sortableFields: ['name', 'path', 'typeName', 'available', 'blobCount', 'totalSizeInBytes', 'availableSpaceInBytes'],
}).withConfig({
  actions: {
    setData: assign({
      data: (_, {data}) => data.data.map(blobStore => ({
        ...blobStore,
        available: !blobStore.unavailable,
        blobCount: blobStore.unavailable ? -1 : blobStore.blobCount,
        totalSizeInBytes: blobStore.unavailable ? -1 : blobStore.totalSizeInBytes,
        availableSpaceInBytes: blobStore.unlimited ? Infinity : blobStore.availableSpaceInBytes
      })),
      pristineData: (_, {data}) => data.data.map(blobStore => ({
        ...blobStore,
        available: !blobStore.unavailable,
        blobCount: blobStore.unavailable ? -1 : blobStore.blobCount,
        totalSizeInBytes: blobStore.unavailable ? -1 : blobStore.totalSizeInBytes,
        availableSpaceInBytes: blobStore.unlimited ? Infinity : blobStore.availableSpaceInBytes
      })),
    }),

    filterData: assign({
      data: ({filter, data, pristineData}, _) => pristineData.filter(({name}) =>
          name.toLowerCase().indexOf(filter.toLowerCase()) !== -1
      )
    })
  },
  services: {
    fetchData: async () => {
      await ExtJS.waitForPermissions();
      return Axios.get(BLOB_STORES_URL);
    }
  }
});
