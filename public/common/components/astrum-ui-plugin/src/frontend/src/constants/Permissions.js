
import {pick, toUpper, toLower} from 'ramda';

const CREATE = 'CREATE';
const READ = 'READ';
const UPDATE = 'UPDATE';
const DELETE = 'DELETE';
const START = 'START';
const STOP = 'STOP';
const UNINSTALL = 'UNINSTALL';

// default options.
const crudOptions = [CREATE, READ, UPDATE, DELETE];

function generatePermissions(permission, options = crudOptions) {
  const base = 'astrum';

  const _ = (value) => `${base}:${permission}:${toLower(value)}`;

  const entries = options.map((value) => [toUpper(value), _(value)]);

  const permissions = Object.fromEntries(entries);

  return pick(options, permissions);
}

const Permissions = {
  ADMIN: 'astrum:*',
  REPOSITORY_ADMIN: {
    READ: 'astrum:repository-admin:*:*:read',
    EDIT: 'astrum:repository-admin:*:*:edit',
    DELETE: 'astrum:repository-admin:*:*:delete',
  },
  SSL_TRUSTSTORE: generatePermissions('ssl-truststore'),
  BLOB_STORES: generatePermissions('blobstores', [READ, CREATE, UPDATE, DELETE]),
  PRIVILEGES: generatePermissions('privileges', [READ, CREATE]),
  SELECTORS: generatePermissions('selectors', [READ, CREATE, UPDATE, DELETE]),
  SETTINGS: generatePermissions('settings', [READ, UPDATE]),
  USERS: generatePermissions('users', [READ]),
  ROLES: generatePermissions('roles'),
  TASKS: generatePermissions('tasks', [...crudOptions, START, STOP]),
  LOGGING: generatePermissions('logging', [READ, UPDATE]),
  ATLAS: generatePermissions('atlas', [READ, CREATE]),
  METRICS: generatePermissions('metrics', [READ]),
  BUNDLES: generatePermissions('bundles', [READ]),
  LICENSING: generatePermissions('licensing', [READ, CREATE, UNINSTALL]),
  LDAP: generatePermissions('ldap'),
  USER_TOKENS_SETTINGS: generatePermissions('usertoken-settings', [READ, UPDATE]),
  USER_TOKENS_USERS: generatePermissions('usertoken-users', [DELETE]),
  SERVICE_ACCOUNTS: generatePermissions('service-accounts', [READ, CREATE, DELETE]),
  TAGS: generatePermissions('tags', [READ, CREATE, DELETE]),
  HEALTHCHECK: generatePermissions('healthcheck', [READ, UPDATE]),
  IQ_VIOLATION_SUMMARY: generatePermissions('iq-violation-summary', [READ]),
  COMPONENT: generatePermissions('component', [CREATE]),
  SEARCH: generatePermissions('search', [READ]),
  CAPABILITIES: generatePermissions('capabilities'),
  AUDIT: generatePermissions('audit', [READ]),
};

export default Permissions;

// Named re-export so consumers using `import { Permissions } from '...'` get
// the same dictionary as the default export under Babel CJS interop.
export { Permissions };
