

import React from 'react';
import { Badge, Box, Flex, Tabs, Text } from '@radix-ui/themes';

import type { EndpointAccessDot } from './utils/endpointAccess';
import type { MergedApiEndpoint } from './utils/mergeSwaggerPermissions';
import { GrantAccessTab } from './tabs/GrantAccessTab';
import { TryItTab } from './tabs/TryItTab';
import { WhoHasAccessTab } from './tabs/WhoHasAccessTab';
import { useEndpointDetail } from './useEndpointDetail';
import ApiStrings from '../../../../constants/pages/settings/system/ApiStrings';

export interface EndpointDetailProps {
  row: MergedApiEndpoint | null;
  fullSwagger: Record<string, unknown> | null;
  access: EndpointAccessDot;
}

function formatPermissions(row: MergedApiEndpoint): string {
  const reqs = row.permission?.permissions ?? [];
  if (reqs.length === 0) {
    return row.permission?.authenticated ? ApiStrings.ENDPOINT_DETAIL.NONE_MAPPED_SEE_DOCUMENTATION : ApiStrings.ENDPOINT_DETAIL.NONE_MAY_ALLOW_ANONYMOUS;
  }
  const logical = reqs[0]?.logical === 'OR' ? 'OR' : 'AND';
  const parts = reqs.map((r) => r.permission);
  return logical === 'OR' ? parts.join(' OR ') : parts.join(' AND ');
}

export function EndpointDetail({ row, fullSwagger, access }: EndpointDetailProps) {
  const { tab, hasSecurityDirectoryRead, hasGrantAccess, selectTab } = useEndpointDetail();

  if (!row) {
    return (
      <Box className="api-endpoint-detail api-endpoint-detail--empty" p="4" aria-live="polite">
        <Text size="3" weight="medium" mb="2" as="div">
          {ApiStrings.ENDPOINT_DETAIL.CHOOSE_ENDPOINT}
        </Text>
        <Text size="2" color="gray" mb="3" as="div">
          {ApiStrings.ENDPOINT_DETAIL.CLICK_ANY_OPERATION_LIST_LEFT}
        </Text>
        <Box as="ul" style={{ margin: 0, paddingLeft: '1.25rem' }} mb="3">
          <li>
            <Text size="2">
              {ApiStrings.ENDPOINT_DETAIL.TRY_IT_HINT(
                <Text weight="bold" as="span">
                  {ApiStrings.ENDPOINT_DETAIL.TRY_IT}
                </Text>
              )}
            </Text>
          </li>
          <li>
            <Text size="2">
              {ApiStrings.ENDPOINT_DETAIL.WHO_HAS_ACCESS_HINT(
                <Text weight="bold" as="span">
                  {ApiStrings.ENDPOINT_DETAIL.WHO_HAS_ACCESS}
                </Text>
              )}
            </Text>
          </li>
          <li>
            <Text size="2">
              {ApiStrings.ENDPOINT_DETAIL.GRANT_ACCESS_HINT(
                <Text weight="bold" as="span">
                  {ApiStrings.ENDPOINT_DETAIL.GRANT_ACCESS}
                </Text>
              )}
            </Text>
          </li>
        </Box>
        <Text size="2" color="gray">
          {ApiStrings.ENDPOINT_DETAIL.ACCESS_DOTS_LIST_REFLECT_PERMISSIONS}
        </Text>
      </Box>
    );
  }

  const accessDenied = access === 'denied' || access === 'partial';

  return (
    <Box className="api-endpoint-detail" aria-live="polite" data-testid="api-endpoint-detail">
      <Tabs.Root value={tab} onValueChange={selectTab}>
        <Box className="api-endpoint-detail__detail-bar">
          <Box className="api-endpoint-detail__header" mb="2">
            <Flex align="center" gap="2" wrap="wrap" mb="2">
              <Badge size="2" color="blue">
                {row.httpMethod}
              </Badge>
              <Text size="3" weight="medium" className="api-endpoint-detail__path">
                {row.fullPath}
              </Text>
            </Flex>
            <Text size="2" color="gray" mb="1">
              {ApiStrings.ENDPOINT_DETAIL.REQUIRED(formatPermissions(row))}
            </Text>
            {row.summary && (
              <Text size="2" mt="2">
                {row.summary}
              </Text>
            )}
          </Box>
          <Tabs.List className="api-endpoint-detail__tabs-list">
            <Tabs.Trigger value="try">{ApiStrings.ENDPOINT_DETAIL.TRY_IT}</Tabs.Trigger>
            {hasSecurityDirectoryRead && <Tabs.Trigger value="who">{ApiStrings.ENDPOINT_DETAIL.WHO_HAS_ACCESS}</Tabs.Trigger>}
            {hasGrantAccess && <Tabs.Trigger value="grant">{ApiStrings.ENDPOINT_DETAIL.GRANT_ACCESS}</Tabs.Trigger>}
          </Tabs.List>
        </Box>
        <Tabs.Content value="try">
          <Box pt="3">
            <TryItTab fullSwagger={fullSwagger} row={row} accessDenied={accessDenied} />
          </Box>
        </Tabs.Content>
        {hasSecurityDirectoryRead && (
          <Tabs.Content value="who">
            <Box pt="3">
              <WhoHasAccessTab row={row} active={tab === 'who'} />
            </Box>
          </Tabs.Content>
        )}
        {hasGrantAccess && (
          <Tabs.Content value="grant">
            <Box pt="3">
              <GrantAccessTab row={row} active={tab === 'grant'} />
            </Box>
          </Tabs.Content>
        )}
      </Tabs.Root>
    </Box>
  );
}
