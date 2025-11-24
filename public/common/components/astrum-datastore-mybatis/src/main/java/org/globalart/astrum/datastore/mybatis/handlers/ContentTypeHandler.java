
package org.globalart.astrum.datastore.mybatis.handlers;

import org.apache.ibatis.type.TypeHandler;

/**
 * Marker interface that identifies content-related MyBatis {@link TypeHandler}s.
 *
 * These handlers are only used by the content data store(s) rather than the config data store.
 *
 * @since 3.21
 */
public interface ContentTypeHandler<T>
    extends TypeHandler<T>
{
  // nothing to add
}
