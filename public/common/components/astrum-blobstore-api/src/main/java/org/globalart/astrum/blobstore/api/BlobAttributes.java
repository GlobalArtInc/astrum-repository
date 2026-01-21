
package org.globalart.astrum.blobstore.api;

import java.io.IOException;
import java.util.Map;
import java.util.Optional;
import java.util.Properties;

import org.joda.time.DateTime;

/**
 * @since 3.4
 */
public interface BlobAttributes
{
  Map<String, String> getHeaders();

  BlobMetrics getMetrics();

  boolean isDeleted();

  void setDeleted(boolean deleted);

  void setDeletedReason(String deletedReason);

  String getDeletedReason();

  /**
   * @since 3.17
   */
  DateTime getDeletedDateTime();

  /**
   * @since 3.17
   */
  void setDeletedDateTime(DateTime deletedDateTime);

  Properties getProperties();

  /**
   * Update attributes based on the {@link BlobAttributes} provided.
   *
   * @since 3.7
   */
  void updateFrom(BlobAttributes blobAttributes);

  /**
   * Stores the attributes in the blob store.
   *
   * @since 3.12
   */
  void store() throws IOException;

  /**
   * Write the attributes in the `propertiesFile`.
   */
  void writeProperties();

  /**
   * Set attributes with the original location upon soft-deletion.
   */
  void setOriginalLocation(String path);

  /**
   * When this attributes represents a properties file which was copied to a new location upon soft-deletion, this
   * method will return an optional containing the original location, empty otherwise.
   */
  Optional<String> getOriginalLocation();

  /**
   * When this attributes represents a properties file which was copied upon soft-deletion, this method will return an
   * optional containing the changed location, empty otherwise.
   */
  Optional<String> getSoftDeletedLocation();

  /**
   * Set attributes with the new location upon soft-deletion.
   */
  void setSoftDeletedLocation(String path);
}
