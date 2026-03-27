
package org.globalart.astrum.repository.maven.internal.cleanup;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.IS_PRERELEASE_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.REGEX_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_KEY;
import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.RETAIN_SORT_BY_KEY;

public class Maven2CleanupPolicyConfigurationTest
{
  private Maven2CleanupPolicyConfiguration underTest;

  @Before
  public void setUp() {
    underTest = new Maven2CleanupPolicyConfiguration();
  }

  @Test
  public void getConfiguration_returnsNonNullMap() {
    assertThat(underTest.getConfiguration(), is(notNullValue()));
  }

  @Test
  public void getConfiguration_hasSixEntries() {
    assertThat(underTest.getConfiguration().size(), is(6));
  }

  @Test
  public void getConfiguration_containsLastBlobUpdatedEnabled() {
    assertThat(underTest.getConfiguration(), hasEntry(LAST_BLOB_UPDATED_KEY, true));
  }

  @Test
  public void getConfiguration_containsLastDownloadedEnabled() {
    assertThat(underTest.getConfiguration(), hasEntry(LAST_DOWNLOADED_KEY, true));
  }

  @Test
  public void getConfiguration_containsIsPrereleaseEnabled() {
    assertThat(underTest.getConfiguration(), hasEntry(IS_PRERELEASE_KEY, true));
  }

  @Test
  public void getConfiguration_containsRegexEnabled() {
    assertThat(underTest.getConfiguration(), hasEntry(REGEX_KEY, true));
  }

  @Test
  public void getConfiguration_retainDisabledOnCommunityEdition() {
    assertThat(underTest.getConfiguration(), hasEntry(RETAIN_KEY, false));
  }

  @Test
  public void getConfiguration_retainSortByDisabledOnCommunityEdition() {
    assertThat(underTest.getConfiguration(), hasEntry(RETAIN_SORT_BY_KEY, false));
  }
}
