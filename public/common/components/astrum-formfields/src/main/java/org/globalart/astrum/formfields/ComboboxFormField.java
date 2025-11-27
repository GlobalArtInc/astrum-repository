
package org.globalart.astrum.formfields;

import java.util.Map;

import com.google.common.collect.Maps;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A mutable combo-box {@link FormField}.
 *
 * @since 2.7
 */
public class ComboboxFormField<V>
    extends Combobox<V>
{

  private String storeApi;

  private final Map<String, String> storeFilters;

  private String idMapping;

  private String nameMapping;

  public ComboboxFormField(
      final String id,
      final String label,
      final String helpText,
      final boolean required,
      final V initialValue)
  {
    super(id, label, helpText, required, initialValue);
    this.storeFilters = Maps.newHashMap();
  }

  public ComboboxFormField(
      final String id,
      final String label,
      final String helpText,
      final boolean required)
  {
    this(id, label, helpText, required, null);
  }

  public ComboboxFormField(
      final String id,
      final String label,
      final String helpText)
  {
    this(id, label, helpText, OPTIONAL);
  }

  public ComboboxFormField(
      final String id,
      final String label)
  {
    this(id, label, null);
  }

  /**
   * @since 3.0
   */
  @Override
  public String getStoreApi() {
    return storeApi;
  }

  /**
   * @since 3.0
   */
  @Override
  public Map<String, String> getStoreFilters() {
    return storeFilters.isEmpty() ? null : storeFilters;
  }

  @Override
  public String getIdMapping() {
    return idMapping;
  }

  @Override
  public String getNameMapping() {
    return nameMapping;
  }

  /**
   * @since 3.0
   */
  public ComboboxFormField<V> withStoreApi(final String storeApi) {
    this.storeApi = checkNotNull(storeApi);
    return this;
  }

  /**
   * Adds a store filter.
   *
   * @param property filter property
   * @param value filter value
   * @since 3.0
   */
  public Combobox<V> withStoreFilter(final String property, final String value) {
    storeFilters.put(checkNotNull(property, "property"), checkNotNull(value, "value"));
    return this;
  }

  public ComboboxFormField<V> withIdMapping(final String idMapping) {
    this.idMapping = idMapping;
    return this;
  }

  public ComboboxFormField<V> withNameMapping(final String nameMapping) {
    this.nameMapping = nameMapping;
    return this;
  }

}
