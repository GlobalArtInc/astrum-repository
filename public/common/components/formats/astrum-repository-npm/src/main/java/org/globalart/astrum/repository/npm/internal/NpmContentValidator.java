
package org.globalart.astrum.repository.npm.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.util.List;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.mime.MimeRulesSource;
import org.globalart.astrum.repository.mime.ContentValidator;
import org.globalart.astrum.repository.mime.DefaultContentValidator;

import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

/**
 * npm specific {@link ContentValidator} that "hints" default content validator for npm metadata and format
 * specific files.
 *
 * @since 3.0
 */
@Component
@Qualifier(NpmFormat.NAME)
public class NpmContentValidator
    implements ContentValidator
{
  private static final List<String> NPM_METADATA_TYPES = ImmutableList.of(APPLICATION_JSON, TEXT_PLAIN);

  private final DefaultContentValidator defaultContentValidator;

  @Autowired
  public NpmContentValidator(final DefaultContentValidator defaultContentValidator) {
    this.defaultContentValidator = checkNotNull(defaultContentValidator);
  }

  @Nonnull
  @Override
  public String determineContentType(final boolean strictContentTypeValidation,
                                     final InputStreamSupplier contentSupplier,
                                     @Nullable final MimeRulesSource mimeRulesSource,
                                     @Nullable final String contentName,
                                     @Nullable final String declaredContentType) throws IOException
  {
    String name = restoreFileExtension(contentName, declaredContentType);
    return defaultContentValidator.determineContentType(
        strictContentTypeValidation, contentSupplier, mimeRulesSource, name, declaredContentType
    );
  }

  @Nullable
  private String restoreFileExtension(final String name, final String declaredContentType) {
    if (name == null) {
      return null;
    }
    if (name.endsWith(".json") || name.endsWith(".tgz")) {
      return name;
    }
    String baseContentType = declaredContentType == null ? null : declaredContentType.split(";")[0].trim();
    if (NPM_METADATA_TYPES.contains(baseContentType) || !name.contains("/-/")) {
      return name + ".json";
    }
    return name + ".tgz";
  }
}
