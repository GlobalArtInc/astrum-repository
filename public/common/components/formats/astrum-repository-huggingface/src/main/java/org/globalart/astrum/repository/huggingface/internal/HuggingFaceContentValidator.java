package org.globalart.astrum.repository.huggingface.internal;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Repositories on the Hub hold weights, tokenizers and data in formats no content detection knows, so the content
 * type of a file is determined but never enforced; the files are checked against the SHA-256 and size the Hub
 * announces instead.
 */
@Component
@Qualifier(HuggingFaceFormat.NAME)
public class HuggingFaceContentValidator
    implements ContentValidator
{
  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public HuggingFaceContentValidator(final DefaultContentValidator defaultContentValidator) {
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
    return defaultContentValidator.determineContentType(false, contentSupplier, mimeRulesSource, contentName,
        declaredContentType);
  }
}
