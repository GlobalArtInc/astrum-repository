package org.globalart.astrum.repository.alpine.internal.cleanup;

import java.util.Map;

import org.globalart.astrum.cleanup.config.CleanupPolicyConfiguration;
import org.globalart.astrum.repository.alpine.AlpineFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;

@Component
@Qualifier(AlpineFormat.NAME)
public class AlpineCleanupPolicyConfiguration
    implements CleanupPolicyConfiguration
{
  @Override
  public Map<String, Boolean> getConfiguration() {
    return Map.of(
        LAST_BLOB_UPDATED_KEY, true,
        LAST_DOWNLOADED_KEY, true,
        IS_PRERELEASE_KEY, false,
        REGEX_KEY, true);
  }
}
