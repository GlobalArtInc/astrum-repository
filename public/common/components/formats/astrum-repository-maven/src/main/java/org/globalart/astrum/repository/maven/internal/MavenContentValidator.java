
package org.globalart.astrum.repository.maven.internal;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven 2 specific {@link ContentValidator} that "hints" default content validator for some Maven specific file
 * extensions and format specific files.
 *
 * @since 3.0
 */
@Component
@Qualifier(Maven2Format.NAME)
public class MavenContentValidator
    implements ContentValidator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public MavenContentValidator(final DefaultContentValidator defaultContentValidator) {
    this.defaultContentValidator = checkNotNull(defaultContentValidator);
  }

  @Nonnull
  @Override
  public String determineContentType(
      final boolean strictContentTypeValidation,
      final InputStreamSupplier contentSupplier,
      @Nullable final MimeRulesSource mimeRulesSource,
      @Nullable final String contentName,
      @Nullable final String declaredContentType) throws IOException
  {
    if (contentName != null) {
      if (contentName.endsWith(".pom")) {
        // Note: this is due fact that Tika has glob "*.pom" extension enlisted at text/plain
        return defaultContentValidator.determineContentType(
            strictContentTypeValidation, contentSupplier, mimeRulesSource, contentName + ".xml", declaredContentType);
      }
      else if (isHashContentType(contentName)) {
        if (strictContentTypeValidation) {
          // hashes are small/simple, do it directly
          try (InputStream is = contentSupplier.get()) {
            final String digestCandidate = DigestExtractor.extract(is);
            if (!DigestExtractor.isDigest(digestCandidate)) {
              throw new InvalidContentException("Not a Maven2 digest: " + contentName);
            }
          }
        }

        if (declaredContentType != null) {
          return declaredContentType;
        }
      }
    }
    // everything else goes to default for now
    return defaultContentValidator.determineContentType(
        strictContentTypeValidation, contentSupplier, mimeRulesSource, contentName, declaredContentType);
  }

  private boolean isHashContentType(final String contentName) {
    return MavenPath.HashType.ALGORITHMS.stream()
        .anyMatch(algorithm -> contentName.endsWith(algorithm.name()));
  }
}
