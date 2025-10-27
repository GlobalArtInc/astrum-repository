
package org.globalart.astrum.common.entity;

import java.util.UUID;

import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link EntityId} backed by a {@link UUID}.
 *
 * @since 3.19
 */
public class EntityUUID
    implements EntityId
{
  @JsonProperty("value")
  private final UUID id;

  public EntityUUID() {
    this.id = UUID.randomUUID();
  }

  public EntityUUID(final UUID id) {
    this.id = checkNotNull(id);
  }

  public UUID uuid() {
    return id;
  }

  @Override
  public String getValue() {
    return id.toString();
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o instanceof EntityUUID) {
      return id.equals(((EntityUUID) o).id);
    }
    if (o instanceof EntityId) {
      return getValue().equals(((EntityId) o).getValue());
    }
    return false;
  }

  @Override
  public int hashCode() {
    return getValue().hashCode();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "id='" + id + '\'' +
        '}';
  }
}
