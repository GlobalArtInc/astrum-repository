
package org.globalart.astrum.common.entity;

import java.io.Serializable;

import javax.annotation.Nonnull;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Detached {@link EntityId}.
 *
 * @since 3.0
 */
public class DetachedEntityId
    implements EntityId, Serializable
{
  private static final long serialVersionUID = 1L;

  private final String value;

  public DetachedEntityId(final String value) {
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
    else if (!(o instanceof EntityId)) {
      return false;
    }

    EntityId that = (EntityId) o;
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
