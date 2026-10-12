package org.globalart.astrum.repository.gitlfs.internal;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Git LFS objects are arbitrary files checked by their SHA-256, so their content type is detected but never enforced.
 */
@Component
@Qualifier(GitLfsFormat.NAME)
public class GitLfsContentValidator
    implements ContentValidator
{
  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public GitLfsContentValidator(final DefaultContentValidator defaultContentValidator) {
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
