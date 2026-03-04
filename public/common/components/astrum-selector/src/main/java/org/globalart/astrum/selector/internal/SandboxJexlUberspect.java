
package org.globalart.astrum.selector.internal;

import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Set;

import com.google.common.collect.ImmutableSet;
import org.apache.commons.jexl3.JexlEngine;
import org.apache.commons.jexl3.internal.introspection.Uberspect;
import org.apache.commons.jexl3.introspection.JexlMethod;
import org.apache.commons.jexl3.introspection.JexlPropertyGet;
import org.apache.commons.jexl3.introspection.JexlPropertySet;
import org.apache.commons.jexl3.introspection.JexlUberspect;
import org.apache.commons.logging.LogFactory;

/**
 * Uberspect for Jexl which blocks writes, restricts method calls to a whitelist,
 * and blocks reflection-based property reads on non-Map objects.
 *
 * @since 3.15
 */
public class SandboxJexlUberspect
    extends Uberspect
{
  private static final Set<String> COLLECTION_METHODS = ImmutableSet.of("contains");

  private static final Set<String> MAP_METHODS = ImmutableSet.of("get", "getOrDefault", "containsKey", "containsValue");

  private static final Set<String> STRING_METHODS = ImmutableSet.of("toUpperCase", "toLowerCase", "endsWith",
      "startsWith");

  public SandboxJexlUberspect() {
    super(LogFactory.getLog(JexlEngine.class), JexlUberspect.JEXL_STRATEGY);
  }

  @Override
  public JexlMethod getConstructor(final Object ctorHandle, final Object... args) {
    return null;
  }

  @Override
  public JexlMethod getMethod(final Object obj, final String method, final Object... args) {
    if (obj instanceof String && STRING_METHODS.contains(method)) {
      return super.getMethod(obj, method, args);
    }
    else if (obj instanceof Map && MAP_METHODS.contains(method)) {
      return super.getMethod(obj, method, args);
    }
    else if (obj instanceof Collection && COLLECTION_METHODS.contains(method)) {
      return super.getMethod(obj, method, args);
    }
    return null;
  }

  @Override
  public JexlPropertySet getPropertySet(final Object obj, final Object identifier, final Object arg) {
    return null;
  }

  @Override
  public JexlPropertySet getPropertySet(
      final List<PropertyResolver> resolvers,
      final Object obj,
      final Object identifier,
      final Object arg)
  {
    return null;
  }

  @Override
  public JexlPropertyGet getPropertyGet(final Object obj, final Object identifier) {
    // String access is restricted to whitelisted methods (see getMethod());
    // block all property reads on non-Map objects, including .class
    if (obj instanceof Map) {
      return super.getPropertyGet(obj, identifier);
    }
    return null;
  }

  @Override
  public JexlPropertyGet getPropertyGet(
      final List<PropertyResolver> resolvers,
      final Object obj,
      final Object identifier)
  {
    if (obj instanceof Map) {
      return super.getPropertyGet(resolvers, obj, identifier);
    }
    return null;
  }
}
