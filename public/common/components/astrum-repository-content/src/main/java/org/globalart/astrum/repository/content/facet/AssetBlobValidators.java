
package org.globalart.astrum.repository.content.facet;

import java.io.IOException;
import java.util.List;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Supplies {@link AssetBlobValidator}s to {@link ContentFacet}s.
 *
 * @since 3.24
 */
@Component
public class AssetBlobValidators
{
  private final Map<String, MimeRulesSource> mimeRulesSources;

  private final Map<String, ContentValidator> contentValidators;

  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public AssetBlobValidators(
      final List<MimeRulesSource> mimeRulesSourcesList,
      final List<ContentValidator> contentValidatorsList,
      final DefaultContentValidator defaultContentValidator)
  {
    this.mimeRulesSources = QualifierUtil.buildQualifierBeanMap(checkNotNull(mimeRulesSourcesList));
    this.contentValidators = QualifierUtil.buildQualifierBeanMap(checkNotNull(contentValidatorsList));
    this.defaultContentValidator = checkNotNull(defaultContentValidator);
  }

  public AssetBlobValidator selectValidator(final Repository repository) {
    String format = repository.getFormat().getValue();
    MimeRulesSource mimeRulesSource = mimeRulesSources.getOrDefault(format, MimeRulesSource.NOOP);
    ContentValidator contentValidator = contentValidators.getOrDefault(format, defaultContentValidator);
    return (strict, contentSupplier, assetPath, declaredContentType) -> {
      try {
        return contentValidator.determineContentType(
            strict,
            contentSupplier,
            mimeRulesSource,
            assetPath,
            declaredContentType);
      }
      catch (IOException e) {
        throw new InvalidContentException("Content type could not be determined", e);
      }
    };
  }
}
