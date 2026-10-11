

import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { Box, Button, Flex, Text } from '@radix-ui/themes';
import { Shield, Plus, ArrowLeft, Trash2, Pencil } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import Permissions from '../../../../../../constants/Permissions';

import { useToast, PageHeader } from '../../../../shared';
import { SettingsAlert, ConfirmDialog, WizardForm, SettingsButton } from '../../../../shared/form';
import { RolesList } from './RolesList';
import { RoleForm } from './RoleForm';
import { RoleProfilePage } from './RoleProfilePage';
import { useRolesApi } from './useRolesApi';
import { Role, isReadOnlyRole, formatRoleSourceDisplay } from './types';

import './RolesPage.scss';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import plural from '../../../../../../interface/plural';

// Base path for role URLs
const BASE_PATH = 'preview/admin/security/roles';

/**
 * URL-based routing patterns:
 * - /roles           → List page
 * - /roles/create    → Wizard (Step 1)
 * - /roles/{id}      → Edit form
 * - /roles/{id}/profile → Profile page
 */
type ViewMode = 'list' | 'create' | 'edit' | 'profile';

interface RouteState {
  viewMode: ViewMode;
  roleId: string | null;
}

function parseRoute(hash: string): RouteState {
  const cleanHash = hash.replace(/^#/, '').replace(/\?.*$/, '');
  const parts = cleanHash.split('/');
  const roleIndex = parts.indexOf('roles');
  
  if (roleIndex === -1) return { viewMode: 'list', roleId: null };

  const pathAfterRoles = parts.slice(roleIndex + 1);
  if (pathAfterRoles.length === 0) return { viewMode: 'list', roleId: null };

  if (pathAfterRoles[0] === 'create') {
    return { viewMode: 'create', roleId: null };
  }

  const roleId = decodeURIComponent(pathAfterRoles[0]);
  if (pathAfterRoles.length >= 2 && pathAfterRoles[1] === 'profile') {
    return { viewMode: 'profile', roleId };
  }

  return { 
    viewMode: 'edit', 
    roleId 
  };
}

function navigateTo(path: string) {
  window.location.hash = path;
}

/**
 * RolesPage - Main Roles management page for Preview UI
 * 
 * Displays role list with search/filter, and allows creating, editing, and deleting roles.
 * Uses a multi-step wizard for creation and editing.
 */
export function RolesPage() {
  const [routeState, setRouteState] = useState<RouteState>(() => parseRoute(window.location.hash));
  const [role, setRole] = useState<Role | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [listDeleteRoleId, setListDeleteRoleId] = useState<string | null>(null);
  const [listDeleteRoleName, setListDeleteRoleName] = useState<string | null>(null);
  const [internalWizardStep, setInternalWizardStep] = useState(0);

  // Toast notifications (app-level provider)
  const toast = useToast();

  const {
    loading,
    error,
    setError,
    findRole,
    deleteRole,
  } = useRolesApi();

  // Provider-independent, reactive permission checks (ASTRUM-54212). ExtJS.checkPermission on
  // its own evaluates once at render and would briefly disable the Create/Edit/Delete controls
  // for a permitted non-admin if permissions load asynchronously after mount; ExtJS.usePermission
  // with a hasUser dependency re-evaluates once the user and their permissions arrive. Matches the
  // pattern used by ContentSelectorsPage and CapabilityDetail.
  const hasUser = ExtJS.useUser() ?? false;
  const canCreate = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.ROLES.CREATE), [hasUser]);
  const canUpdate = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.ROLES.UPDATE), [hasUser]);
  const canDelete = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.ROLES.DELETE), [hasUser]);

  // Handle hash changes for routing
  useEffect(() => {
    const handleHashChange = () => {
      const newState = parseRoute(window.location.hash);
      setRouteState(newState);
      setError(null);
      setInternalWizardStep(0);
    };
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, [setError]);

  // Load role details when in edit or profile mode
  useEffect(() => {
    if ((routeState.viewMode === 'edit' || routeState.viewMode === 'profile') && routeState.roleId) {
      findRole(routeState.roleId)
        .then((roleData) => {
          if (roleData) {
            setRole(roleData);
          } else {
            setError(RolesStrings.PAGE.ROLE_NOT_FOUND);
            navigateTo(BASE_PATH);
          }
        })
        .catch((err) => {
          setError(err.message);
          navigateTo(BASE_PATH);
        });
    } else {
      setRole(null);
    }
  }, [routeState.viewMode, routeState.roleId, findRole, setError]);

  const handleCreate = useCallback(() => {
    navigateTo(`${BASE_PATH}/create`);
  }, []);

  const handleBack = useCallback(() => {
    navigateTo(BASE_PATH);
  }, []);

  const handleWizardStepChange = useCallback((step: number) => {
    setInternalWizardStep(step);
  }, []);

  const roleFormSubmitRef = useRef<(() => void) | null>(null);
  const [isRoleFormValid, setIsRoleFormValid] = useState(false);

  const handleFinalSubmit = useCallback(() => {
    if (roleFormSubmitRef.current) {
      roleFormSubmitRef.current();
    }
  }, []);

  const handleComplete = useCallback(() => {
    setRefreshKey((k) => k + 1);
    navigateTo(BASE_PATH);
  }, []);

  const handleDelete = useCallback(() => {
    if (!role) return;
    setDeleteDialogOpen(true);
  }, [role]);

  const handleDeleteConfirm = useCallback(async () => {
    const roleIdToDelete = listDeleteRoleId || role?.id;
    if (!roleIdToDelete) return;
    
    setDeleteDialogOpen(false);
    setListDeleteRoleId(null);
    setListDeleteRoleName(null);
    try {
      await deleteRole(roleIdToDelete);
      toast.success(RolesStrings.PAGE.ROLE_DELETED_SUCCESSFULLY(listDeleteRoleName || roleIdToDelete));
      setRefreshKey((k) => k + 1);
      if (role && role.id === roleIdToDelete) handleBack();
    } catch (_err) {
      // Error is set by the API hook
    }
  }, [role, listDeleteRoleId, deleteRole, handleBack, toast, listDeleteRoleName]);

  const handleListDelete = useCallback((roleId: string, roleName?: string) => {
    setListDeleteRoleId(roleId);
    setListDeleteRoleName(roleName || null);
    setDeleteDialogOpen(true);
  }, []);

  // Determine if role can be edited/deleted
  const _roleCanEdit = canUpdate && role && !isReadOnlyRole(role);
  const roleCanDelete = canDelete && role && !isReadOnlyRole(role);

  // Header configuration
  const headerProps = useMemo(() => {
    switch (routeState.viewMode) {
      case 'list':
        return {
          icon: Shield,
          title: RolesStrings.PAGE.ROLES,
          description: RolesStrings.PAGE.MANAGE_ROLES_THEIR_PRIVILEGE_ASSIGNMENTS,
          actions: (
            <Button variant="solid" color="blue" onClick={handleCreate} disabled={!canCreate} data-testid="create-role-button" data-analytics-id="as-role-create">
              <Plus size={16} /> {RolesStrings.PAGE.CREATE_ROLE}
            </Button>
          )
        };
      case 'create': {
        const createSteps = [CommonStrings.LABELS.TYPE, RolesStrings.PAGE.SETUP, RolesStrings.PAGE.PRIVILEGES, RolesStrings.PAGE.CONTAINED_ROLES];
        return {
          icon: Shield,
          title: RolesStrings.PAGE.CREATE_ROLE,
          description: RolesStrings.PAGE.STEP(internalWizardStep + 1, createSteps[internalWizardStep]),
          actions: (
            <Button variant="soft" color="gray" onClick={handleBack}>
              <ArrowLeft size={16} /> {CommonStrings.ACTIONS.BACK_TO_LIST}
            </Button>
          )
        };
      }
      case 'edit': {
        // Type step is skipped in edit mode (role type is immutable once persisted),
        // so the edit wizard runs Setup → Privileges → Contained Roles.
        const editSteps = [RolesStrings.PAGE.SETUP, RolesStrings.PAGE.PRIVILEGES, RolesStrings.PAGE.CONTAINED_ROLES];
        return {
          icon: Shield,
          title: role ? RolesStrings.PAGE.EDIT_NAMED(role.name) : RolesStrings.PAGE.ROLE_DETAILS,
          description: role
            ? RolesStrings.PAGE.STEP_SOURCE(internalWizardStep + 1, editSteps[internalWizardStep], formatRoleSourceDisplay(role.source))
            : CommonStrings.MESSAGES.LOADING,
          actions: (
            <Button variant="soft" color="gray" onClick={handleBack}>
              <ArrowLeft size={16} /> {CommonStrings.ACTIONS.BACK_TO_LIST}
            </Button>
          )
        };
      }
      case 'profile': {
        const canEditThisRole = canUpdate && role && !isReadOnlyRole(role);
        return {
          icon: Shield,
          title: routeState.roleId || RolesStrings.PAGE.ROLE_PROFILE,
          description: RolesStrings.PAGE.ROLE_OPERATIONAL_DASHBOARD_IMPACT_ANALYSIS,
          actions: (
            <Flex gap="2">
              {canEditThisRole && (
                <Button
                  variant="solid"
                  color="blue"
                  onClick={() => navigateTo(`${BASE_PATH}/${encodeURIComponent(routeState.roleId!)}`)}
                >
                  <Pencil size={16} /> {RolesStrings.PAGE.EDIT_ROLE}
                </Button>
              )}
              <Button variant="soft" color="gray" onClick={handleBack}>
                <ArrowLeft size={16} /> {CommonStrings.ACTIONS.BACK_TO_LIST}
              </Button>
            </Flex>
          )
        };
      }
      default:
        return {
          icon: Shield,
          title: RolesStrings.PAGE.ROLES,
          description: RolesStrings.PAGE.MANAGE_ROLES
        };
    }
  }, [routeState.viewMode, role, canCreate, handleCreate, handleBack, internalWizardStep, canUpdate, routeState.roleId]);

  return (
    <Box 
      className="roles-page"
      data-testid="roles-page"
      data-view={routeState.viewMode}
      data-loading={loading ? 'true' : 'false'}
    >
      {/* Alerts */}
      {error && (
        <Box className="roles-page__alerts">
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Content */}
      <Box className="roles-page__content">
        {routeState.viewMode === 'list' && (
          <>
            <PageHeader 
              icon={headerProps.icon} 
              title={headerProps.title} 
              description={headerProps.description} 
              actions={headerProps.actions} 
            
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: RolesStrings.PAGE.ROLES }
          ]}
/>
            <RolesList
              key={refreshKey}
              onSelect={(roleId, mode) => {
                const path = mode === 'edit'
                  ? `${BASE_PATH}/${encodeURIComponent(roleId)}`
                  : `${BASE_PATH}/${encodeURIComponent(roleId)}/profile`;
                navigateTo(path);
              }}
              onDelete={canDelete ? handleListDelete : undefined}
              onCreate={handleCreate}
              canUpdate={canUpdate}
              canDelete={canDelete}
            />
          </>
        )}

        {routeState.viewMode === 'profile' && routeState.roleId && (
          <>
            <PageHeader
              icon={Shield}
              title={headerProps.title}
              description={headerProps.description}
              breadcrumbs={[
                { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
                { label: RolesStrings.PAGE.ROLES, onClick: handleBack },
                { label: routeState.roleId || RolesStrings.PAGE.ROLE_PROFILE },
              ]}
              actions={headerProps.actions}
            />
            <RoleProfilePage 
              roleName={routeState.roleId} 
              onBack={handleBack} 
            />
          </>
        )}

        {(routeState.viewMode === 'create' || (routeState.viewMode === 'edit' && role)) && (() => {
          // Edit mode skips the Type step (role type is immutable once persisted),
          // so the wizard is 3 steps: Setup → Privileges → Contained Roles.
          // RoleForm still interprets `wizardStep` in its logical sense (0=Type, 1=Setup,
          // 2=Privileges, 3=Roles), so in edit mode we shift by +1 at the boundary.
          const isCreateView = routeState.viewMode === 'create';
          const steps = isCreateView
            ? [
                { id: 'type', label: CommonStrings.LABELS.TYPE },
                { id: 'setup', label: RolesStrings.PAGE.SETUP },
                { id: 'privileges', label: RolesStrings.PAGE.PRIVILEGES },
                { id: 'roles', label: RolesStrings.PAGE.CONTAINED_ROLES },
              ]
            : [
                { id: 'setup', label: RolesStrings.PAGE.SETUP },
                { id: 'privileges', label: RolesStrings.PAGE.PRIVILEGES },
                { id: 'roles', label: RolesStrings.PAGE.CONTAINED_ROLES },
              ];
          const setupStepIndex = isCreateView ? 1 : 0;
          const finalStepIndex = isCreateView ? 3 : 2;
          const logicalWizardStep = isCreateView ? internalWizardStep : internalWizardStep + 1;
          return (
          <WizardForm
            steps={steps}
            currentStep={internalWizardStep}
            onStepChange={handleWizardStepChange}
            onComplete={handleFinalSubmit}
            onCancel={handleBack}
            completeLabel={isCreateView ? RolesStrings.PAGE.CREATE_ROLE : CommonStrings.ACTIONS.SAVE}
            submitAnalyticsId={isCreateView ? 'as-role-create' : 'as-role-save'}
            dirty={false}
            canAdvance={
              internalWizardStep === setupStepIndex || internalWizardStep === finalStepIndex
                ? isRoleFormValid
                : true
            }
            loading={loading && internalWizardStep === finalStepIndex}
            noDirtyTracking={true}
            title={headerProps.title}
            description={headerProps.description}
            footerExtra={
              <Flex align="center" gap="3">
                {headerProps.actions}
                {routeState.viewMode === 'edit' && roleCanDelete && (
                  <SettingsButton variant="danger" onClick={handleDelete} disabled={loading} icon={Trash2}>
                    {RolesStrings.PAGE.DELETE_ROLE}
                  </SettingsButton>
                )}
              </Flex>
            }
          >
            <RoleForm
              isCreate={isCreateView}
              role={role}
              onCancel={handleBack}
              onComplete={handleComplete}
              loading={loading}
              onSubmitRef={roleFormSubmitRef}
              onValidationChange={setIsRoleFormValid}
              wizardStep={logicalWizardStep}
              hideActions
            />
          </WizardForm>
          );
        })()}
      </Box>

      {/* Delete Confirmation Dialog (edit view or list) */}
      {(roleCanDelete || listDeleteRoleId) && (
        <ConfirmDialog
          open={deleteDialogOpen}
          testId="delete-role-dialog"
          data-analytics-id="as-role-delete"
          onOpenChange={(open) => {
            setDeleteDialogOpen(open);
            if (!open) setListDeleteRoleId(null);
          }}
          title={RolesStrings.PAGE.DELETE_ROLE_CONFIRM}
          message={RolesStrings.PAGE.ACTION_CANNOT_UNDONE}
          entityName={listDeleteRoleName || listDeleteRoleId || role?.name}
          confirmLabel={CommonStrings.ACTIONS.DELETE}
          cancelLabel={CommonStrings.ACTIONS.CANCEL}
          variant="danger"
          onConfirm={handleDeleteConfirm}
        >
          {role && (
            <Box mt="3">
              <Flex gap="2">
                <Text size="2" color="gray">{RolesStrings.PAGE.PRIVILEGES_LABEL}</Text>
                <Text size="2">{role.privileges.length === 0 ? RolesStrings.PAGE.NONE : plural(RolesStrings.PAGE.PRIVILEGE_COUNT, role.privileges.length)}</Text>
              </Flex>
            </Box>
          )}
          <Box mt="3">
            <Text as="p" size="2" color="gray">
              {RolesStrings.PAGE.USERS_ASSIGNED_ROLE_LOSE_ACCESS}
            </Text>
          </Box>
        </ConfirmDialog>
      )}
    </Box>
  );
}

export default RolesPage;
