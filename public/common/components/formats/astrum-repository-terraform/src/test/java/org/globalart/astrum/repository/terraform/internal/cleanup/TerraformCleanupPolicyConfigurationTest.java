package org.globalart.astrum.repository.terraform.internal.cleanup;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_KEY;

public class TerraformCleanupPolicyConfigurationTest
{
  @Test
  public void supportsAgeDownloadPathAndRetainCriteria() {
    assertThat(new TerraformCleanupPolicyConfiguration().getConfiguration())
        .containsEntry(LAST_DOWNLOADED_KEY, true)
        .containsEntry(RETAIN_KEY, true)
        .containsEntry(IS_PRERELEASE_KEY, false);
  }
}
