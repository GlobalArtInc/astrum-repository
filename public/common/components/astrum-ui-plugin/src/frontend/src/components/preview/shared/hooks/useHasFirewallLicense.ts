

import { ExtJS } from '../../../../interface/ExtJS';

/** Returns true when the instance has an active Firewall (CLM/IQ) license. Session-scoped. */
export function useHasFirewallLicense(): boolean {
  const clm = ExtJS.useState?.(() => ExtJS.state()?.getValue?.('clm'));
  return Boolean((clm?.enabled ?? clm?.hasFirewall));
}
