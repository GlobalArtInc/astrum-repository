import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ADMIN_STATE, renderWithApp } from '../../../test/render';
import { repository } from '../test-fixtures';
import { ShareButton } from './ShareButton';

const SHARE_PRIVILEGE = 'astrum:sharelinks:create';

describe('the share button', () => {
  it('is shown for a hosted repository to a user with the privilege', () => {
    renderWithApp(
      <ShareButton
        repository={repository('files', 'raw', 'hosted')}
        path="/releases/app.tgz"
        prefix={false}
      />,
      { state: { ...ADMIN_STATE, permissions: [SHARE_PRIVILEGE] } },
    );
    expect(screen.getByRole('button', { name: 'Share link' })).toBeInTheDocument();
  });

  it('is hidden without the privilege', () => {
    renderWithApp(
      <ShareButton
        repository={repository('files', 'raw', 'hosted')}
        path="/releases/app.tgz"
        prefix={false}
      />,
      { state: { ...ADMIN_STATE, permissions: ['astrum:search:read'] } },
    );
    expect(screen.queryByRole('button', { name: 'Share link' })).toBeNull();
  });

  it('is hidden on docker and on proxy repositories', () => {
    const state = { ...ADMIN_STATE, permissions: [SHARE_PRIVILEGE] };
    const { unmount } = renderWithApp(
      <ShareButton repository={repository('images', 'docker', 'hosted')} path="/app/tags" prefix={false} />,
      { state },
    );
    expect(screen.queryByRole('button', { name: 'Share link' })).toBeNull();
    unmount();
    renderWithApp(
      <ShareButton
        repository={repository('mirror', 'raw', 'proxy')}
        path="/releases/app.tgz"
        prefix={false}
      />,
      { state },
    );
    expect(screen.queryByRole('button', { name: 'Share link' })).toBeNull();
  });

  it('offers the folder action to the same users, and only where a folder can be shared', () => {
    const privileged = { ...ADMIN_STATE, permissions: [SHARE_PRIVILEGE] };
    const { unmount } = renderWithApp(
      <ShareButton repository={repository('files', 'raw', 'hosted')} path="/releases/" prefix />,
      { state: privileged },
    );
    expect(screen.getByRole('button', { name: 'Share folder' })).toBeInTheDocument();
    unmount();
    const { unmount: unmountWithout } = renderWithApp(
      <ShareButton repository={repository('files', 'raw', 'hosted')} path="/releases/" prefix />,
      { state: { ...ADMIN_STATE, permissions: ['astrum:search:read'] } },
    );
    expect(screen.queryByRole('button', { name: 'Share folder' })).toBeNull();
    unmountWithout();
    const { unmount: unmountAnonymous } = renderWithApp(
      <ShareButton repository={repository('files', 'raw', 'hosted')} path="/releases/" prefix />,
      { state: { ...privileged, user: null } },
    );
    expect(screen.queryByRole('button', { name: 'Share folder' })).toBeNull();
    unmountAnonymous();
    renderWithApp(<ShareButton repository={repository('images', 'docker', 'hosted')} path="/app/" prefix />, {
      state: privileged,
    });
    expect(screen.queryByRole('button', { name: 'Share folder' })).toBeNull();
  });
});
