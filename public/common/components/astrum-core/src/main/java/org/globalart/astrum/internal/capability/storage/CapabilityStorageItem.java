
package org.globalart.astrum.internal.capability.storage;

import java.util.Map;

/**
 * @since 2.8
 */
public interface CapabilityStorageItem
{
  int getVersion();

  void setVersion(int version);

  String getType();

  void setType(String type);

  boolean isEnabled();

  void setEnabled(boolean enabled);

  String getNotes();

  void setNotes(final String notes);

  Map<String, String> getProperties();

  void setProperties(final Map<String, String> properties);
}
