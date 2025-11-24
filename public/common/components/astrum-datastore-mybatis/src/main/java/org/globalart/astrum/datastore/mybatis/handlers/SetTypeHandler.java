
package org.globalart.astrum.datastore.mybatis.handlers;

import java.util.Set;

import org.globalart.astrum.datastore.mybatis.AbstractRawJsonTypeHandler;

import org.apache.ibatis.type.TypeHandler;

/**
 * MyBatis {@link TypeHandler} that maps a set to/from JSON without any additional behaviour/processing.
 *
 * @since 3.21
 */
// not @Component
public class SetTypeHandler
    extends AbstractRawJsonTypeHandler<Set<?>>
{
  // nothing to add
}
