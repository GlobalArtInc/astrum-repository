

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { Box } from '@radix-ui/themes';
import { Users, Plus, UserPlus } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { useToast, PageHeader } from '../../../../shared';
import { SettingsButton, SettingsAlert } from '../../../../shared/form';
import { DeleteConfirmationModal } from '../../../../shared/modals/DeleteConfirmationModal';
import { UsersList } from './UsersList';
import { UserDetail } from './UserDetail';
import { UserProfilePage } from './UserProfilePage';
import { InviteUserForm } from './InviteUserForm';
import { EditUserView } from './EditUserView';
import { useUsersApi } from './useUsersApi';
import {
  User,
  DEFAULT_SOURCE,
  getFullName,
  getUserProtectionReason,
} from './types';

import './UsersPage.scss';
import UsersStrings from '../../../../constants/pages/settings/security/UsersStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const BASE_PATH = 'admin/security/users';

/**
 * URL-based routing patterns:
 * - /users                    → List page
 * - /users/create             → Create form
 * - /users/{id}/{source}      → Edit form
 * - /users/{id}/{source}/profile → Profile page (read-only)
 */
type ViewMode = 'list' | 'create' | 'detail' | 'profile' | 'invite';

interface RouteState {
  viewMode: ViewMode;
  userId: string | null;
  source: string | null;
}

function parseRoute(hash: string): RouteState {
  const cleanHash = hash.replace(/^#/, '').replace(/\?.*$/, '');
  const parts = cleanHash.split('/');
  const userIndex = parts.indexOf('users');

  if (userIndex === -1) return { viewMode: 'list', userId: null, source: null };

  const pathAfterUsers = parts.slice(userIndex + 1);
  if (pathAfterUsers.length === 0) return { viewMode: 'list', userId: null, source: null };

  if (pathAfterUsers[0] === 'create') {
    return { viewMode: 'create', userId: null, source: null };
  }

  if (pathAfterUsers[0] === 'invite') {
    return { viewMode: 'invite', userId: null, source: null };
  }

  const userId = decodeURIComponent(pathAfterUsers[0]);
  const source = pathAfterUsers[1] ? decodeURIComponent(pathAfterUsers[1]) : DEFAULT_SOURCE;

  if (pathAfterUsers.length >= 3 && pathAfterUsers[2] === 'profile') {
    return { viewMode: 'profile', userId, source };
  }

  return { viewMode: 'detail', userId, source };
}

function navigateTo(path: string) {
  window.location.hash = path;
}

export function UsersPage() {
  const [routeState, setRouteState] = useState<RouteState>(() => parseRoute(window.location.hash));
  const [user, setUser] = useState<User | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const toast = useToast();

  const {
    loading,
    error,
    setError,
    fetchUser,
    deleteUser,
  } = useUsersApi();

  const canCreate = ExtJS.checkPermission('astrum:users:create');
  const canUpdate = ExtJS.checkPermission('astrum:users:update');
  const canDelete = ExtJS.checkPermission('astrum:users:delete');
  const state = ExtJS.state?.();
  const isCloud = state?.getValue?.('isCloud', false) ?? false;

  const protectionReason = user
    ? getUserProtectionReason(user, {
        anonymousUsername: state?.getValue?.('anonymousUsername'),
        currentUserId: state?.getUser?.()?.id,
      })
    : null;

  useEffect(() => {
    const handleHashChange = () => {
      const newState = parseRoute(window.location.hash);
      setRouteState(newState);
      setError(null);
    };
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, [setError]);

  useEffect(() => {
    if ((routeState.viewMode === 'detail' || routeState.viewMode === 'profile') && routeState.userId) {
      fetchUser(routeState.userId, routeState.source || DEFAULT_SOURCE)
        .then((userData) => {
          if (userData) {
            setUser(userData);
          } else {
            setError(UsersStrings.PAGE.USER_NOT_FOUND);
            navigateTo(BASE_PATH);
          }
        })
        .catch((err) => {
          setError(err.message);
          navigateTo(BASE_PATH);
        });
    } else {
      setUser(null);
    }
  }, [routeState.viewMode, routeState.userId, routeState.source, fetchUser, setError, refreshKey]);

  const handleViewUser = useCallback((userId: string, realm: string) => {
    navigateTo(`${BASE_PATH}/${encodeURIComponent(userId)}/${encodeURIComponent(realm)}/profile`);
  }, []);

  const handleEditUser = useCallback((userId: string, realm: string) => {
    navigateTo(`${BASE_PATH}/${encodeURIComponent(userId)}/${encodeURIComponent(realm)}`);
  }, []);

  const handleRowClick = useCallback((userId: string, realm: string) => {
    if (canUpdate) {
      handleEditUser(userId, realm);
    } else {
      handleViewUser(userId, realm);
    }
  }, [canUpdate, handleEditUser, handleViewUser]);

  const handleCreate = useCallback(() => {
    navigateTo(`${BASE_PATH}/create`);
  }, []);

  const handleInvite = useCallback(() => {
    navigateTo(`${BASE_PATH}/invite`);
  }, []);

  const handleBack = useCallback(() => {
    navigateTo(BASE_PATH);
  }, []);

  const bumpRefresh = useCallback(() => {
    setRefreshKey((k) => k + 1);
  }, []);

  const handleSaveSuccess = useCallback(async () => {
    bumpRefresh();
  }, [bumpRefresh]);

  const handleDelete = useCallback(() => {
    if (!user) return;
    setDeleteDialogOpen(true);
  }, [user]);

  const handleDeleteConfirm = useCallback(async () => {
    if (!user?.userId) return;

    setError(null);
    setIsDeleting(true);
    try {
      await deleteUser(user.userId);
      toast.success(UsersStrings.PAGE.USER_DELETED_SUCCESSFULLY(getFullName(user) || user.userId));
      bumpRefresh();
      setDeleteDialogOpen(false);
      handleBack();
    } catch (err: any) {
      setError(err.message);
    } finally {
      setIsDeleting(false);
    }
  }, [user, deleteUser, handleBack, toast, setError, bumpRefresh]);

  const isEditOrCreate =
    routeState.viewMode === 'create' ||
    (routeState.viewMode === 'detail' && canUpdate);

  const listHeaderProps = useMemo(() => ({
    icon: Users,
    title: UsersStrings.PAGE.USERS,
    description: UsersStrings.PAGE.MANAGE_USERS_THEIR_ROLE_ASSIGNMENTS,
    actions: canCreate ? (
      isCloud ? (
        <SettingsButton testId="invite-user-button" variant="primary" onClick={handleInvite} icon={UserPlus}>
          {UsersStrings.PAGE.INVITE_USER}
        </SettingsButton>
      ) : (
        <SettingsButton testId="create-user-button" variant="primary" onClick={handleCreate} icon={Plus}>
          {UsersStrings.PAGE.CREATE_LOCAL_USER}
        </SettingsButton>
      )
    ) : undefined,
  }), [canCreate, isCloud, handleCreate, handleInvite]);

  return (
    <Box
      className="users-page"
      data-testid="users-page"
      data-view={routeState.viewMode}
      data-loading={loading ? 'true' : 'false'}
      aria-busy={loading}
    >
      {routeState.viewMode === 'list' && (
        <PageHeader
          icon={listHeaderProps.icon}
          title={listHeaderProps.title}
          description={listHeaderProps.description}
          actions={listHeaderProps.actions}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
            { label: UsersStrings.PAGE.USERS },
          ]}
        />
      )}

      {routeState.viewMode === 'invite' && (
        <PageHeader
          icon={Users}
          title={UsersStrings.PAGE.INVITE_USER}
          description={UsersStrings.PAGE.SEND_INVITATION_NEW_USER}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
            { label: UsersStrings.PAGE.USERS, onClick: handleBack },
            { label: UsersStrings.PAGE.INVITE },
          ]}
        />
      )}

      {routeState.viewMode === 'detail' && !canUpdate && (
        <PageHeader
          icon={Users}
          title={user ? getFullName(user) || user.userId : UsersStrings.PAGE.USER_DETAILS}
          description={UsersStrings.PAGE.READ_ONLY_VIEW}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#admin/settings') },
            { label: UsersStrings.PAGE.USERS, onClick: handleBack },
            { label: user?.userId || CommonStrings.MESSAGES.LOADING },
          ]}
        />
      )}

      {error && (
        <Box className="users-page__alerts">
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      <Box className="users-page__content">
        {routeState.viewMode === 'list' && (
          <UsersList
            key={refreshKey}
            onSelect={handleRowClick}
            getRowAriaLabel={(u) => `${canUpdate ? CommonStrings.ACTIONS.EDIT : UsersStrings.PAGE.VIEW} ${getFullName(u) || u.userId}`}
            onCreate={isCloud ? handleInvite : handleCreate}
            isCloud={isCloud}
          />
        )}

        {isEditOrCreate && (
          <EditUserView
            key={refreshKey}
            isCreate={routeState.viewMode === 'create'}
            userId={routeState.viewMode === 'detail' ? routeState.userId : undefined}
            userSource={routeState.viewMode === 'detail' ? (routeState.source || DEFAULT_SOURCE) : undefined}
            user={routeState.viewMode === 'detail' ? user : null}
            canDelete={canDelete}
            protectionReason={protectionReason}
            onDeleteRequest={handleDelete}
            onSuccess={handleSaveSuccess}
            onCancel={handleBack}
            isDeleting={isDeleting}
          />
        )}

        {routeState.viewMode === 'profile' && (
          <UserProfilePage
            userId={routeState.userId!}
            userSource={routeState.source || DEFAULT_SOURCE}
            onBack={handleBack}
            onEdit={() =>
              navigateTo(
                `${BASE_PATH}/${encodeURIComponent(routeState.userId!)}/${encodeURIComponent(routeState.source || DEFAULT_SOURCE)}`,
              )
            }
            canEdit={canUpdate}
          />
        )}

        {routeState.viewMode === 'detail' && !canUpdate && (
          <UserDetail
            user={user}
            loading={loading && !user}
            canEdit={false}
            onCancel={handleBack}
          />
        )}

        {routeState.viewMode === 'invite' && (
          <InviteUserForm
            onSuccess={(email) => {
              toast.success(
                UsersStrings.PAGE.INVITATION_EMAIL_SENT,
                UsersStrings.PAGE.INVITATION_EMAIL_HAS_BEEN_SENT(email),
              );
              handleBack();
              bumpRefresh();
            }}
            onCancel={handleBack}
            loading={loading}
            error={error || undefined}
          />
        )}
      </Box>

      <DeleteConfirmationModal
        open={deleteDialogOpen}
        testId="delete-user-dialog"
        data-analytics-id="as-user-delete"
        onClose={() => setDeleteDialogOpen(false)}
        onConfirm={handleDeleteConfirm}
        entityName={user ? (getFullName(user) || user.userId) : ''}
        entityType="user"
        loading={isDeleting}
      />
    </Box>
  );
}

export default UsersPage;
