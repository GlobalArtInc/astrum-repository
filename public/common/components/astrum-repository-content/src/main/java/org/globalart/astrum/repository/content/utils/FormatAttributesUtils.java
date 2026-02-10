
package org.globalart.astrum.repository.content.utils;

import java.util.Collections;
import java.util.HashMap;
import java.util.Map;
import java.util.Set;
import java.util.function.Supplier;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.RepositoryContent;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAttributes;
import org.globalart.astrum.repository.content.fluent.FluentComponent;

/**
 * @since 3.29
 */
public class FormatAttributesUtils
{
  public static Map<String, Object> getFormatAttributes(final FluentAsset fluent) {
    return getFormatAttributes(fluent.repository().getFormat().getValue(), fluent);
  }

  public static Map<String, Object> getFormatAttributes(final Asset asset, final String formatName) {
    return getFormatAttributes(formatName, asset);
  }

  public static Map<String, Object> getFormatAttributes(final FluentComponent fluent) {
    return getFormatAttributes(fluent.repository().getFormat().getValue(), fluent);
  }

  public static void setFormatAttributes(final FluentAsset fluent, final Map<String, Object> values) {
    setFormatAttributes(fluent.repository().getFormat().getValue(), fluent, fluent, values);
  }

  public static void setFormatAttributes(final FluentAsset fluent, final Supplier<Map<String, Object>> supplier) {
    setFormatAttributes(fluent, supplier.get());
  }

  public static void setFormatAttributes(final FluentAsset fluent, final String key, final Object value) {
    setFormatAttributes(
        fluent,
        Collections.singletonMap(key, value));
  }

  public static void setFormatAttributes(final FluentComponent fluent, final Map<String, Object> values) {
    setFormatAttributes(fluent.repository().getFormat().getValue(), fluent, fluent, values);
  }

  public static void removeFormatAttributes(final FluentComponent fluent, final Set<String> keys) {
    removeFormatAttributes(
        fluent.repository().getFormat().getValue(),
        fluent,
        fluent,
        keys);
  }

  public static void removeFormatAttributes(final FluentAsset fluent, final Set<String> keys) {
    removeFormatAttributes(
        fluent.repository().getFormat().getValue(),
        fluent,
        fluent,
        keys);
  }

  public static void removeFormatAttributes(final FluentComponent fluent, final String key) {
    removeFormatAttributes(
        fluent,
        Collections.singleton(key));
  }

  public static void removeFormatAttributes(final FluentAsset fluent, final String key) {
    removeFormatAttributes(
        fluent,
        Collections.singleton(key));
  }

  private static Map<String, Object> getFormatAttributes(
      final String formatName,
      final RepositoryContent repositoryContent)
  {
    @SuppressWarnings("unchecked")
    Map<String, Object> attributes = (Map<String, Object>) repositoryContent
        .attributes()
        .get(formatName);
    if (attributes == null) {
      return new HashMap<>();
    }
    // "Attributes" can be CollectionSingletonMap or ImmutableMap for example,
    // that does not support methods such as put, putAll and so on.
    // To support setFormatAttributes, removeFormatAttributes, here is necessary to repack it into "Map",
    // that supports put, remove. HashMap - good candidate.
    return new HashMap<>(attributes);
  }

  private static void setFormatAttributes(
      final String formatName,
      final RepositoryContent repositoryContent,
      final FluentAttributes attributes,
      final Map<String, Object> values)
  {
    Map<String, Object> formatAttributes = getFormatAttributes(formatName, repositoryContent);
    formatAttributes.putAll(values);
    attributes.withAttribute(formatName, formatAttributes);
  }

  private static void removeFormatAttributes(
      final String formatName,
      final RepositoryContent repositoryContent,
      final FluentAttributes attributes,
      final Set<String> keys)
  {
    Map<String, Object> formatAttributes = getFormatAttributes(formatName, repositoryContent);
    keys.forEach(key -> formatAttributes.remove(key));
    attributes.withAttribute(formatName, formatAttributes);
  }
}
