

import { ExtJS } from '../../../../../../../interface/ExtJS';

// Wrap calls in ExtJS.usePermission at the call site so the check is reactive to async permission loads.
// The Who Has Access and Grant Access tabs both load the full security directory (roles + privileges +
// users), so any missing read causes a 403 and a red error banner in the tab body. Gate on all three.

export function canReadSecurityDirectory(): boolean {
  return (
    ExtJS.checkPermission('astrum:roles:read') &&
    ExtJS.checkPermission('astrum:users:read') &&
    ExtJS.checkPermission('astrum:privileges:read')
  );
}

export function canGrantAccess(): boolean {
  return (
    canReadSecurityDirectory() &&
    (ExtJS.checkPermission('astrum:roles:update') || ExtJS.checkPermission('astrum:roles:create')) &&
    ExtJS.checkPermission('astrum:users:update')
  );
}
