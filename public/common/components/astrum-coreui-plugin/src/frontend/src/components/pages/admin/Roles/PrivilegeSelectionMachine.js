
import { mergeDeepRight } from 'ramda';
import { assign } from 'xstate';
import { ListMachineUtils } from '@globalart/astrum-ui-plugin';

const PrivilegeSelectionMachine = ListMachineUtils.buildListMachine({
  id: 'PrivilegeSelectionMachine',
  sortableFields: ['select', 'name', 'description'],
  sortField: 'name',
  initial: 'loaded',
  config: config => mergeDeepRight(config, {
    states: {
      loaded: {
        on: {
          SELECT_PRIVILEGE: {
            target: 'loaded',
            actions: ['selectPrivilege']
          },
          ON_CONFIRM: {
            target: 'loaded',
            actions: ['onConfirm']
          },
          CHANGE_PAGE: {
            target: 'loaded',
            actions: ['changePage']
          },
        },
      },
    }
  }),
}).withConfig({
  actions: {
    selectPrivilege: assign({
      data: ({ data }, { privilege }) => data.map(oldPrivilege => oldPrivilege === privilege ?
        { ...privilege, isSelected: !privilege.isSelected } : oldPrivilege),
      tempSelectedPrivileges: ({ data }, { privilege }) =>
        data.filter(oldPrivilege => oldPrivilege === privilege ?
          !oldPrivilege.isSelected : oldPrivilege.isSelected).map(privilege => privilege.name)
    }),

    onConfirm: assign({
      selectedPrivileges: ({ tempSelectedPrivileges }) => tempSelectedPrivileges
    }),

    changePage: assign({
      offsetPage: (_, { offsetPage }) => offsetPage
    }),

    setFilter: assign({
      filter: (_, { filter }) => filter,
      offsetPage: 0,
    }),

    filterData: assign({
      filteredData: ({ filter, data }, _) => {
        const regExp = new RegExp('[-]?' + filter.toLowerCase().replace(/\*/g, '.*')) ;
        return data.filter(({ name, description }) => regExp.test(name.toLowerCase()) || regExp.test(description.toLowerCase()));
      },
      numberOfPrivileges: ({ filter, data }, _) => {
        const regExp = new RegExp('[-]?' + filter.toLowerCase().replace(/\*/g, '.*'));
        return data.filter(({ name, description }) => regExp.test(name.toLowerCase()) ||
          regExp.test(description.toLowerCase())).length;
      }
    }),

    sortData: assign({
      filteredData: ({ sortField, sortDirection, filteredData }) => {
        if (sortField === 'select') {
          return sortDirection === 'asc' ? filteredData.sort((a, b) =>
            Number(b.isSelected) - Number(a.isSelected) || a.name.localeCompare(b.name))
            : filteredData.sort((a, b) => Number(a.isSelected) - Number(b.isSelected) || a.name.localeCompare(b.name));
        } else {
          return ListMachineUtils.sortDataByFieldAndDirection({ useLowerCaseSorting: true })
            ({ sortField, sortDirection, data: filteredData });
        }
      }
    })
  }
});

export default PrivilegeSelectionMachine;
