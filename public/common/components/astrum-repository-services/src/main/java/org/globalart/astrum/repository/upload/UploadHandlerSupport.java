
package org.globalart.astrum.repository.upload;

import java.util.List;
import java.util.Set;

import org.globalart.astrum.repository.rest.UploadDefinitionExtension;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.emptyList;
import static java.util.stream.Collectors.toList;
import static java.util.stream.Stream.concat;

/**
 * Base class for format specific {@link UploadHandler} classes which
 * enables {@link UploadFieldDefinition} contributions to the format
 * specific {@link UploadDefinition}
 *
 * @since 3.10
 */
public abstract class UploadHandlerSupport
    implements UploadHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Set<UploadDefinitionExtension> uploadDefinitionExtensions;

  public UploadHandlerSupport(final Set<UploadDefinitionExtension> uploadDefinitionExtensions) {
    this.uploadDefinitionExtensions = checkNotNull(uploadDefinitionExtensions);
  }

  /**
   * Provides a mechanism for subclasses to generate an UploadDefinition that allows for
   * extension point contributions.
   *
   * Order of the UploadFieldDefinitions is important as it affects the presentation in the ui.
   */
  public UploadDefinition getDefinition(
      final String format,
      final boolean multipleUpload,
      final List<UploadFieldDefinition> componentFields,
      final List<UploadFieldDefinition> assetFields,
      final UploadRegexMap regexMap)
  {

    // Gather the existing and contributed field definitions
    List<UploadFieldDefinition> componentFieldDefinitions = concat(
        componentFields.stream(),
        uploadDefinitionExtensions.stream().map(UploadDefinitionExtension::contribute)).collect(toList());

    return new UploadDefinition(format, supportsUiUpload(), supportsApiUpload(), multipleUpload,
        componentFieldDefinitions, assetFields, regexMap);
  }

  /**
   * Provides a mechanism for subclasses to generate an UploadDefinition that allows for
   * extension point contributions.
   *
   * This variant only requires the format and multiple upload argument.
   */
  public UploadDefinition getDefinition(final String format, final boolean multipleUpload) {
    return getDefinition(format, multipleUpload, emptyList(), emptyList(), null);
  }
}
