
package org.globalart.astrum.datastore.mybatis.handlers;

import java.util.List;

import org.globalart.astrum.datastore.mybatis.AbstractRawJsonTypeHandler;

import org.apache.ibatis.type.TypeHandler;

/**
 * MyBatis {@link TypeHandler} that maps a list to/from JSON without any additional behaviour/processing.
 *
 * @since 3.21
 */
// not @Component
public class ListTypeHandler
    extends AbstractRawJsonTypeHandler<List<?>>
{
  // nothing to add
}
