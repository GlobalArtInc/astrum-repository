
package org.globalart.astrum.rest;

import jakarta.ws.rs.core.MediaType;

/**
 * Siesta specific media types.
 *
 * @since 3.0
 */
public class MediaTypes
{
  private MediaTypes() {
    // empty
  }

  // application/vnd.siesta-validation-errors-v1+xml

  public static final String VND_VALIDATION_ERRORS_V1_XML = "application/vnd.siesta-validation-errors-v1+xml";

  public static final MediaType VND_VALIDATION_ERRORS_V1_XML_TYPE =
      new MediaType("application", "vnd.siesta-validation-errors-v1+xml");

  // application/vnd.siesta-validation-errors-v1+json

  public static final String VND_VALIDATION_ERRORS_V1_JSON = "application/vnd.siesta-validation-errors-v1+json";

  public static final MediaType VND_VALIDATION_ERRORS_V1_JSON_TYPE =
      new MediaType("application", "vnd.siesta-validation-errors-v1+json");
}
