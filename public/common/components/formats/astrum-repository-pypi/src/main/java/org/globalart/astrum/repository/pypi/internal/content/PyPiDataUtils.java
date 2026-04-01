
package org.globalart.astrum.repository.pypi.internal.content;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.content.RepositoryContent;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAttributes;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.view.Content;

import com.google.common.base.Strings;
import org.joda.time.DateTime;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.SUPPORTED_ATTRIBUTES;
import static org.globalart.astrum.repository.view.Content.CONTENT_LAST_MODIFIED;

public final class PyPiDataUtils
{
  private PyPiDataUtils() {
  }

  static void copySupportedAttributes(final FluentAsset asset, final Map<String, String> attributes) {
    checkNotNull(asset);
    checkNotNull(attributes);
    Map<String, Object> formatAttributes = formatAttributes(asset.attributes().get(PyPiFormat.NAME));
    for (String attribute : SUPPORTED_ATTRIBUTES) {
      String value = Strings.nullToEmpty(attributes.get(attribute)).trim();
      if (!value.isEmpty()) {
        formatAttributes.put(attribute, value);
      }
    }
    asset.withAttribute(PyPiFormat.NAME, formatAttributes);
  }

  static <A extends FluentAttributes<A>> void copyFormatAttributes(
      final A fluentAttributes,
      final Map<String, ?> attributes)
  {
    checkNotNull(fluentAttributes);
    checkNotNull(attributes);
    Map<String, Object> formatAttributes = formatAttributes(currentAttributes(fluentAttributes));
    formatAttributes.putAll(attributes);
    fluentAttributes.withAttribute(PyPiFormat.NAME, formatAttributes);
  }

  static <A extends FluentAttributes<A>> void setFormatAttribute(
      final A fluentAttributes,
      final String key,
      final Object value)
  {
    checkNotNull(fluentAttributes);
    checkNotNull(key);
    checkNotNull(value);
    Map<String, Object> formatAttributes = formatAttributes(currentAttributes(fluentAttributes));
    formatAttributes.put(key, value);
    fluentAttributes.withAttribute(PyPiFormat.NAME, formatAttributes);
  }

  private static Object currentAttributes(final FluentAttributes<?> fluentAttributes) {
    if (fluentAttributes instanceof RepositoryContent content) {
      return content.attributes().get(PyPiFormat.NAME);
    }
    return null;
  }

  @SuppressWarnings("unchecked")
  private static Map<String, Object> formatAttributes(final Object existing) {
    Map<String, Object> result = new HashMap<>();
    if (existing instanceof Map) {
      result.putAll((Map<String, Object>) existing);
    }
    return result;
  }

  static void setLastModified(final Content content) {
    if (!content.getAttributes().contains(CONTENT_LAST_MODIFIED)) {
      content.getAttributes().set(CONTENT_LAST_MODIFIED, DateTime.now());
    }
  }

  public static Optional<String> getChecksum(final FluentAsset asset, final HashAlgorithm algorithm) {
    checkNotNull(asset);
    return asset.blob().map(blob -> blob.checksums().get(algorithm.name()));
  }
}
