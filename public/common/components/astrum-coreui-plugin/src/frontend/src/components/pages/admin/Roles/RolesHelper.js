
import {APIConstants} from '@globalart/astrum-ui-plugin';

const {REST: {PUBLIC: {ROLES: rolesUrl, PRIVILEGES: privilegesUrl}}} = APIConstants;

export const TYPES = {
  INTERNAL: 'internal',
  EXTERNAL: 'external',
};

export const EMPTY_DATA = {
  id: '',
  name: '',
  description: '',
  privileges: [],
  roles: [],
};

const sourcesApi = {action: 'coreui_Role', method: 'readSources'};
const getRolesUrl = (source) => `${rolesUrl}?source=${encodeURIComponent(source)}`;
const defaultRolesUrl = getRolesUrl('default');
const singleRoleUrl = (id) => `${rolesUrl}/${encodeURIComponent(id)}`;
const getLdapRolesUrl = (query, source) => `service/rest/internal/ui/roles?search=${query}&source=${source}`;

export const URL = {rolesUrl, privilegesUrl, sourcesApi, getRolesUrl, defaultRolesUrl, singleRoleUrl, getLdapRolesUrl};
