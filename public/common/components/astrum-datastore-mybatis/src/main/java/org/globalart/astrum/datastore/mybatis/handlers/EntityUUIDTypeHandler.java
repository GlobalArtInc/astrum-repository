
package org.globalart.astrum.datastore.mybatis.handlers;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.UUID;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.entity.EntityUUID;

import org.apache.ibatis.type.BaseTypeHandler;
import org.apache.ibatis.type.JdbcType;
import org.apache.ibatis.type.TypeHandler;

import static java.util.UUID.fromString;

/**
 * MyBatis {@link TypeHandler} that maps UUID-backed {@link EntityId}s to/from SQL.
 *
 * @since 3.19
 */
// not @Component
public class EntityUUIDTypeHandler
    extends BaseTypeHandler<EntityId>
{
  private final boolean lenient;

  public EntityUUIDTypeHandler(final boolean lenient) {
    this.lenient = lenient; // pass UUIDs by string when we're being lenient
  }

  @Override
  public void setNonNullParameter(
      final PreparedStatement ps,
      final int parameterIndex,
      final EntityId parameter,
      final JdbcType jdbcType) throws SQLException
  {
    UUID uuid;
    if (parameter instanceof EntityUUID) {
      uuid = ((EntityUUID) parameter).uuid();
    }
    else {
      uuid = UUID.fromString(parameter.getValue());
    }

    if (lenient) {
      ps.setString(parameterIndex, uuid.toString());
    }
    else {
      ps.setObject(parameterIndex, uuid);
    }
  }

  @Override
  public EntityUUID getNullableResult(final ResultSet rs, final String columnName) throws SQLException {
    return nullableEntityUUID(rs.getObject(columnName));
  }

  @Override
  public EntityUUID getNullableResult(final ResultSet rs, final int columnIndex) throws SQLException {
    return nullableEntityUUID(rs.getObject(columnIndex));
  }

  @Override
  public EntityUUID getNullableResult(final CallableStatement cs, final int columnIndex) throws SQLException {
    return nullableEntityUUID(cs.getObject(columnIndex));
  }

  @Nullable
  private EntityUUID nullableEntityUUID(@Nullable final Object uuid) {
    // expect UUIDs but also accept strings
    if (uuid instanceof UUID) {
      return new EntityUUID((UUID) uuid);
    }
    if (uuid instanceof String) {
      return new EntityUUID(fromString((String) uuid));
    }
    return null;
  }
}
