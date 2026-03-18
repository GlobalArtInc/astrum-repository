
package org.globalart.astrum.repository.raw;

/**
 * @since 3.25
 */
public enum ContentDisposition
{
  INLINE("inline"),
  ATTACHMENT("attachment");

  private final String value;

  ContentDisposition(final String value) {
    this.value = value;
  }

  public String getValue() {
    return value;
  }
}
