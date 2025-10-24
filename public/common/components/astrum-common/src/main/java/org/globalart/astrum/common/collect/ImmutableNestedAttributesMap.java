
package org.globalart.astrum.common.collect;

import java.util.Collections;
import java.util.Map;

import javax.annotation.Nullable;

import com.google.common.collect.ImmutableMap;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * An immutable {@link NestedAttributesMap}.
 *
 * @since 3.0
 */
public class ImmutableNestedAttributesMap
    extends NestedAttributesMap
{
  public ImmutableNestedAttributesMap(
      @Nullable final NestedAttributesMap parent,
      final String key,
      final Map<String, Object> backing)
  {
    super(parent, key, Collections.unmodifiableMap(backing));
  }

  /**
   * Returns nested children attributes for given name.
   */
  @Override
  @SuppressWarnings("unchecked")
  public NestedAttributesMap child(final String name) {
    checkNotNull(name);

    Object child = backing.get(name);
    if (child == null) {
      child = ImmutableMap.of();
    }
    else {
      checkState(child instanceof Map, "child '%s' not a Map", name);
    }
    // noinspection unchecked,ConstantConditions
    return new ImmutableNestedAttributesMap(this, name, (Map<String, Object>) child);
  }
}
