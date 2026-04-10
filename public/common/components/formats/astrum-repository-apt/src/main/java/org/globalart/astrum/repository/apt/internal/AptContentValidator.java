
package org.globalart.astrum.repository.apt.internal;

import java.io.IOException;
import java.util.Objects;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Apt-specific {@link ContentValidator} that "hints" default content validator for apt package metadata and format
 * specific files
 *
 * @since 3.17
 */
@Component
@Qualifier(AptFormat.NAME)
public class AptContentValidator
    implements ContentValidator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String TEXT_PLAIN = "text/plain";

  private static final String TXT = ".txt";

  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public AptContentValidator(final DefaultContentValidator defaultContentValidator) {
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
    String name = contentName;
    // Append .txt only for extension-less metadata files (Packages, Release, InRelease, etc.)
    // so the default validator can confirm they are plain text. Files with extensions like
    // Release.gpg or Packages.gz must not be renamed — the extension already conveys the type.
    if (name != null && Objects.equals(declaredContentType, TEXT_PLAIN)) {
      String baseName = name.contains("/") ? name.substring(name.lastIndexOf('/') + 1) : name;
      if (!baseName.contains(".")) {
        name += TXT;
      }
    }
    return defaultContentValidator.determineContentType(
        strictContentTypeValidation, contentSupplier, mimeRulesSource, name, declaredContentType);
  }
}
