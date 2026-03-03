
package org.globalart.astrum.repository.search.normalize;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class PEP440VersionNormalizerTest
{
  private final PEP440VersionNormalizer underTest = new PEP440VersionNormalizer();

  @Test
  public void testBlankInputs() {
    assertThat(underTest.getNormalizedVersion(null)).isEqualTo("");
    assertThat(underTest.getNormalizedVersion("")).isEqualTo("");
    assertThat(underTest.getNormalizedVersion("  ")).isEqualTo("");
  }

  @Test
  public void testReleaseVersion() {
    String result = underTest.getNormalizedVersion("1.2.3");
    assertThat(result).startsWith("1000000000."); // default epoch
    assertThat(result).endsWith(".0c");
  }

  @Test
  public void testEpoch() {
    String withEpoch = underTest.getNormalizedVersion("2!1.2.3");
    String withoutEpoch = underTest.getNormalizedVersion("1.2.3");
    assertThat(withEpoch).startsWith("1000000002.");
    assertThat(withoutEpoch).isLessThan(withEpoch);
  }

  @Test
  public void testLocalVersionStripped() {
    String withLocal = underTest.getNormalizedVersion("1.2.3+local.abc");
    String withoutLocal = underTest.getNormalizedVersion("1.2.3");
    assertThat(withLocal).isEqualTo(withoutLocal);
  }

  @Test
  public void testAlphaPreRelease() {
    String alpha = underTest.getNormalizedVersion("1.0a1");
    String release = underTest.getNormalizedVersion("1.0");
    assertThat(alpha).isLessThan(release);
  }

  @Test
  public void testBetaPreRelease() {
    String beta = underTest.getNormalizedVersion("1.0b1");
    String release = underTest.getNormalizedVersion("1.0");
    assertThat(beta).isLessThan(release);
  }

  @Test
  public void testRcPreRelease() {
    String rc = underTest.getNormalizedVersion("1.0rc1");
    String release = underTest.getNormalizedVersion("1.0");
    assertThat(rc).isLessThan(release);
  }

  @Test
  public void testDevRelease() {
    String dev = underTest.getNormalizedVersion("1.0.dev1");
    String release = underTest.getNormalizedVersion("1.0");
    assertThat(dev).isLessThan(release);
  }

  @Test
  public void testPostRelease() {
    String post = underTest.getNormalizedVersion("1.0.post1");
    String release = underTest.getNormalizedVersion("1.0");
    assertThat(release).isLessThan(post);
  }

  @Test
  public void testPreReleaseOrdering() {
    String dev = underTest.getNormalizedVersion("1.0.dev1");
    String alpha = underTest.getNormalizedVersion("1.0a1");
    String beta = underTest.getNormalizedVersion("1.0b1");
    String rc = underTest.getNormalizedVersion("1.0rc1");
    assertThat(dev).isLessThan(alpha);
    assertThat(alpha).isLessThan(beta);
    assertThat(beta).isLessThan(rc);
  }

  @Test
  public void testCaseInsensitiveQualifiers() {
    assertThat(underTest.getNormalizedVersion("1.0A1"))
        .isEqualTo(underTest.getNormalizedVersion("1.0a1"));
  }

  @Test
  public void testHigherVersionSortsAfter() {
    assertThat(underTest.getNormalizedVersion("1.0"))
        .isLessThan(underTest.getNormalizedVersion("2.0"));
  }
}
