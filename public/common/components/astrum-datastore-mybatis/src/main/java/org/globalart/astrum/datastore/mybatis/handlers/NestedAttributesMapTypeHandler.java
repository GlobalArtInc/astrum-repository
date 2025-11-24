
package org.globalart.astrum.datastore.mybatis.handlers;

import java.lang.reflect.Type;
import java.sql.SQLException;
import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.datastore.mybatis.AbstractJsonTypeHandler;

import org.apache.ibatis.type.TypeHandler;

/**
 * MyBatis {@link TypeHandler} that maps a {@link NestedAttributesMap} to/from JSON.
 *
 * Sensitive fields will be automatically encrypted at rest when persisting to the config store.
 *
 * @see org.globalart.astrum.datastore.mybatis.SensitiveAttributes
 *
 * @since 3.20
 */
// not @Component
public class NestedAttributesMapTypeHandler
    extends AbstractJsonTypeHandler<NestedAttributesMap>
{
  @Override
  protected Type getJsonType() {
    return Map.class;
  }

  @Override
  protected byte[] writeToJson(final Object value) throws SQLException {
    return super.writeToJson(((NestedAttributesMap) value).backing());
  }

  @Override
  @SuppressWarnings("unchecked")
  protected Object readFromJson(final byte[] json) throws SQLException {
    return new NestedAttributesMap("attributes", (Map<String, Object>) super.readFromJson(json));
  }
}
