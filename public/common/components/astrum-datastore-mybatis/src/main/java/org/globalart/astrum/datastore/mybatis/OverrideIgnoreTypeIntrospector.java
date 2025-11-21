
package org.globalart.astrum.datastore.mybatis;

import java.lang.annotation.Annotation;
import java.util.List;

import com.fasterxml.jackson.annotation.JsonIgnoreType;
import com.fasterxml.jackson.databind.introspect.Annotated;
import com.fasterxml.jackson.databind.introspect.JacksonAnnotationIntrospector;

/**
 * Introspector that overrides {@link JsonIgnoreType} annotation for specific types.
 */
public class OverrideIgnoreTypeIntrospector
    extends JacksonAnnotationIntrospector
{
  private final List<Class<?>> overriddenTypes;

  public OverrideIgnoreTypeIntrospector(final List<Class<?>> overriddenTypes) {
    super();
    this.overriddenTypes = overriddenTypes;
  }

  @Override
  protected <A extends Annotation> A _findAnnotation(final Annotated ann, final Class<A> annoClass) {
    if (annoClass == JsonIgnoreType.class && shouldOverride(ann.getRawType())) {
      return null;
    }

    return super._findAnnotation(ann, annoClass);
  }

  private boolean shouldOverride(final Class<?> type) {
    return overriddenTypes.stream().anyMatch(overriddenType -> overriddenType.isAssignableFrom(type));
  }
}
