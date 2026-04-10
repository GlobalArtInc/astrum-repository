import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../../i18n';
import type { Privilege } from '../api';
import { EMPTY_VALUES, fromPrivilege, permissionOf } from './model';
import {
  contentErrors,
  formatOptions,
  fromSelectorTarget,
  repositoryOptions,
  requestOf,
  selectorTarget,
  selectorTargetOptions,
  settingsErrors,
  stepsOf,
} from './PrivilegePage';
import { filterPrivileges } from './PrivilegesPage';

const t = makeTranslate('en');

const privilege = (overrides: Partial<Privilege>): Privilege => ({
  type: 'application',
  name: 'p',
  description: '',
  readOnly: false,
  ...overrides,
});

describe('the permission of a privilege', () => {
  it('writes the actions as the old pages did, in their order', () => {
    expect(permissionOf(privilege({ domain: 'accesstokens', actions: ['ADD', 'READ'] }))).toBe(
      'astrum:accesstokens:add,read',
    );
    expect(permissionOf(privilege({ domain: 'users', actions: ['ALL'] }))).toBe('astrum:users:all');
  });

  it('names repository actions in the same way', () => {
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
    ).toBe('astrum:repository-admin:*:*:all');
  });

  it('puts the selector of a content selector privilege after the repository', () => {
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
    ).toBe('astrum:repository-content-selector:raw:files:team:read,browse');
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
  const repositories = [
    { name: 'b-raw', format: 'raw', type: 'hosted' },
    { name: 'a-raw', format: 'raw', type: 'hosted' },
    { name: 'central', format: 'maven2', type: 'proxy' },
  ];

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
    expect(requestOf('repository-content-selector', values)).toEqual({
      name: 'team-read',
      description: 'Team files',
      contentSelector: 'team',
      format: 'raw',
      repository: 'files',
      actions: ['READ'],
    });
    expect(requestOf('wildcard', { ...values, pattern: 'astrum:*' })).toEqual({
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

  it('asks for the name first and then for the fields of the type', () => {
    expect(settingsErrors(t, EMPTY_VALUES)).toEqual({ name: 'Fill this in' });
    expect(settingsErrors(t, { ...EMPTY_VALUES, name: '.bad' })).toEqual({
      name: expect.stringContaining('letters'),
    });
    expect(contentErrors(t, 'application', EMPTY_VALUES)).toEqual({
      domain: 'Fill this in',
      actions: 'Choose at least one action',
    });
    expect(contentErrors(t, 'wildcard', { ...EMPTY_VALUES, pattern: 'astrum:*' })).toEqual({});
    expect(stepsOf('repository-content-selector')).toEqual(['settings', 'content', 'preview']);
    expect(stepsOf('repository-view')).toEqual(['settings', 'content']);
  });

  it('offers the formats in use and the repositories of the chosen format', () => {
    expect(formatOptions(t, repositories).map((option) => option.value)).toEqual(['*', 'maven2', 'raw']);
    expect(repositoryOptions(t, repositories, 'raw', '*').map((option) => option.value)).toEqual([
      '*',
      'b-raw',
      'a-raw',
    ]);
    expect(repositoryOptions(t, repositories, 'raw', 'gone').map((option) => option.value)).toEqual([
      '*',
      'gone',
      'b-raw',
      'a-raw',
    ]);
  });

  it('picks the format of a content selector privilege with its repository', () => {
    expect(selectorTargetOptions(t, repositories, '*').map((option) => option.label)).toEqual([
      '(All Repositories)',
      '(All maven2 Repositories)',
      '(All raw Repositories)',
      'b-raw',
      'a-raw',
      'central',
    ]);
    expect(fromSelectorTarget('format:raw', repositories)).toEqual({ format: 'raw', repository: '*' });
    expect(fromSelectorTarget('central', repositories)).toEqual({ format: 'maven2', repository: 'central' });
    expect(selectorTarget({ format: 'npm', repository: '*' })).toBe('format:npm');
    expect(selectorTarget({ format: 'npm', repository: 'npmjs' })).toBe('npmjs');
  });
});

describe('the privilege list filter', () => {
  const list = [
    privilege({
      name: 'as-users-read',
      domain: 'users',
      actions: ['READ'],
      description: 'Read users',
      readOnly: true,
    }),
    privilege({ name: 'team', type: 'wildcard', pattern: 'astrum:search:read' }),
  ];
  const names = (items: Privilege[]) => items.map((item) => item.name);

  it('filters by type, by status and by words in name, description and permission', () => {
    expect(names(filterPrivileges(list, { type: ['wildcard'] }, ''))).toEqual(['team']);
    expect(names(filterPrivileges(list, { status: ['locked'] }, ''))).toEqual(['as-users-read']);
    expect(names(filterPrivileges(list, { status: ['locked', 'unlocked'] }, ''))).toHaveLength(2);
    expect(names(filterPrivileges(list, {}, 'astrum:users'))).toEqual(['as-users-read']);
    expect(names(filterPrivileges(list, {}, 'read USERS'))).toEqual(['as-users-read']);
    expect(filterPrivileges(list, { type: ['application'] }, 'search')).toEqual([]);
  });
});
