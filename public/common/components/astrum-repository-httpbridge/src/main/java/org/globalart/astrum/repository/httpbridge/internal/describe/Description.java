
package org.globalart.astrum.repository.httpbridge.internal.describe;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * Accumulates a renderable description of request-processing activity.
 *
 * @since 3.0
 */
public class Description
{
  private final Map<String, Object> parameters;

  private final List<DescriptionItem> items = new ArrayList<>();

  public Description(final Map<String, Object> parameters) {
    this.parameters = parameters;
  }

  public Description topic(final String name) {
    items.add(new DescriptionItem(name, "topic", name));
    return this;
  }

  public Description addTable(final String name, final Map<String, Object> values) {
    items.add(new DescriptionItem(name, "table", values));
    return this;
  }

  public Map<String, Object> getParameters() {
    return parameters;
  }

  public List<DescriptionItem> getItems() {
    return items;
  }
}
