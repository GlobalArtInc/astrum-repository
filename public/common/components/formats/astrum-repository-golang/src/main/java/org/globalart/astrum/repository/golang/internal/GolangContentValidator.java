
package org.globalart.astrum.repository.golang.internal;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.golang.GolangFormat;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(GolangFormat.NAME)
public class GolangContentValidator
    implements ContentValidator
{
  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public GolangContentValidator(final DefaultContentValidator defaultContentValidator) {
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
    return defaultContentValidator.determineContentType(
        strictContentTypeValidation, contentSupplier, mimeRulesSource, modifyContentName(contentName),
        declaredContentType);
  }

  @Nullable
  private String modifyContentName(@Nullable final String contentName) {
    if (contentName == null) {
      return null;
    }
    if (contentName.endsWith(".mod")) {
      return contentName + ".txt";
    }
    if (contentName.endsWith("/@v/list") || contentName.endsWith("/@latest") || contentName.endsWith(".info")) {
      return contentName + ".json";
    }
    return contentName;
  }
}
