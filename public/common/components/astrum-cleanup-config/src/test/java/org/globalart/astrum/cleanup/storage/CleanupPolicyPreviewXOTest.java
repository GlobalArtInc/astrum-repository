
package org.globalart.astrum.cleanup.storage;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;

public class CleanupPolicyPreviewXOTest
{
  private final CleanupPolicyPreviewXO underTest = new CleanupPolicyPreviewXO();

  @Test
  public void testDefaultsAreNull() {
    assertThat(underTest.getRepositoryName(), is(nullValue()));
    assertThat(underTest.getCriteria(), is(nullValue()));
  }

  @Test
  public void testGetSetRepositoryName() {
    underTest.setRepositoryName("my-repo");
    assertThat(underTest.getRepositoryName(), is("my-repo"));
  }

  @Test
  public void testGetSetCriteria() {
    CleanupPolicyCriteria criteria = newCriteria();
    underTest.setCriteria(criteria);
    assertThat(underTest.getCriteria(), is(sameInstance(criteria)));
  }

  @Test
  public void testToString() {
    CleanupPolicyCriteria criteria = newCriteria();
    underTest.setRepositoryName("my-repo");
    underTest.setCriteria(criteria);

    String result = underTest.toString();

    assertThat(result, is("CleanupPolicyPreviewXO(repositoryName:my-repo, criteria:" + criteria + ")"));
    assertThat(result, containsString("CleanupPolicyPreviewXO("));
    assertThat(result, containsString("repositoryName:my-repo"));
    assertThat(result, containsString(", criteria:" + criteria));
  }

  @Test
  public void testToStringWithNullFields() {
    assertThat(underTest.toString(), is("CleanupPolicyPreviewXO(repositoryName:null, criteria:null)"));
  }

  private static CleanupPolicyCriteria newCriteria() {
    return new CleanupPolicyCriteria(10, 20, CleanupPolicyReleaseType.RELEASES, "*.tmp", 5, "version");
  }
}
