import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../../i18n';
import type { Privilege, PrivilegeType } from '../api';
import {
  EMPTY_VALUES,
  fromPrivilege,
  permissionOf,
  toggleAction,
  toRequest,
  validatePrivilege,
} from './model';
import { formatOptions, repositoryOptions } from './PrivilegePage';
import { filterPrivileges } from './PrivilegesPage';

const t = makeTranslate('en');

const text = (id: string, required = true) => ({ id, kind: 'text' as const, required, any: false });

const TYPES: Record<string, PrivilegeType> = {
  application: {
    id: 'application',
    fields: [
      text('name'),
      text('description', false),
      text('domain'),
      { id: 'actions', kind: 'actions', required: true, values: ['READ', 'EDIT', 'ALL'], any: false },
    ],
  },
  wildcard: {
    id: 'wildcard',
    fields: [
      text('name'),
      text('description', false),
      { id: 'pattern', kind: 'permission', required: true, any: false },
    ],
  },
  'repository-content-selector': {
    id: 'repository-content-selector',
    fields: [
      text('name'),
      text('description', false),
      { id: 'contentSelector', kind: 'contentSelector', required: true, any: false },
      { id: 'format', kind: 'format', required: true, values: ['*', 'raw', 'maven2'], any: true },
      { id: 'repository', kind: 'repository', required: true, any: true },
      { id: 'actions', kind: 'actions', required: true, values: ['BROWSE', 'READ', 'ALL'], any: false },
    ],
  },
};

const privilege = (overrides: Partial<Privilege>): Privilege => ({
  type: 'application',
  name: 'p',
  description: '',
  readOnly: false,
  ...overrides,
});

describe('the permission of a privilege', () => {
  it('maps application actions to the server names, sorted', () => {
    expect(permissionOf(privilege({ domain: 'users', actions: ['EDIT', 'READ', 'ADD'] }))).toBe(
      'astrum:users:create,read,update',
    );
    expect(permissionOf(privilege({ domain: 'users', actions: ['READ', 'ALL'] }))).toBe('astrum:users:*');
  });

  it('maps repository actions to their own names', () => {
    expect(
      permissionOf(
        privilege({
          type: 'repository-view',
          format: 'maven2',
          repository: 'central',
          actions: ['ADD', 'EDIT'],
        }),
      ),
    ).toBe('astrum:repository-view:maven2:central:add,edit');
    expect(
      permissionOf(privilege({ type: 'repository-admin', format: '*', repository: '*', actions: ['ALL'] })),
    ).toBe('astrum:repository-admin:*:*:*');
  });

  it('includes the selector of a content selector privilege', () => {
    expect(
      permissionOf(
        privilege({
          type: 'repository-content-selector',
          contentSelector: 'team',
          format: 'raw',
          repository: 'files',
          actions: ['READ', 'BROWSE'],
        }),
      ),
    ).toBe('astrum:repository-content-selector:team:raw:files:browse,read');
  });

  it('shows a wildcard as written and nothing for incomplete privileges', () => {
    expect(permissionOf(privilege({ type: 'wildcard', pattern: 'astrum:search:read' }))).toBe(
      'astrum:search:read',
    );
    expect(permissionOf(privilege({ domain: 'users', actions: [] }))).toBeNull();
    expect(permissionOf(privilege({ type: 'script' }))).toBeNull();
  });
});

describe('the privilege form', () => {
  it('sends only the fields of the type', () => {
    const values = {
      ...EMPTY_VALUES,
      name: ' team-read ',
      description: 'Team files',
      domain: 'ignored',
      contentSelector: 'team',
      format: 'raw',
      repository: 'files',
      actions: ['READ' as const],
    };
    expect(toRequest(TYPES['repository-content-selector'] as PrivilegeType, values)).toEqual({
      name: 'team-read',
      description: 'Team files',
      contentSelector: 'team',
      format: 'raw',
      repository: 'files',
      actions: ['READ'],
    });
    expect(toRequest(TYPES.wildcard as PrivilegeType, { ...values, pattern: 'astrum:*' })).toEqual({
      name: 'team-read',
      description: 'Team files',
      pattern: 'astrum:*',
    });
  });

  it('reads a stored privilege back into the form', () => {
    expect(fromPrivilege(privilege({ name: 'x', domain: 'users', actions: ['READ'] }))).toEqual({
      ...EMPTY_VALUES,
      name: 'x',
      domain: 'users',
      actions: ['READ'],
    });
  });

  it('keeps All apart from the other actions', () => {
    expect(toggleAction(['READ', 'EDIT'], 'ALL', true)).toEqual(['ALL']);
    expect(toggleAction(['ALL'], 'READ', true)).toEqual(['READ']);
    expect(toggleAction(['READ'], 'EDIT', true)).toEqual(['READ', 'EDIT']);
    expect(toggleAction(['READ', 'EDIT'], 'READ', false)).toEqual(['EDIT']);
  });

  it('asks for the required fields of the type and checks the name', () => {
    expect(validatePrivilege(t, TYPES.application as PrivilegeType, EMPTY_VALUES)).toEqual({
      name: 'Fill this in',
      domain: 'Fill this in',
      actions: 'Choose at least one action',
    });
    expect(
      validatePrivilege(t, TYPES.wildcard as PrivilegeType, {
        ...EMPTY_VALUES,
        name: '.bad',
        pattern: 'astrum:*',
      }),
    ).toEqual({ name: expect.stringContaining('letters') });
  });

  it('offers all formats and the repositories of the chosen format', () => {
    expect(formatOptions(t, ['*', 'maven2']).map((option) => option.label)).toEqual(['All formats', 'Maven']);
    const repositories = [
      { name: 'b-raw', format: 'raw', type: 'hosted' },
      { name: 'a-raw', format: 'raw', type: 'hosted' },
      { name: 'central', format: 'maven2', type: 'proxy' },
    ];
    expect(repositoryOptions(t, repositories, 'raw', '*').map((option) => option.value)).toEqual([
      '*',
      'a-raw',
      'b-raw',
    ]);
    expect(repositoryOptions(t, repositories, 'raw', 'gone').map((option) => option.value)).toEqual([
      '*',
      'gone',
      'a-raw',
      'b-raw',
    ]);
  });
});

describe('the privilege list filter', () => {
  const list = [
    privilege({ name: 'as-users-read', domain: 'users', actions: ['READ'], description: 'Read users' }),
    privilege({ name: 'team', type: 'wildcard', pattern: 'astrum:search:read' }),
  ];

  it('filters by type and by words in name, description and permission', () => {
    expect(filterPrivileges(list, 'wildcard', '').map((item) => item.name)).toEqual(['team']);
    expect(filterPrivileges(list, 'all', 'astrum:users').map((item) => item.name)).toEqual(['as-users-read']);
    expect(filterPrivileges(list, 'all', 'read USERS').map((item) => item.name)).toEqual(['as-users-read']);
    expect(filterPrivileges(list, 'application', 'search')).toEqual([]);
  });
});
