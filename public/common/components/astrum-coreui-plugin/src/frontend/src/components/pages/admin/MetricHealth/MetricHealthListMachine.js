
import Axios from 'axios';
import {toPairs, map, filter} from 'ramda';
import {assign} from 'xstate';
import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {URL} from './MetricHealthHelper';

export const convert = (data) => {
  return map(({hostname, nodeId, results}) => {
    const errors = filter(([_, value]) => !value.healthy, toPairs(results));
    const errorsNumber = errors.length;
    const oneError = errorsNumber === 1;
    const multipleErrors = errorsNumber > 1;

    const name = hostname;
    const status = '';
    let error = '';
    let message = '';

    if (oneError) {
      const [key, value] = errors[0];
      error = key;
      message = value.message;
    }

    if (multipleErrors) {
      error = `(${errorsNumber}) Errors found`;
      message = 'Click to see details';
    }

    return {name, status, error, message, nodeId};
  }, data);
};

const setData = (_, event) => convert(event.data.data);

export default ListMachineUtils.buildListMachine({
  id: 'MetricHealthListMachine',
  sortableFields: ['name', 'status', 'error', 'message'],
}).withConfig({
  actions: {
    setData: assign({
      data: setData,
      pristineData: (_, event) => event.data.data,
    }),
  },
  services: {
    fetchData: () => Axios.get(URL.nodesUrl),
  },
});
