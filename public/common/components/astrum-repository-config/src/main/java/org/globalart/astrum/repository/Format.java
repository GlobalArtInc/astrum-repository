
package org.globalart.astrum.repository;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Repository format symbol.
 *
 * @since 3.0
 */
public abstract class Format
{
  private final String value;

  public Format(final String value) {
    this.value = checkNotNull(value);
  }

  public String getValue() {
    return value;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }

    Format format = (Format) o;

    if (!value.equals(format.value)) {
      return false;
    }

    return true;
  }

  @Override
  public int hashCode() {
    return value.hashCode();
  }

  @Override
  public String toString() {
    return value;
  }
}
