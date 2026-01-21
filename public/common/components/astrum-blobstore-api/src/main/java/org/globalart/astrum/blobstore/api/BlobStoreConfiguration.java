
package org.globalart.astrum.blobstore.api;

import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;

/**
 * All of the configuration for a particular {@link BlobStore}
 * 
 * @since 3.0
 */
public interface BlobStoreConfiguration
{
  String getName();

  void setName(final String name);

  String getType();

  void setType(final String type);

  Map<String, Map<String, Object>> getAttributes();

  void setAttributes(final Map<String, Map<String, Object>> attributes);

  NestedAttributesMap attributes(final String key);

  BlobStoreConfiguration copy(String name);

  boolean isWritable();

  void setWritable(final boolean writable);
}
