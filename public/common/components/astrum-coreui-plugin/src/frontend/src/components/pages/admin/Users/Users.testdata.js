
import {indexBy, prop} from 'ramda';
import {APIConstants} from '@globalart/astrum-ui-plugin';

const {EXT: {USER: {ACTION, METHODS}}} = APIConstants;

export const ROWS = {
  CROWD: [
    {
      userId: 'test #1',
      version: '0',
      realm: 'Crowd',
      firstName: 'first1',
      lastName: 'last1',
      email: 'first1last1@example.org',
      status: 'active',
      password: '#~NXRM~PLACEHOLDER~PASSWORD~#',
      roles: [],
      external: true,
      externalRoles: []
    },
    {
      userId: 'test #2',
      version: '0',
      realm: 'Crowd',
      firstName: 'first2',
      lastName: 'last2',
      email: 'first2last2@example.org',
      status: 'active',
      password: '#~NXRM~PLACEHOLDER~PASSWORD~#',
      roles: [
        'crowd-administrators'
      ],
      external: true,
      externalRoles: [
        'crowd-administrators'
      ]
    },
    {
      userId: 'test #3',
      version: '0',
      realm: 'Crowd',
      firstName: 'first3',
      lastName: 'last3',
      email: 'first3last3@example.org',
      status: 'active',
      password: '#~NXRM~PLACEHOLDER~PASSWORD~#',
      roles: [
        'group__5',
        'group_3',
      ],
      external: true,
      externalRoles: [
        'group__5',
        'group_3',
      ]
    },
    {
      userId: 'admin',
      version: '0',
      realm: 'Crowd',
      firstName: 'admin crowd',
      lastName: 'admin crowd',
      email: 'admin@example.org',
      status: 'active',
      password: '#~NXRM~PLACEHOLDER~PASSWORD~#',
      roles: [],
      external: true,
      externalRoles: []
    }
  ],
  LOCAL: [
    {
      userId: 'anonymous',
      version: '1',
      realm: 'default',
      firstName: 'Anonymous',
      lastName: 'User',
      email: 'anonymous@example.org',
      status: 'active',
      password: '#~NXRM~PLACEHOLDER~PASSWORD~#',
      roles: [
        'as-anonymous'
      ],
      external: false,
      externalRoles: null
    },
    {
      userId: 'admin',
      version: '1',
      realm: 'default',
      firstName: 'Administrator',
      lastName: 'User',
      email: 'admin@example.org',
      status: 'active',
      password: '#~NXRM~PLACEHOLDER~PASSWORD~#',
      roles: [
        'as-admin'
      ],
      external: false,
      externalRoles: null
    }
  ],
};

export const SOURCES = [
  {
    id: 'default',
    name: 'Local'
  },
  {
    id: 'SAML',
    name: 'SAML'
  },
  {
    id: 'Crowd',
    name: 'Crowd'
  },
  {
    id: 'LDAP',
    name: 'LDAP'
  },
  {
    id: 'allConfigured',
    name: 'All Users with Roles'
  }
];

export const SOURCES_MAP = indexBy(prop('id'), SOURCES);

export const ROLES = {
  'as-admin': {
    description: 'Administrator Role',
    id: 'as-admin',
    name: 'as-admin',
    privileges: ['as-all'],
    roles: [],
    source: 'default'
  },
  'TestRole': {
    description: 'Test role',
    id: 'TestRole',
    name: 'Test Role name',
    privileges: ['as-healthcheck-read'],
    roles: [],
    source: 'default',
  },
  'replication-role': {
    description: 'Replication',
    id: 'replication-role',
    name: 'Replication role',
    privileges: ['as-replication-update'],
    roles: [],
    source: 'default',
  }
};

export const FIELDS = {
  USER_ID: 'userId',
  REALM: 'realm',
  FIRST_NAME: 'firstName',
  LAST_NAME: 'lastName',
  EMAIL: 'email',
  STATUS: 'status',
};

export const DEFAULT_DATA = {
  action: ACTION,
  data: [
    {
      limit: 50,
      page: 1,
      filter: [
        {property: 'userId', value: ''},
        {property: 'source', value: 'default'},
      ],
      start: 0
    }
  ],
  method: METHODS.READ,
  tid: 1,
  type: 'rpc'
};
