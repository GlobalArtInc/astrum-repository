
package org.globalart.astrum.repository.conan.internal;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;
import org.globalart.astrum.repository.view.ContentTypes;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(ConanFormat.NAME)
public class ConanContentValidator
    implements ContentValidator
{
  private static final String X_PYTHON = "text/x-python";

  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public ConanContentValidator(final DefaultContentValidator defaultContentValidator) {
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
    if (contentName != null && !ConanPaths.isMetadata(contentName)) {
      if (contentName.endsWith(".json")) {
        return ContentTypes.APPLICATION_JSON;
      }
      if (contentName.endsWith(".tgz")) {
        return ContentTypes.APPLICATION_GZIP;
      }
      if (contentName.endsWith(".txz")) {
        return "application/x-xz";
      }
      if (contentName.endsWith(".tzst")) {
        return "application/zstd";
      }
      if (contentName.endsWith("/" + ConanPaths.CONAN_FILE)) {
        return X_PYTHON;
      }
      if (contentName.endsWith("/" + ConanPaths.CONAN_MANIFEST) || contentName.endsWith("/" + ConanPaths.CONAN_INFO)) {
        return ContentTypes.TEXT_PLAIN;
      }
    }
    boolean strict = strictContentTypeValidation && hasExtension(contentName);
    return defaultContentValidator.determineContentType(
        strict, contentSupplier, mimeRulesSource, contentName, declaredContentType);
  }

  private static boolean hasExtension(@Nullable final String contentName) {
    if (contentName == null) {
      return false;
    }
    String name = contentName.substring(contentName.lastIndexOf('/') + 1);
    return name.lastIndexOf('.') > 0;
  }
}
