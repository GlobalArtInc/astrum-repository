

import React from 'react';
import { Box, Text } from '@radix-ui/themes';

import { GrantWizard } from '../grant/GrantWizard';
import { canGrantAccess } from '../utils/endpointPermissions';
import type { MergedApiEndpoint } from '../utils/mergeSwaggerPermissions';

export interface GrantAccessTabProps {
  row: MergedApiEndpoint;
  /** When false, skip loading grant wizard data */
  active: boolean;
}

export function GrantAccessTab({ row, active }: GrantAccessTabProps) {
  if (!canGrantAccess()) {
    return (
      <Box data-testid="api-grant-access-tab">
        <Text size="2" color="gray">
          You need additional permissions to grant API access (roles read/update/create and users update). Contact an
          administrator.
        </Text>
      </Box>
    );
  }

  return (
    <Box data-testid="api-grant-access-tab">
      <GrantWizard key={`${row.httpMethod}-${row.fullPath}`} row={row} active={active} />
    </Box>
  );
}
