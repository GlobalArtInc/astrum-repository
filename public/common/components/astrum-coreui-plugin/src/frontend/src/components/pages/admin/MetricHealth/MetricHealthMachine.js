
import Axios from 'axios';
import {toPairs, map, mergeRight, sortBy, compose, toLower, prop} from 'ramda';
import {ListMachineUtils, APIConstants} from '@globalart/astrum-ui-plugin';
import {assign} from 'xstate';
import {URL, isClustered} from './MetricHealthHelper';

export const convert = (result) => {
  const list = map(
    ([name, metrics]) => mergeRight({name}, metrics),
    toPairs(result)
  );
  const sortByNameCaseInsensitive = sortBy(compose(toLower, prop('name')));

  return sortByNameCaseInsensitive(list);
};

const setData = (_, event) => {
  const result = isClustered() ? event.data.data.results : event.data.data;
  return convert(result);
};

export default ListMachineUtils.buildListMachine({
  id: 'MetricHealthMachineDetails',
  sortableFields: ['name', 'message', 'error'],
}).withConfig({
  actions: {
    setData: assign({
      data: setData,
      pristineData: (_, event) => event.data.data,
      name: (_, event) => event.data.data.hostname,
    }),
  },
  services: {
    fetchData: ({itemId}) => {
      return isClustered()
        ? Axios.get(URL.singleNodeUrl(itemId))
        : Axios.get(APIConstants.REST.INTERNAL.GET_STATUS);
    },
  },
});
