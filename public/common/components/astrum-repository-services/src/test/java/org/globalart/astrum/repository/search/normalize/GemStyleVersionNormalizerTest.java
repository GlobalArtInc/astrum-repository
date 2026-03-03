
package org.globalart.astrum.repository.search.normalize;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class GemStyleVersionNormalizerTest
{
  private final GemStyleVersionNormalizer underTest = new GemStyleVersionNormalizer();

  @Test
  public void testBlankInputs() {
    assertThat(underTest.getNormalizedVersion(null)).isEqualTo("");
    assertThat(underTest.getNormalizedVersion("")).isEqualTo("");
    assertThat(underTest.getNormalizedVersion("   ")).isEqualTo("");
  }

  @Test
  public void testSimpleNumericRelease() {
    String result = underTest.getNormalizedVersion("1.2.3");
    assertThat(result).endsWith(".0c");
  }

  @Test
  public void testPreReleaseWithLetters() {
    String preRelease = underTest.getNormalizedVersion("1.0.0.pre");
    String release = underTest.getNormalizedVersion("1.0.0");
    assertThat(preRelease).contains(".0a.");
    assertThat(preRelease).isLessThan(release);
  }

  @Test
  public void testHigherVersionSortsAfter() {
    assertThat(underTest.getNormalizedVersion("1.0.0"))
        .isLessThan(underTest.getNormalizedVersion("2.0.0"));
  }

  @Test
  public void testCaseInsensitivePreRelease() {
    String upper = underTest.getNormalizedVersion("1.0.0.PRE");
    String lower = underTest.getNormalizedVersion("1.0.0.pre");
    assertThat(upper).isEqualTo(lower);
  }

  @Test
  public void testRcPreRelease() {
    String rc = underTest.getNormalizedVersion("1.0.0.rc1");
    String release = underTest.getNormalizedVersion("1.0.0");
    assertThat(rc).isLessThan(release);
  }

  @Test
  public void testPureLetterVersion() {
    String result = underTest.getNormalizedVersion("alpha");
    assertThat(result).isNotEmpty();
  }
}
