
package org.globalart.astrum.security.config;

import java.io.Serializable;
import java.util.Map;

/**
 * Persistent privilege.
 *
 * @since 3.0
 */
public interface CPrivilege
    extends Serializable, Cloneable
{
  void setProperty(String key, String value);

  String getDescription();

  String getId();

  String getName();

  Map<String, String> getProperties();

  String getProperty(String key);

  String getType();

  boolean isReadOnly();

  void removeProperty(String key);

  void setDescription(String description);

  void setId(String id);

  void setName(String name);

  void setProperties(Map<String, String> properties);

  void setReadOnly(boolean readOnly);

  void setType(String type);

  int getVersion();

  void setVersion(int version);

  CPrivilege clone();
}
