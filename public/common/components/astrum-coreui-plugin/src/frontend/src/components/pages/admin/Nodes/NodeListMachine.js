
import Axios from 'axios';
import {ListMachineUtils, APIConstants} from '@globalart/astrum-ui-plugin';

const {
  REST: {
    INTERNAL: {GET_SUPPORT_ZIP_ACTIVE_NODES}
  }
} = APIConstants;

export default ListMachineUtils.buildListMachine({
  id: 'NodesListMachine'
}).withConfig({
  services: {
    fetchData: () => Axios.get(GET_SUPPORT_ZIP_ACTIVE_NODES)
  }
});
