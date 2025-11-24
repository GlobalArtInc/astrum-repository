
package org.globalart.astrum.datastore.mybatis.handlers;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Timestamp;

import javax.annotation.Nullable;

import org.apache.ibatis.type.BaseTypeHandler;
import org.apache.ibatis.type.JdbcType;
import org.apache.ibatis.type.TypeHandler;
import org.joda.time.DateTime;

import static org.joda.time.DateTimeZone.UTC;

/**
 * MyBatis {@link TypeHandler} that maps Joda {@link DateTime} values to/from SQL.
 *
 * @since 3.20
 */
// not @Component
public class DateTimeTypeHandler
    extends BaseTypeHandler<DateTime>
{
  @Override
  public void setNonNullParameter(
      final PreparedStatement ps,
      final int parameterIndex,
      final DateTime parameter,
      final JdbcType jdbcType) throws SQLException
  {
    ps.setTimestamp(parameterIndex, new Timestamp(parameter.getMillis()));
  }

  @Override
  public DateTime getNullableResult(final ResultSet rs, final String columnName) throws SQLException {
    return nullableDateTime(rs.getTimestamp(columnName));
  }

  @Override
  public DateTime getNullableResult(final ResultSet rs, final int columnIndex) throws SQLException {
    return nullableDateTime(rs.getTimestamp(columnIndex));
  }

  @Override
  public DateTime getNullableResult(final CallableStatement cs, final int columnIndex) throws SQLException {
    return nullableDateTime(cs.getTimestamp(columnIndex));
  }

  @Nullable
  private DateTime nullableDateTime(@Nullable final Timestamp timestamp) {
    return timestamp != null ? new DateTime(timestamp.getTime(), UTC) : null;
  }
}
