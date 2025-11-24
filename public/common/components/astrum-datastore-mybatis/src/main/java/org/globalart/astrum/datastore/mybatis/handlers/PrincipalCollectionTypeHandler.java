

package org.globalart.astrum.datastore.mybatis.handlers;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.LinkedList;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.datastore.mybatis.AbstractSerializableTypeHandler;
import org.globalart.astrum.security.AstrumSimplePrincipalCollection;
import org.globalart.astrum.security.RealmCaseMapping;
import org.globalart.astrum.security.anonymous.AnonymousPrincipalCollection;

import org.apache.ibatis.type.TypeHandler;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;

/**
 * MyBatis {@link TypeHandler} that serializes and encrypts {@link PrincipalCollection}s to/from SQL.
 *
 * @since 3.21
 */
// not @Component
public class PrincipalCollectionTypeHandler
    extends AbstractSerializableTypeHandler<PrincipalCollection>
{
  /**
   * Allow-list for classes used by Shiro's {@code PrincipalCollection} serialization.
   * Includes Shiro internals, Astrum security classes, and common JDK collection types.
   */
  private static final Set<Class<?>> ALLOWED_CLASSES = Set.of(
      // Shiro PrincipalCollection classes
      SimplePrincipalCollection.class,
      // Astrum PrincipalCollection classes
      AnonymousPrincipalCollection.class,
      AstrumSimplePrincipalCollection.class,
      RealmCaseMapping.class,
      // Common JDK types used by SimplePrincipalCollection
      LinkedHashMap.class,
      HashMap.class,
      Map.Entry[].class,
      Map.Entry.class,
      LinkedHashSet.class,
      HashSet.class,
      LinkedList.class,
      ArrayList.class,
      String.class,
      Integer.class,
      Long.class,
      Number.class);

  @Override
  protected Set<Class<?>> getAllowedClasses() {
    return ALLOWED_CLASSES;
  }
}
