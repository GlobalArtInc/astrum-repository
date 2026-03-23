
package org.globalart.astrum.repository.maven;

/**
 * @since 3.33
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
