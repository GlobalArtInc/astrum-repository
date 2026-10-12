package org.globalart.astrum.repository.terraform.internal.cleanup;

import java.util.Map;

import org.globalart.astrum.cleanup.config.CleanupPolicyConfiguration;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_SORT_BY_KEY;

/**
 * Module and provider versions are semantic versions, so cleanup can keep the newest N of each module or provider
 * besides the age, download and path criteria.
 */
@Component
@Qualifier(TerraformFormat.NAME)
public class TerraformCleanupPolicyConfiguration
    implements CleanupPolicyConfiguration
{
  @Override
  public Map<String, Boolean> getConfiguration() {
    return ImmutableMap.<String, Boolean>builder()
        .put(LAST_BLOB_UPDATED_KEY, true)
        .put(LAST_DOWNLOADED_KEY, true)
        .put(IS_PRERELEASE_KEY, false)
        .put(REGEX_KEY, true)
        .put(RETAIN_KEY, true)
        .put(RETAIN_SORT_BY_KEY, true)
        .build();
  }
}
