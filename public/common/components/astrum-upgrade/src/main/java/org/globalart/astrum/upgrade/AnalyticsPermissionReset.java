
package org.globalart.astrum.upgrade;

/**
 * Used during the upgrade process to allow OSS analytics permission to be re-requested.
 *
 * @since 3.35
 */
public interface AnalyticsPermissionReset
{
  void resetAnalyticsPermissionIfDisabled();
}
