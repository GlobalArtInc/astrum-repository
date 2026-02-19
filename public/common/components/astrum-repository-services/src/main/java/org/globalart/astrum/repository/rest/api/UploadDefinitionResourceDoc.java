
package org.globalart.astrum.repository.rest.api;

import java.util.List;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;

/**
 * Swagger documentation for {@link UploadDefinitionResource}
 */
@Tag(name = "Formats")
public interface UploadDefinitionResourceDoc
{
  @Operation(summary = "Get upload field requirements for each supported format")
  List<UploadDefinitionXO> get();

  @Operation(summary = "Get upload field requirements for the desired format")
  UploadDefinitionXO get(@Parameter(description = "The desired repository format") final String format);
}
