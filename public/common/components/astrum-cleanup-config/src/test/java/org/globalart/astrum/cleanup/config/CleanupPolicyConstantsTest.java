
package org.globalart.astrum.cleanup.config;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.hasItem;

/**
 * Tests for {@link CleanupPolicyConstants}.
 */
public class CleanupPolicyConstantsTest
{
  @Test
  public void repositoriesFieldSupportedFormatsContainsExactlyTheExpectedFormats() {
    assertThat(CleanupPolicyConstants.REPOSITORIES_FIELD_SUPPORTED_FORMATS, hasSize(11));
    assertThat(CleanupPolicyConstants.REPOSITORIES_FIELD_SUPPORTED_FORMATS, containsInAnyOrder(
        "npm", "pypi", "go", "helm", "nuget",
        "yum", "rubygems", "terraform", "swift", "apt", "pub"));
  }

  @Test
  public void repositoriesFieldSupportedFormatsDoesNotContainMaven2() {
    assertThat(CleanupPolicyConstants.REPOSITORIES_FIELD_SUPPORTED_FORMATS, not(hasItem("maven2")));
  }

  @Test
  public void stringConstantsEqualTheirLiteralValues() {
    assertThat(CleanupPolicyConstants.CLEANUP_ATTRIBUTES_KEY, is("cleanup"));
    assertThat(CleanupPolicyConstants.CLEANUP_NAME_KEY, is("policyName"));
    assertThat(CleanupPolicyConstants.IS_PRERELEASE_KEY, is("isPrerelease"));
    assertThat(CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY, is("lastBlobUpdated"));
    assertThat(CleanupPolicyConstants.LAST_DOWNLOADED_KEY, is("lastDownloaded"));
    assertThat(CleanupPolicyConstants.RETAIN_KEY, is("retain"));
    assertThat(CleanupPolicyConstants.RETAIN_SORT_BY_KEY, is("sortBy"));
    assertThat(CleanupPolicyConstants.REGEX_KEY, is("regex"));
    assertThat(CleanupPolicyConstants.MAVEN2_FORMAT, is("maven2"));
    assertThat(CleanupPolicyConstants.DOCKER_FORMAT, is("docker"));
  }
}
