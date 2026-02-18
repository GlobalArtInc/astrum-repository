
package org.globalart.astrum.repository.json;

import java.util.HashMap;

import org.globalart.astrum.common.collect.NestedAttributesMap;

import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.ValueInstantiator;
import com.fasterxml.jackson.databind.deser.std.StdValueInstantiator;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Implementation of {@link StdValueInstantiator} that forces the providing and using of one root {@link
 * NestedAttributesMap}, this allows jackson {@link ObjectMapper}'s that deserialize into maps to always use
 * the shared map.
 *
 * @since 3.16
 */
public class NestedAttributesMapStdValueInstantiator
    extends ValueInstantiator.Base
    implements java.io.Serializable
{
  private final NestedAttributesMap root;

  public NestedAttributesMapStdValueInstantiator(final NestedAttributesMap root) {
    super(HashMap.class);
    this.root = checkNotNull(root);
  }

  /**
   * Overwritten to always pass back one and the same map for adding to
   */
  @Override
  public Object createUsingDefault(final DeserializationContext context) {
    return root.backing();
  }

  @Override
  public boolean canInstantiate() {
    return true;
  }

  @Override
  public boolean canCreateUsingDefault() {
    return true;
  }
}
