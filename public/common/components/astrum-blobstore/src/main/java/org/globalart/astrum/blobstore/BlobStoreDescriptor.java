
package org.globalart.astrum.blobstore;

import java.util.Collections;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.formfields.FormField;

/**
 * Describes a blob store.
 * Note: Do not inject Map<String, BlobStoreDescriptors> and instead opt to use BlobStoreDescriptorProvider
 *
 * @since 3.6
 */
public interface BlobStoreDescriptor
{
  /**
   * A url-friendly identifier. This must match the identifier used for the blob store REST apis.
   */
  String getId();

  /**
   * A user friendly name of the blob store type to be presented in UI.
   *
   * @return blob store name
   */
  String getName();

  /**
   * Form fields to configure the blob store.
   *
   * @return blob store configuration form fields
   */
  List<FormField> getFormFields();

  /**
   * Name of a custom form for configuring the blob store.
   *
   * @return custom form name
   */
  @Nullable
  default String customFormName() {
    return null;
  }

  /**
   * @return true if the blob store can be modified after creating.
   *
   * @since 3.14
   */
  default boolean isModifiable() {
    return true;
  }

  /**
   * Validate configuration.
   *
   * @since 3.14
   */
  default void validateConfig(BlobStoreConfiguration config) {
  }

  /**
   * Modifies the config to ensure the input is valid
   *
   * @since 3.17
   */
  default void sanitizeConfig(BlobStoreConfiguration config) {
  }

  /**
   * @return true if the blob store type is enabled.
   *
   * @since 3.14
   */
  default boolean isEnabled() {
    return true;
  }

  /**
   * @return true if the configuration has a dependency on another blobstore with the given name
   *
   * @since 3.14
   */
  default boolean configHasDependencyOn(BlobStoreConfiguration config, String blobStoreName) {
    return false;
  }

  /**
   * @return true if the the blob store has a connection to be tested by the backend.
   *
   * @since 3.30
   */
  default boolean isConnectionTestable() {
    return false;
  }

  /**
   * @return a list of the names of the fields that should be encrypted in the configuration
   */
  default List<String> getSensitiveConfigurationFields() {
    return Collections.emptyList();
  }

  default Map<String, List<SelectOption>> getDropDownValues() {
    return null;
  }
}
