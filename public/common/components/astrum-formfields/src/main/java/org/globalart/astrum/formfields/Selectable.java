
package org.globalart.astrum.formfields;

import java.util.Map;

import javax.annotation.Nullable;

/**
 * Implemented by {@link FormField}s whose value should be selected from a data store.
 *
 * The data store should return collections of records that have "id" and "name" fields.
 *
 * @since 2.7
 */
public interface Selectable
{
  /**
   * Returns Ext.Direct API name used to configure Ext proxy.
   *
   * E.g. "coreui_RepositoryTarget.read"
   *
   * @since 3.0
   */
  String getStoreApi();

  /**
   * @return Filters to be applied to store
   * @since 3.0
   */
  @Nullable
  Map<String, String> getStoreFilters();

  /**
   * Returns the name of the property that should be considered as an record id. Defaults to "id";
   */
  String getIdMapping();

  /**
   * Returns the name of the property that should be considered as an record description. Defaults to "name";
   */
  String getNameMapping();

}
