
package org.globalart.astrum.repository.search.normalize;

import java.util.List;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Regression coverage for the qualifier collision fixed by giving each SemVer-compatible format its own
 * {@link SemVerVersionNormalizer} subclass (see that class's javadoc). Built with the real
 * {@code VersionNormalizer} beans, the same way Spring would inject them, rather than mocks, so that a
 * future reintroduction of the shared-instance aliasing bug would fail these tests.
 */
public class VersionNormalizerServiceTest
{
  private final VersionNormalizerService underTest = new VersionNormalizerService(List.of(
      new NpmVersionNormalizer(),
      new HelmVersionNormalizer(),
      new PubVersionNormalizer(),
      new TerraformVersionNormalizer(),
      new SwiftVersionNormalizer(),
      new GoVersionNormalizer(new NpmVersionNormalizer())));

  @Test
  public void testEachSemVerFormatResolvesItsOwnNormalizerRatherThanFallingBackToGenericExpansion() {
    for (String format : List.of("npm", "helm", "pub", "terraform", "swift")) {
      assertThat(underTest.getNormalizedVersionByFormat("1.0.0", format))
          .as("format=%s", format)
          .endsWith(".c");
      assertThat(underTest.getNormalizedVersionByFormat("1.0.0-alpha", format))
          .as("format=%s", format)
          .contains(".a.");
    }
  }

  @Test
  public void testAliasedFormatsProduceTheSameEncodingAsNpm() {
    String npmEncoding = underTest.getNormalizedVersionByFormat("1.2.3-beta.1", "npm");

    for (String format : List.of("helm", "pub", "terraform", "swift")) {
      assertThat(underTest.getNormalizedVersionByFormat("1.2.3-beta.1", format))
          .as("format=%s", format)
          .isEqualTo(npmEncoding);
    }
  }

  @Test
  public void testUnrecognizedFormatFallsBackToGenericExpansion() {
    assertThat(underTest.getNormalizedVersionByFormat("1.0.0", "maven2"))
        .isEqualTo(VersionNumberExpander.expand("1.0.0"));
  }
}
