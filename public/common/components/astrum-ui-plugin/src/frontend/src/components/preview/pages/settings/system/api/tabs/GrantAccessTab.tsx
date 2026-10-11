

import React from 'react';
import { Box, Text } from '@radix-ui/themes';

import { GrantWizard } from '../grant/GrantWizard';
import { canGrantAccess } from '../utils/endpointPermissions';
import type { MergedApiEndpoint } from '../utils/mergeSwaggerPermissions';
import ApiStrings from '../../../../../constants/pages/settings/system/ApiStrings';

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
          {ApiStrings.GRANT_ACCESS_TAB.YOU_NEED_ADDITIONAL_PERMISSIONS_GRANT}
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
