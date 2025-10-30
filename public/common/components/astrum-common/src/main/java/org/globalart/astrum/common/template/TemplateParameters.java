
package org.globalart.astrum.common.template;

import java.util.HashMap;
import java.util.Map;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Helper to build template parameters map.
 *
 * @since 3.0
 */
public class TemplateParameters
{
  private final Map<String, Object> params;

  public TemplateParameters(final Map<String, Object> params) {
    this.params = checkNotNull(params);
  }

  public TemplateParameters() {
    this(new HashMap<>());
  }

  public TemplateParameters set(final String key, final Object value) {
    params.put(key, value);
    return this;
  }

  public TemplateParameters setAll(final Map<String, Object> entries) {
    params.putAll(entries);
    return this;
  }

  public Map<String, Object> get() {
    return params;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "params=" + params +
        '}';
  }
}
