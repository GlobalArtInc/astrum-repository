
package org.globalart.astrum.api.rest.selfhosted.lifecycle;

// ASTRUM-46395 sample migration: Swagger 1.x annotations \u2192 OpenAPI 3.x (Flavor B in the
// resteasy-spike-notes.md). This file is the proof-of-concept for the 390-file annotation
// sweep that the full Phase 3 migration must perform. Mapping applied here:
//
//   @Tag(name = "X")                            \u2192 @Tag(name = "X")
//   @Operation(summary = "summary")             \u2192 @Operation(summary = "summary")
//   @Operation(summary = , notes=)        \u2192 @Operation(summary=, description=)
//   @Parameter(description = "description")             \u2192 @Parameter(description = "description")
//
// All replacement annotations live under io.swagger.v3.oas.annotations.* (provided by
// io.swagger.core.v3:swagger-annotations-jakarta).
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.tags.Tag;

/**
 * REST API to manage the Astrum application lifecycle.
 *
 * @since 3.16
 */
@Tag(name = "Lifecycle")
public interface ManagedLifecycleResourceDoc
{
  @Operation(summary = "Get current lifecycle phase")
  String getPhase();

  @Operation(summary = "Move to new lifecycle phase")
  void setPhase(@Parameter(description = "The phase to move to") final String phase);

  @Operation(summary = "Bounce lifecycle phase",
      description = "Re-runs all phases from the given phase to the current phase")
  void bounce(@Parameter(description = "The phase to bounce") final String phase);
}
