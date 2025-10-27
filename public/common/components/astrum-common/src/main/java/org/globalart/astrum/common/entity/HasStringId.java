
package org.globalart.astrum.common.entity;

/**
 * Mix-in for entities that use an externally assigned {@link String} id.
 *
 * @since 3.21
 */
public interface HasStringId
{
  String getId();

  void setId(String id);
}
