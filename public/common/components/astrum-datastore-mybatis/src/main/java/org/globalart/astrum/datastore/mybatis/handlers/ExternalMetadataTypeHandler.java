
package org.globalart.astrum.datastore.mybatis.handlers;

import java.lang.reflect.Type;

import org.globalart.astrum.blobstore.api.ExternalMetadata;
import org.globalart.astrum.datastore.mybatis.AbstractJsonTypeHandler;

import org.apache.ibatis.type.TypeHandler;

/**
 * MyBatis {@link TypeHandler} that maps a {@link ExternalMetadata} to/from JSON.
 */
// not @Component
public class ExternalMetadataTypeHandler
    extends AbstractJsonTypeHandler<ExternalMetadata>
{
  @Override
  protected Type getJsonType() {
    return ExternalMetadata.class;
  }
}
