
import {ExtJS} from '@globalart/astrum-ui-plugin';

export const isIqServerEnabled = () => {
  return ExtJS.state().getValue('clm', {}).enabled;
};

/**
 * Check if Firewall is enabled via IQ Server.
 * When Firewall is enabled, Health Check column should not be shown
 * because Firewall provides vulnerability data instead.
 *
 * This mirrors the ExtJS condition in NX.util.condition.HasNoFirewall
 */
export const hasFirewall = () => {
  const clmState = ExtJS.state().getValue('clm', {hasFirewall: false});
  return clmState.hasFirewall === true;
};

export const canReadHealthCheck = () => {
  return ExtJS.checkPermission('astrum:healthcheck:read');
};

export const canUpdateHealthCheck = () => {
  return ExtJS.checkPermission('astrum:healthcheck:update');
};

export const canReadFirewallStatus = () => {
  return ExtJS.checkPermission('astrum:iq-violation-summary:read');
};

export const canReadHealthCheckSummary = () => {
  return ExtJS.checkPermission('astrum:healthchecksummary:read');
};

export const canReadHealthCheckDetail = () => {
  return ExtJS.checkPermission('astrum:healthcheckdetail:read');
};
