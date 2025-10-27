
package org.globalart.astrum.common.entity;

import javax.annotation.Nullable;

/**
 * Entity.
 *
 * @since 3.7
 */
public interface Entity
{
  @Nullable
  EntityMetadata getEntityMetadata();

  void setEntityMetadata(@Nullable final EntityMetadata metadata);
}
