
import Axios from 'axios';
import {APIConstants} from '@globalart/astrum-ui-plugin';

export const cleanNode = async (nodeId) => {
  return await Axios.delete(
    APIConstants.REST.INTERNAL.CLEAR_SUPPORT_ZIP_HISTORY + nodeId
  );
};
