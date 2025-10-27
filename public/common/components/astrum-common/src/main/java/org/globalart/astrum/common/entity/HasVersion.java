
package org.globalart.astrum.common.entity;

/**
 * Mix-in for entities that want to track their version; for example to detect concurrent updates.
 *
 * @since 3.19
 */
public interface HasVersion
{
  int getVersion();

  void setVersion(int version);
}
