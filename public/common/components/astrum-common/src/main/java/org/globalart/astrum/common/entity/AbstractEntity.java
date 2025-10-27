
package org.globalart.astrum.common.entity;

import javax.annotation.Nullable;

// This class is intentionally uber-simple, DO NOT add more helpers to this implementation.

/**
 * @see EntityHelper
 * @since 3.7
 */
public abstract class AbstractEntity
    implements Entity
{
  private transient volatile EntityMetadata metadata;

  @Nullable
  public EntityMetadata getEntityMetadata() {
    return metadata;
  }

  public void setEntityMetadata(@Nullable final EntityMetadata metadata) {
    this.metadata = metadata;
  }
}
