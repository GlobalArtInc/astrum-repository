
package org.globalart.astrum.api.rest.selfhosted.email;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.api.rest.selfhosted.email.model.ApiEmailConfiguration;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import org.globalart.astrum.api.rest.selfhosted.email.model.ApiEmailValidation;

/**
 * Swagger documentation for {@link EmailConfigurationApiResource}
 *
 * @since 3.19
 */
@Tag(name = "Email")
public interface EmailConfigurationApiResourceDoc
{
  @Operation(summary = "Retrieve the current email configuration")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Email configuration returned",
          content = @Content(schema = @Schema(implementation = ApiEmailConfiguration.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to retrieve the email configuration")
  })
  ApiEmailConfiguration getEmailConfiguration();

  @Operation(summary = "Set the current email configuration")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Email configuration was successfully updated"),
      @ApiResponse(responseCode = "400", description = "Invalid request"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to update the email configuration")
  })
  void setEmailConfiguration(@Parameter(required = true) @NotNull @Valid ApiEmailConfiguration emailConfiguration);

  @Operation(summary = "Send a test email to the email address provided in the request body")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Test email was sent successfully",
          content = @Content(schema = @Schema(implementation = ApiEmailValidation.class))),
      @ApiResponse(responseCode = "400", description = "There was a problem sending the test email" /*
                                                                                                     * ASTRUM-46395
                                                                                                     * TODO:
                                                                                                     * examples=
                                                                                                     * dropped; use
                                                                                                     * OpenAPI
                                                                                                     * 3 @Content(
                                                                                                     * examples
                                                                                                     * = @ExampleObject
                                                                                                     * (...))
                                                                                                     */,
          content = @Content(schema = @Schema(implementation = ApiEmailValidation.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to verify the email configuration")
  })
  Response testEmailConfiguration(
      @Parameter(required = true,
          description = "An email address to send a test email to") @NotNull String validationEmail);

  @Operation(summary = "Disable and clear the email configuration")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Email configuration was successfully cleared")
  })
  void deleteEmailConfiguration();
}
