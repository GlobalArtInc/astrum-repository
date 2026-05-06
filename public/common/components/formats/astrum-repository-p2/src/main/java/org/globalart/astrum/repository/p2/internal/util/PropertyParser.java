
package org.globalart.astrum.repository.p2.internal.util;

import java.io.IOException;
import java.util.Optional;
import java.util.PropertyResourceBundle;
import java.util.jar.JarEntry;
import java.util.jar.JarInputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.p2.internal.exception.AttributeParsingException;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class PropertyParser
{
  private static final String PROPERTY_RESOURCE_BUNDLE_EXTENSION = ".properties";

  private JarExtractor<PropertyResourceBundle> jarExtractor;

  @Autowired
  public PropertyParser(final TempBlobConverter tempBlobConverter) {
    jarExtractor = new JarExtractor<PropertyResourceBundle>(tempBlobConverter)
    {
      @Override
      protected PropertyResourceBundle createSpecificEntity(JarInputStream jis, JarEntry jarEntry) throws IOException
      {
          return new PropertyResourceBundle(jis);
      }
    };
  }

  protected Optional<PropertyResourceBundle> getBundleProperties(
      final TempBlob tempBlob,
      final String extension,
      @Nullable final String startNameForSearch) throws IOException, AttributeParsingException
  {

    return jarExtractor.getSpecificEntity(tempBlob, extension, startNameForSearch + PROPERTY_RESOURCE_BUNDLE_EXTENSION);
  }

  @Nullable
  protected String extractValueFromProperty(
      @Nullable final String value,
      final Optional<PropertyResourceBundle> propertyResourceBundleOpt)
  {
    if (value == null || !value.startsWith("%")) {
      return value;
    }
    String key = value.substring(1);
    return propertyResourceBundleOpt
        .filter(bundle -> bundle.containsKey(key))
        .map(bundle -> bundle.getString(key))
        .orElse(null);
  }
}
