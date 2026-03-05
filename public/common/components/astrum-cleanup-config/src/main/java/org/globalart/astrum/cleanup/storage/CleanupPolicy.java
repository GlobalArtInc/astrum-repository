
package org.globalart.astrum.cleanup.storage;

import java.util.Map;

/**
 * Represents a configured cleanup policy
 *
 * @since 3.14
 */
public interface CleanupPolicy // NOSONAR
{
  String ALL_CLEANUP_POLICY_FORMAT = "ALL_FORMATS";

  String ALL_FORMATS = "*";

  String getName();

  void setName(final String name);

  String getNotes();

  void setNotes(final String notes);

  String getFormat();

  void setFormat(final String format);

  String getMode();

  void setMode(final String mode);

  Map<String, String> getCriteria();

  void setCriteria(final Map<String, String> criteria);
}
