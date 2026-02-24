
package org.globalart.astrum.repository.search.normalize;

import java.util.List;
import java.util.Map;
import java.util.Objects;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class VersionNormalizerService
{
  private final Map<String, VersionNormalizer> versionNormalizers;

  @Autowired
  public VersionNormalizerService(final List<VersionNormalizer> versionNormalizersList) {
    this.versionNormalizers = QualifierUtil.buildQualifierBeanMap(checkNotNull(versionNormalizersList));
  }

  /**
   * Normalizes supplied version using default approach or format specific
   * if format specific implementation of {@link VersionNormalizer} is present
   *
   * @param version version to be normalized
   * @param format format that component belongs to
   * @return normalized version
   */
  public String getNormalizedVersionByFormat(final String version, final Format format) {
    return getNormalizedVersionByFormat(version, format.getValue());
  }

  /**
   * Normalizes supplied version using default approach or format specific
   * if format specific implementation of {@link VersionNormalizer} is present
   *
   * @param version version to be normalized
   * @param formatName format that component belongs to
   * @return normalized version
   */
  public String getNormalizedVersionByFormat(final String version, final String formatName) {
    VersionNormalizer normalizerForFormat = versionNormalizers.get(formatName);
    if (Objects.nonNull(normalizerForFormat)) {
      return normalizerForFormat.getNormalizedVersion(version);
    }
    return VersionNumberExpander.expand(version);
  }
}
