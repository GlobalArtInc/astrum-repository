
package org.globalart.astrum.repository.apt.internal.debian;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

/**
 * @since 3.17
 */
public class DebianVersionTest
{
  private final static String LOWER_EPOCH_UPSTREAM_DEBIAN = "2:7.3.429-2ubuntu2.1";

  private final static String HIGHER_EPOCH_UPSTREAM_DEBIAN = "3:7.3.429-2ubuntu2.1";

  private final static String UPSTREAM = "0.13";

  private final static String UPSTREAM_DEBIAN = "1.11-1";

  private final static String UPSTREAM_DEBIAN_TILDE = "30~pre9-5ubuntu2";

  @Test
  public void testCompareVersionIsSimilar() {
    assertThat(new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN).compareTo(new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN)),
        is(0));
    assertThat(new DebianVersion(UPSTREAM_DEBIAN).compareTo(new DebianVersion(UPSTREAM_DEBIAN)), is(0));
    assertThat(new DebianVersion(UPSTREAM_DEBIAN_TILDE).compareTo(new DebianVersion(UPSTREAM_DEBIAN_TILDE)), is(0));
    assertThat(new DebianVersion(UPSTREAM).compareTo(new DebianVersion(UPSTREAM)), is(0));
  }

  @Test
  public void testEqualityVersionIsSimilar() {
    assertThat(new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN),
        is(equalTo(new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN))));
    assertThat(new DebianVersion(UPSTREAM_DEBIAN), is(equalTo(new DebianVersion(UPSTREAM_DEBIAN))));
    assertThat(new DebianVersion(UPSTREAM_DEBIAN_TILDE), is(equalTo(new DebianVersion(UPSTREAM_DEBIAN_TILDE))));
    assertThat(new DebianVersion(UPSTREAM), is(equalTo(new DebianVersion(UPSTREAM))));
  }

  @Test
  public void testCompareLowerEpochVersion() {
    assertThat(
        new DebianVersion(HIGHER_EPOCH_UPSTREAM_DEBIAN).compareTo(new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN)),
        is(1));
  }

  @Test
  public void testCompareHigherEpochVersion() {
    assertThat(
        new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN).compareTo(new DebianVersion(HIGHER_EPOCH_UPSTREAM_DEBIAN)),
        is(-1));
  }

  @Test
  public void testVerifyEpochVersionParthIsCorrect_IfAllPartsPresent() {
    assertThat(new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN).getEpoch(), is(equalTo(2)));
  }

  @Test
  public void testVerifyEpochVersionPartIsZero_IfEpochNotExist() {
    assertThat(new DebianVersion(UPSTREAM).getEpoch(), is(equalTo(0)));
  }

  @Test
  public void testVerifyUpstreamVersionPartIsCorrect_IfAllPartsPresent() {
    assertThat(new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN).getUpstreamVersion(), is(equalTo("7.3.429")));
  }

  @Test
  public void testVerifyUpstreamVersionPartIsCorrect_IfThereAreNoEpochAndDebian() {
    assertThat(new DebianVersion(UPSTREAM).getUpstreamVersion(), is(equalTo("0.13")));
  }

  @Test
  public void testVerifyDebianVersionPartIsCorrect_IfAllPartsPresent() {
    assertThat(new DebianVersion(LOWER_EPOCH_UPSTREAM_DEBIAN).getDebianRevision(), is(equalTo("2ubuntu2.1")));
  }
}
