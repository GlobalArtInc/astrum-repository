
package org.globalart.astrum.cleanup.config;

import java.util.Map;

/**
 * Controls which cleanup criteria are allowed per format
 *
 * @since 3.14
 */
public interface CleanupPolicyConfiguration
{
  Map<String, Boolean> getConfiguration();
}
