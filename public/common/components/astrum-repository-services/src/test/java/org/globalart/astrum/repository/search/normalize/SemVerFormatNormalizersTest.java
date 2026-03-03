
package org.globalart.astrum.repository.search.normalize;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.QualifierUtil;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Verifies that helm, terraform, pub, and swift each resolve to their own @Component subclass
 * of SemVerVersionNormalizer, and that QualifierUtil correctly keys them by format name.
 */
public class SemVerFormatNormalizersTest
{
  @Test
  public void eachSemVerFormatHasDistinctQualifier() {
    List<VersionNormalizer> normalizers = List.of(
        new HelmVersionNormalizer(),
        new TerraformVersionNormalizer(),
        new PubVersionNormalizer(),
        new SwiftVersionNormalizer(),
        new NpmVersionNormalizer());

    Map<String, VersionNormalizer> map = QualifierUtil.buildQualifierBeanMap(normalizers);

    assertThat(map).containsKey("helm");
    assertThat(map).containsKey("terraform");
    assertThat(map).containsKey("pub");
    assertThat(map).containsKey("swift");
    assertThat(map).containsKey("npm");
  }

  @Test
  public void helmNormalizesReleaseVersionCorrectly() {
    assertThat(new HelmVersionNormalizer().getNormalizedVersion("1.2.3"))
        .isEqualTo("000000001.000000002.000000003.c");
  }

  @Test
  public void terraformNormalizesReleaseVersionCorrectly() {
    assertThat(new TerraformVersionNormalizer().getNormalizedVersion("6.46.0"))
        .isEqualTo("000000006.000000046.000000000.c");
  }

  @Test
  public void pubNormalizesReleaseVersionCorrectly() {
    assertThat(new PubVersionNormalizer().getNormalizedVersion("1.0.0"))
        .isEqualTo("000000001.000000000.000000000.c");
  }

  @Test
  public void swiftNormalizesReleaseVersionCorrectly() {
    assertThat(new SwiftVersionNormalizer().getNormalizedVersion("2.0.0"))
        .isEqualTo("000000002.000000000.000000000.c");
  }

  @Test
  public void allSubclassesProduceSameOutputAsSemVer() {
    SemVerVersionNormalizer semver = new SemVerVersionNormalizer();
    String version = "3.1.0-alpha.1";

    assertThat(new HelmVersionNormalizer().getNormalizedVersion(version))
        .isEqualTo(semver.getNormalizedVersion(version));
    assertThat(new TerraformVersionNormalizer().getNormalizedVersion(version))
        .isEqualTo(semver.getNormalizedVersion(version));
    assertThat(new PubVersionNormalizer().getNormalizedVersion(version))
        .isEqualTo(semver.getNormalizedVersion(version));
    assertThat(new SwiftVersionNormalizer().getNormalizedVersion(version))
        .isEqualTo(semver.getNormalizedVersion(version));
  }
}
