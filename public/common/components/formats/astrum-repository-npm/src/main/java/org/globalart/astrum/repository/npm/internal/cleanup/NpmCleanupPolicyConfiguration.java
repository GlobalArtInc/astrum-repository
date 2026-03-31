
package org.globalart.astrum.repository.npm.internal.cleanup;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.util.Map;


import org.globalart.astrum.cleanup.config.CleanupPolicyConfiguration;
import org.globalart.astrum.repository.npm.internal.NpmFormat;

import com.google.common.collect.ImmutableMap;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;

/**
 * Defines which cleanup policy fields to display for npm.
 *
 * @since 3.14
 */
@Component
@Qualifier(NpmFormat.NAME)
public class NpmCleanupPolicyConfiguration
    implements CleanupPolicyConfiguration
{
  @Override
  public Map<String, Boolean> getConfiguration() {
    return ImmutableMap.of(LAST_BLOB_UPDATED_KEY, true,
        LAST_DOWNLOADED_KEY, true,
        IS_PRERELEASE_KEY, false,
        REGEX_KEY, true);
  }
}
