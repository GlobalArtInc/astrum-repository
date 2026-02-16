
package org.globalart.astrum.repository.httpbridge.internal.describe;

/**
 * One element of a {@link Description}.
 *
 * @since 3.0
 */
public class DescriptionItem
{
  private final String name;

  private final String type;

  /**
   * Value should be a primitive/char-sequence or collection/map of primitive/char-sequence
   * to support html + json rendering w/o overly complex mapping configurations.
   *
   * @see DescriptionHelper#convert(Object)
   */
  private final Object value;

  DescriptionItem(final String name, final String type, final Object value) {
    this.name = name;
    this.type = type;
    this.value = value;
  }

  public String getName() {
    return name;
  }

  public String getType() {
    return type;
  }

  public Object getValue() {
    return value;
  }
}
