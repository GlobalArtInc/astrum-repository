
package org.globalart.astrum.repository.pypi.internal.cleanup;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.util.Map;


import org.globalart.astrum.cleanup.config.CleanupPolicyConfiguration;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;

import com.google.common.collect.ImmutableMap;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;

/**
 * @since 3.19
 */
@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiCleanupPolicyConfiguration
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