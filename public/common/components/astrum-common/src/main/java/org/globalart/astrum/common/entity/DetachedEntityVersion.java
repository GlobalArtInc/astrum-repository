
package org.globalart.astrum.common.entity;

import java.io.Serializable;

import javax.annotation.Nonnull;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Detached {@link EntityVersion}.
 *
 * @since 3.0
 */
public class DetachedEntityVersion
    implements EntityVersion, Serializable
{
  private static final long serialVersionUID = 1L;

  private final String value;

  public DetachedEntityVersion(final String value) {
    this.value = checkNotNull(value);
  }

  @Override
  @Nonnull
  public String getValue() {
    return value;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    else if (!(o instanceof EntityVersion)) {
      return false;
    }

    EntityVersion that = (EntityVersion) o;
    return getValue().equals(that.getValue());
  }

  @Override
  public int hashCode() {
    return getValue().hashCode();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "value='" + value + '\'' +
        '}';
  }
}
