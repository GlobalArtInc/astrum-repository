
package org.globalart.astrum.repository.maven.internal.cleanup;

import java.util.Map;

import org.globalart.astrum.cleanup.config.CleanupPolicyConfiguration;
import org.globalart.astrum.repository.maven.internal.Maven2Format;

import com.google.common.collect.ImmutableMap;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_SORT_BY_KEY;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Defines which cleanup policy fields to display for maven.
 *
 * @since 3.14
 */
@Component
@Qualifier(Maven2Format.NAME)
public class Maven2CleanupPolicyConfiguration
    implements CleanupPolicyConfiguration
{
  @Override
  public Map<String, Boolean> getConfiguration() {
    return ImmutableMap.<String, Boolean>builder()
        .put(LAST_BLOB_UPDATED_KEY, true)
        .put(LAST_DOWNLOADED_KEY, true)
        .put(IS_PRERELEASE_KEY, true)
        .put(REGEX_KEY, true)
        .put(RETAIN_KEY, false)
        .put(RETAIN_SORT_BY_KEY, false)
        .build();
  }
}
