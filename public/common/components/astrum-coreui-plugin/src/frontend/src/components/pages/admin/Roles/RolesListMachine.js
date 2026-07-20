
import {assign} from 'xstate';
import Axios from 'axios';
import {ListMachineUtils, ExtAPIUtils, APIConstants, ExtJS, Permissions} from '@globalart/astrum-ui-plugin';
import {URL} from './RolesHelper';

const {EXT: {CAPABILITY: {ACTION, METHODS}}} = APIConstants;

const DEFAULT_ROLE_CAPABILITY_TYPE_ID = 'defaultrole';
const DEFAULT_ROLE_STATE_KEY = 'defaultRole';

export default ListMachineUtils.buildListMachine({
  id: 'RolesListMachine',
  sortableFields: ['id', 'name', 'description'],
  sortField: 'id',
}).withConfig({
  actions: {
    filterData: assign({
      data: ({filter, pristineData}, _) => pristineData.filter(({id, name, description}) =>
          ListMachineUtils.hasAnyMatches([id, name, description], filter)
      )}),
    setData: assign((_, {data: [roles, capabilities]}) => {
      const defaultRoleState = ExtJS.state().getValue(DEFAULT_ROLE_STATE_KEY);
      const defaultRoleCapability = capabilities.find(it => it.typeId === DEFAULT_ROLE_CAPABILITY_TYPE_ID);
      const defaultRole = {
        capabilityId: defaultRoleCapability && defaultRoleCapability.enabled ? defaultRoleCapability.id : null,
        roleId: defaultRoleState ? defaultRoleState.id : null,
        roleName: defaultRoleState ? defaultRoleState.name : null,
      };

      return {
        defaultRole,
        data: roles.data,
        pristineData: roles.data,
      };
    }),
  },
  services: {
    fetchData: () => Axios.all([
      Axios.get(URL.defaultRolesUrl),
      ExtJS.state().getValue('capabilityActiveTypes').includes(DEFAULT_ROLE_CAPABILITY_TYPE_ID) &&
      ExtJS.checkPermission(Permissions.CAPABILITIES.READ)
          ? ExtAPIUtils.extAPIRequest(ACTION, METHODS.READ).then(ExtAPIUtils.checkForErrorAndExtract)
          : Promise.resolve([]),
    ]),
  }
});
