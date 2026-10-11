

import React, { useState, useEffect, } from 'react';
import { Box, Flex, Text, Tabs, } from '@radix-ui/themes';
import { 
  Shield, 
  Key, 
  Users, 
  Info,
} from 'lucide-react';

import {
  LoadingState,
  ErrorState,
  MetadataGrid,
} from '../../../../shared';
import { SettingsFormSection } from '../../../../shared/form';
import { useRoleTree } from './useRoleTree';
import { RoleExplorerTree } from './RoleExplorerTree';
import { CalculatedPermissions } from './CalculatedPermissions';
import { RoleAssignedUsers } from './RoleAssignedUsers';
import { useRolesApi } from './useRolesApi';
import { Role } from './types';

import './RoleProfilePage.scss';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface RoleProfilePageProps {
  roleName: string;
  onBack: () => void;
  /** When true, cross-entity links (users) render as plain text — used in full-screen modal */
  embedMode?: boolean;
}

/**
 * RoleProfilePage - Dedicated read-only operational dashboard for a Role.
 * Follows the "Profile Pattern" established for Repositories.
 */
export function RoleProfilePage({ roleName, onBack, embedMode = false }: RoleProfilePageProps) {
  const [role, setRole] = useState<Role | null>(null);
  const [roleLoading, setRoleLoading] = useState(true);
  const [roleError, setRoleError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('overview');
  const [treeExpandedNodes, setTreeExpandedNodes] = useState<Set<string>>(() => new Set([roleName]));
  const [treeSearchTerm, setTreeSearchTerm] = useState('');

  useEffect(() => {
    setTreeExpandedNodes(new Set([roleName]));
  }, [roleName]);

  const { findRole } = useRolesApi();
  const { tree, effectivePrivileges, loading: treeLoading, toggleExpand, expandAll, collapseAll } = useRoleTree(roleName, {
    searchTerm: treeSearchTerm,
    initialExpandedNodes: treeExpandedNodes,
    onExpandedNodesChange: setTreeExpandedNodes,
  });

  useEffect(() => {
    const loadRole = async () => {
      setRoleLoading(true);
      try {
        const data = await findRole(roleName);
        setRole(data);
      } catch (err: any) {
        setRoleError(err.message || RolesStrings.PROFILE.FAILED_LOAD_ROLE);
      } finally {
        setRoleLoading(false);
      }
    };

    loadRole();
  }, [roleName, findRole]);

  if (roleLoading) return <LoadingState message={RolesStrings.PROFILE.LOADING_ROLE_PROFILE} />;
  if (roleError) return <ErrorState message={roleError} onRetry={() => window.location.reload()} />;
  if (!role) return <ErrorState message={RolesStrings.PROFILE.ROLE_NOT_FOUND} />;

  return (
    <Box className="role-profile-page">
      <Tabs.Root value={activeTab} onValueChange={setActiveTab}>
        <Tabs.List className="role-profile-page__tabs-list">
          <Tabs.Trigger value="overview">
            <Flex align="center" gap="1">
              <Info size={14} />
              <Text size="2">{RolesStrings.PROFILE.OVERVIEW}</Text>
            </Flex>
          </Tabs.Trigger>
          <Tabs.Trigger value="explorer">
            <Flex align="center" gap="1">
              <Shield size={14} />
              <Text size="2">{RolesStrings.PROFILE.SECURITY_TREE}</Text>
            </Flex>
          </Tabs.Trigger>
          <Tabs.Trigger value="effective">
            <Flex align="center" gap="1">
              <Key size={14} />
              <Text size="2">{RolesStrings.PROFILE.EFFECTIVE_PERMISSIONS}</Text>
            </Flex>
          </Tabs.Trigger>
          <Tabs.Trigger value="users">
            <Flex align="center" gap="1">
              <Users size={14} />
              <Text size="2">{RolesStrings.PROFILE.ASSIGNED_USERS}</Text>
            </Flex>
          </Tabs.Trigger>
        </Tabs.List>

        <Box pt="4" className="role-profile-page__content">
          <Tabs.Content value="overview">
            <SettingsFormSection title={RolesStrings.PROFILE.ROLE_METADATA} defaultOpen>
              <MetadataGrid
                items={[
                  {
                    label: 'ID',
                    value: (
                      <Text size="2" style={{ fontFamily: 'var(--font-mono)' }}>
                        {role.id}
                      </Text>
                    ),
                  },
                  { label: CommonStrings.LABELS.NAME, value: role.name },
                  { label: CommonStrings.LABELS.DESCRIPTION, value: role.description },
                ]}
              />
            </SettingsFormSection>
          </Tabs.Content>

          <Tabs.Content value="explorer">
            <Box className="role-profile-page__pane role-profile-page__pane--tree">
              <RoleExplorerTree
                tree={tree}
                loading={treeLoading}
                onToggleExpand={toggleExpand}
                onExpandAll={expandAll}
                onCollapseAll={collapseAll}
                onSearchChange={setTreeSearchTerm}
              />
            </Box>
          </Tabs.Content>

          <Tabs.Content value="effective">
            <Box className="role-profile-page__pane">
              <CalculatedPermissions
                privileges={effectivePrivileges}
                loading={treeLoading}
                linksDisabled={embedMode}
              />
            </Box>
          </Tabs.Content>

          <Tabs.Content value="users">
            <Box className="role-profile-page__pane">
              <RoleAssignedUsers roleId={roleName} />
            </Box>
          </Tabs.Content>
        </Box>
      </Tabs.Root>
    </Box>
  );
}

export default RoleProfilePage;
