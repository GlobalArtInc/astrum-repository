
package org.globalart.astrum.datastore.mybatis.handlers;

import java.util.Map;

import org.globalart.astrum.datastore.mybatis.AbstractJsonTypeHandler;

import org.apache.ibatis.type.TypeHandler;

/**
 * MyBatis {@link TypeHandler} that maps a (possibly nested) attribute map to/from JSON.
 *
 * Sensitive fields will be automatically encrypted at rest when persisting to the config store.
 *
 * @see org.globalart.astrum.datastore.mybatis.SensitiveAttributes
 *
 * @since 3.19
 */
// not @Component
public class AttributesTypeHandler
    extends AbstractJsonTypeHandler<Map<String, ?>>
{
  // nothing to do
}
