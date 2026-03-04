
package org.globalart.astrum.cleanup.config;

import java.util.Map;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_SORT_BY_KEY;

/**
 * Defines which default cleanup policy fields to display.
 *
 * @since 3.14
 */
@Primary
@Component
@Qualifier(DefaultCleanupPolicyConfiguration.NAME)
public class DefaultCleanupPolicyConfiguration
    implements CleanupPolicyConfiguration
{
  public static final String NAME = "DefaultCleanupPolicyConfiguration";

  @Override
  public Map<String, Boolean> getConfiguration() {
    return ImmutableMap.<String, Boolean>builder()
        .put(LAST_BLOB_UPDATED_KEY, true)
        .put(LAST_DOWNLOADED_KEY, true)
        .put(IS_PRERELEASE_KEY, false)
        .put(REGEX_KEY, false)
        .put(RETAIN_KEY, false)
        .put(RETAIN_SORT_BY_KEY, false)
        .build();
  }
}
