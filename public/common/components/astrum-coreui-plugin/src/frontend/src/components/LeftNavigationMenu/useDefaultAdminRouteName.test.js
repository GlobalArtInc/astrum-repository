import React from 'react';
import { renderHook } from '@testing-library/react';
import { UIRouter } from '@uirouter/react';
import { Permissions } from '@globalart/astrum-ui-plugin';
import { getRouter } from '../../routerConfig/routerConfig';
import givenPermissions from '../../testUtils/givenPermissions';
import givenUserLoggedIn from '../../testUtils/givenUserLoggedIn';
import givenExtJSState from '../../testUtils/givenExtJSState';
import { useDefaultAdminRouteName } from './useDefaultAdminRouteName';

function renderDefaultAdminRouteName() {
  const router = getRouter();
  return renderHook(() => useDefaultAdminRouteName(), {
    wrapper: ({ children }) => <UIRouter router={router}>{children}</UIRouter>,
  });
}

describe('useDefaultAdminRouteName', () => {
  beforeEach(() => {
    givenExtJSState();
    givenUserLoggedIn();
  });

  it('opens the repository section of Astrum One when its pages are visible', () => {
    givenPermissions({ [Permissions.REPOSITORY_ADMIN.READ]: true });

    const { result } = renderDefaultAdminRouteName();

    expect(result.current).toBe('preview.admin.repository');
  });

  it('falls back to the first other section the user may open', () => {
    givenPermissions({ [Permissions.TASKS.READ]: true });

    const { result } = renderDefaultAdminRouteName();

    expect(result.current).toBe('preview.admin.system');
  });

  it('is null without any settings permission, although the sub-pages declare no requirements', () => {
    givenPermissions({});

    const { result } = renderDefaultAdminRouteName();

    expect(result.current).toBeNull();
  });
});
