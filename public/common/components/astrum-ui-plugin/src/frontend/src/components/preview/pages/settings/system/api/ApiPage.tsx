

import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { ExternalLink, Info } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { usePrivilegesApi } from '../../security/privileges/usePrivilegesApi';
import type { Privilege } from '../../security/privileges/types';
import { useRolesApi } from '../../security/roles/useRolesApi';
import type { Role } from '../../security/roles/types';
import { SettingsAlert } from '../../../../shared/form';
import { PageHeader } from '../../../../shared';
import { ApiLayout } from './ApiLayout';
import { EndpointDetail } from './EndpointDetail';
import { EndpointList, endpointRowId } from './EndpointList';
import { useApiModuleDeepLink } from './hooks/useApiModuleDeepLink';
import { useApiPage } from './useApiPage';
import { useEndpointPermissions } from './hooks/useEndpointPermissions';
import { useViewAsUserAccess } from './hooks/useViewAsUserAccess';
import type { ApiPageProps } from './types';
import {
  filterEndpointsByPermissionSubstring,
  findEndpointByDeepLink,
} from './utils/apiModuleDeepLinkParams';
import { readCurrentSessionUserId } from './utils/currentSessionUser';
import { computeEndpointAccess } from './utils/endpointAccess';
import { filterSwaggerSpecForCloud } from './utils/filterSwaggerForCloud';
import { mergeSwaggerAndPermissions, type MergedApiEndpoint } from './utils/mergeSwaggerPermissions';
import { readAstrumPermissionMap } from './utils/astrumPermissionMap';
import { computeRoleLensAccessById } from './utils/roleLensAccess';

import './ApiPage.scss';
import ApiStrings from '../../../../constants/pages/settings/system/ApiStrings';
import CommonStrings from '../../../../constants/CommonStrings';

function isAdminUser(): boolean {
  try {
    return ExtJS.checkPermission('astrum:*');
  } catch {
    return false;
  }
}

/**
 * ApiPage — API hub (master–detail): permission map + Swagger Try It per operation.
 */
export function ApiPage({ className }: ApiPageProps) {
  const isCloud = ExtJS.state?.().getValue?.('isCloud', false) ?? false;
  const deepLink = useApiModuleDeepLink();
  const currentUserId = readCurrentSessionUserId();
  const { data: permData, loading: permLoading, error: permError } = useEndpointPermissions();
  const { fetchRoles, findRole } = useRolesApi();
  const { fetchPrivileges } = usePrivilegesApi();

  const [dismissPermBanner, setDismissPermBanner] = useState(false);
  const [dismissSwaggerBanner, setDismissSwaggerBanner] = useState(false);
  const [dismissDeepLinkBanner, setDismissDeepLinkBanner] = useState(false);
  const [leftCollapsed, setLeftCollapsed] = useState(false);
  const [selected, setSelected] = useState<MergedApiEndpoint | null>(null);

  const [roleLens, setRoleLens] = useState<{
    allRoles: Role[];
    privileges: Privilege[];
    target: Role;
  } | null>(null);
  const [roleLensError, setRoleLensError] = useState<string | null>(null);

  const swaggerUrl = useMemo(() => ExtJS.urlOf('/service/rest/swagger.json'), []);
  const { swagger: swaggerJson, hasError: swaggerError } = useApiPage({ swaggerUrl });

  const swaggerSpec = useMemo(() => {
    if (!swaggerJson) {
      return null;
    }
    return isCloud ? filterSwaggerSpecForCloud(swaggerJson) : swaggerJson;
  }, [swaggerJson, isCloud]);

  useEffect(() => {
    if (!deepLink.roleLensId) {
      setRoleLens(null);
      setRoleLensError(null);
      return undefined;
    }
    let cancelled = false;
    setRoleLens(null);
    setRoleLensError(null);
    (async () => {
      try {
        const [allRoles, privRes, target] = await Promise.all([
          fetchRoles(),
          fetchPrivileges(undefined, undefined, undefined, 0, undefined),
          findRole(deepLink.roleLensId!),
        ]);
        if (cancelled) {
          return;
        }
        if (!target) {
          setRoleLensError(ApiStrings.PAGE.ROLE_WAS_NOT_FOUND(deepLink.roleLensId));
          return;
        }
        setRoleLens({
          allRoles,
          privileges: privRes.data,
          target,
        });
      } catch (e: unknown) {
        if (!cancelled) {
          setRoleLensError(e instanceof Error ? e.message : ApiStrings.PAGE.FAILED_LOAD_ROLE_LENS_DATA);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [deepLink.roleLensId, fetchRoles, findRole, fetchPrivileges]);

  const merged = useMemo(
    () => mergeSwaggerAndPermissions(swaggerSpec, permData?.endpoints ?? []),
    [swaggerSpec, permData?.endpoints]
  );

  const permissionFiltered = useMemo(
    () => filterEndpointsByPermissionSubstring(merged, deepLink.permissionFilter),
    [merged, deepLink.permissionFilter]
  );

  const viewAs = useViewAsUserAccess(
    permissionFiltered,
    deepLink.viewAsUserId,
    currentUserId,
    true
  );

  const roleLensAccessById = useMemo(() => {
    if (!deepLink.roleLensId || !roleLens) {
      return null;
    }
    return computeRoleLensAccessById(
      permissionFiltered,
      deepLink.roleLensId,
      roleLens.allRoles,
      roleLens.privileges
    );
  }, [deepLink.roleLensId, roleLens, permissionFiltered]);

  const accessById = useMemo(() => {
    const permMap = readAstrumPermissionMap();
    const m: Record<string, ReturnType<typeof computeEndpointAccess>> = {};
    for (const row of permissionFiltered) {
      const id = endpointRowId(row);
      if (viewAs.accessById && viewAs.accessById[id] !== undefined) {
        m[id] = viewAs.accessById[id]!;
      } else if (roleLensAccessById) {
        m[id] = roleLensAccessById[id] ?? 'unknown';
      } else {
        m[id] = computeEndpointAccess(row.permission, permMap);
      }
    }
    return m;
  }, [permissionFiltered, viewAs.accessById, roleLensAccessById]);

  const deepLinkWarnings = useMemo(() => {
    const w = [...deepLink.warnings];
    if (roleLensError) {
      w.push(roleLensError);
    }
    if (viewAs.error) {
      w.push(viewAs.error);
    }
    if (
      deepLink.viewAsUserId &&
      currentUserId &&
      deepLink.viewAsUserId !== currentUserId &&
      !isAdminUser()
    ) {
      w.push(
        ApiStrings.PAGE.CHECKING_ANOTHER_USERS_ACCESS_REQUIRES
      );
    }
    if (deepLink.endpointParam && permissionFiltered.length > 0) {
      const f = findEndpointByDeepLink(permissionFiltered, deepLink.endpointParam);
      if (!f) {
        w.push(
          ApiStrings.PAGE.NO_ENDPOINT_MATCHED_ENDPOINT_DEEP
        );
      }
    }
    return w;
  }, [deepLink, roleLensError, viewAs.error, currentUserId, permissionFiltered]);

  const _deepLinkWarningKey = deepLinkWarnings.join('\n');
  useEffect(() => {
    setDismissDeepLinkBanner(false);
  }, []);

  useEffect(() => {
    if (!deepLink.endpointParam || permissionFiltered.length === 0) {
      return;
    }
    const f = findEndpointByDeepLink(permissionFiltered, deepLink.endpointParam);
    if (f) {
      setSelected(f);
    }
  }, [deepLink.endpointParam, permissionFiltered]);

  const selectedAccess = selected ? accessById[endpointRowId(selected)] ?? 'unknown' : 'unknown';

  const handleSelect = useCallback((row: MergedApiEndpoint) => {
    setSelected(row);
  }, []);

  const initialLoading = permLoading && !permData;
  const roleLensLoading = !!deepLink.roleLensId && !roleLens && !roleLensError;
  const listLoading = initialLoading || viewAs.loading || roleLensLoading;

  const accessDotPalette = deepLink.roleLensId ? 'roleLens' : 'session';

  // Navigation helper for Settings breadcrumb
  const navigateToSettings = () => {
    window.location.hash = '#admin/settings';
  };

  return (
    <Box
      className={`api-page ${className || ''}`.trim()}
      data-testid="api-page"
      data-loading={initialLoading ? 'true' : 'false'}
    >
      <PageHeader
        title={ApiStrings.PAGE.API}
        description={ApiStrings.PAGE.DOCUMENTATION_PERMISSIONS_ACCESS_TOOLS}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: navigateToSettings },
          { label: 'API' },
        ]}
        className="api-page__header"
      />

      {/* Deep link info */}
      {(deepLink.viewAsUserId || deepLink.roleLensId || deepLink.permissionFilter) && (
        <Box className="api-page__deep-link-info" mt="2">
          {deepLink.viewAsUserId && (
            <Text size="2" color="gray" as="div">
              {ApiStrings.PAGE.VIEWING_ACCESS_FOR_USER(<Text weight="bold">{deepLink.viewAsUserId}</Text>)}
              {currentUserId && deepLink.viewAsUserId === currentUserId ? ` ${ApiStrings.PAGE.YOU}` : null}
            </Text>
          )}
          {deepLink.roleLensId && (
            <Text size="2" color="gray" as="div">
              {ApiStrings.PAGE.ROLE_LENS(
                <Text weight="bold">
                  {roleLens?.target?.name ?? deepLink.roleLensId}
                </Text>
              )}
            </Text>
          )}
          {deepLink.permissionFilter && (
            <Text size="2" color="gray" as="div">
              {ApiStrings.PAGE.FILTERED_TO_PERMISSION(<Text weight="bold">{deepLink.permissionFilter}</Text>)}
            </Text>
          )}
        </Box>
      )}

      {deepLinkWarnings.length > 0 && !dismissDeepLinkBanner && (
        <Box className="api-page__alerts" mb="3">
          <SettingsAlert type="warning" onClose={() => setDismissDeepLinkBanner(true)}>
            <Flex direction="column" gap="1">
              {deepLinkWarnings.map((msg, i) => (
                <Text key={`${i}-${msg}`} size="2">
                  {msg}
                </Text>
              ))}
            </Flex>
          </SettingsAlert>
        </Box>
      )}

      {permError && !dismissPermBanner && (
        <Box className="api-page__alerts" mb="3">
          <SettingsAlert type="warning" onClose={() => setDismissPermBanner(true)}>
            <Text size="2" as="div" weight="medium">
              {ApiStrings.PAGE.COULD_NOT_LOAD_API_PERMISSION}
            </Text>
            <Text size="2" as="div" mt="2" style={{ opacity: 0.95 }}>
              {ApiStrings.PAGE.PERMISSION_MAP_HINT(<Text style={{ fontFamily: 'var(--font-mono)' }}>astrum:settings:read</Text>)}
            </Text>
            <Text size="2" as="div" mt="2" style={{ fontFamily: 'var(--font-mono)', wordBreak: 'break-word' }}>
              {permError}
            </Text>
          </SettingsAlert>
        </Box>
      )}

      {swaggerError && !dismissSwaggerBanner && (
        <Box className="api-page__alerts" mb="3">
          <SettingsAlert type="warning" onClose={() => setDismissSwaggerBanner(true)}>
            {ApiStrings.PAGE.API_DOCUMENTATION_FAILED_LOAD_PERMISSION}
          </SettingsAlert>
        </Box>
      )}

      <ApiLayout
        leftCollapsed={leftCollapsed}
        onToggleLeft={() => setLeftCollapsed((c) => !c)}
        leftPanel={
          <EndpointList
            endpoints={permissionFiltered}
            accessById={accessById}
            selectedId={selected ? endpointRowId(selected) : null}
            onSelect={handleSelect}
            loading={listLoading}
            accessDotPalette={accessDotPalette}
          />
        }
        rightPanel={
          <EndpointDetail row={selected} fullSwagger={swaggerSpec} access={selectedAccess} />
        }
      />

      <Box className="api-page__help">
        <Flex align="center" gap="2" className="api-page__help-header">
          <Info size={16} />
          <Text size="2" weight="medium">
            {ApiStrings.PAGE.ABOUT_API_DOCUMENTATION}
          </Text>
        </Flex>
        <Text as="p" size="2" className="api-page__help-text">
          {ApiStrings.PAGE.EXPLORE_ASTRUM_REPOSITORY_REST_API}
        </Text>
        <Text as="p" size="2" className="api-page__help-text">
          {ApiStrings.PAGE.SEE_DOCUMENTATION(
            <a
              href="https://help.globalart.dev/en/rest-and-integration-api.html"
              target="_blank"
              rel="noopener noreferrer"
              className="api-page__help-link"
            >
              {ApiStrings.PAGE.DOCUMENTATION_LINK}
              <ExternalLink size={12} />
            </a>
          )}
        </Text>
      </Box>
    </Box>
  );
}

export default ApiPage;
