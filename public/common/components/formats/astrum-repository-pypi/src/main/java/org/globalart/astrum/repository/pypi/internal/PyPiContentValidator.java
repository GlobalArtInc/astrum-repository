
package org.globalart.astrum.repository.pypi.internal;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.apache.commons.lang3.StringUtils.endsWith;

@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiContentValidator
    implements ContentValidator
{
  protected static final String ASC_FILE_EXTENSION = ".asc";

  protected static final String TEXT_FILE_EXTENSION = ".txt";

  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public PyPiContentValidator(final DefaultContentValidator defaultContentValidator) {
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
    String name = endsWith(contentName, ASC_FILE_EXTENSION) ? contentName + TEXT_FILE_EXTENSION : contentName;
    return defaultContentValidator
        .determineContentType(strictContentTypeValidation, contentSupplier, mimeRulesSource, name, declaredContentType);
  }
}
