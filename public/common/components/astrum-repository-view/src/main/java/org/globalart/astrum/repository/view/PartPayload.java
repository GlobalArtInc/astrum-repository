
package org.globalart.astrum.repository.view;

import javax.annotation.Nullable;

/**
 * Multipart payload.
 *
 * @since 3.1
 */
public interface PartPayload
    extends Payload
{
  /**
   * Returns the original name of this file as provided by the client.
   */
  @Nullable
  String getName();

  /**
   * Returns the name of the form field.
   */
  String getFieldName();

  /**
   * Returns true for form fields and false for files.
   */
  boolean isFormField();

  /**
   * Returns the value of the specified header for this part.
   * Returns null if the header is not present.
   *
   * @param name the header name (case-insensitive)
   * @return the header value, or null if not present
   * @since 3.80
   */
  @Nullable
  default String getHeader(final String name) {
    return null;
  }
}
