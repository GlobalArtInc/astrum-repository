
package org.globalart.astrum.datastore.mybatis.handlers;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;

import org.apache.ibatis.type.BaseTypeHandler;
import org.apache.ibatis.type.JdbcType;

/**
 * Type handler for use with {@code ::tsvector}<br/>
 *
 * Not @Component to avoid injection
 */
public class QuotingTypeHandler
    extends BaseTypeHandler<String>
{
  private static final String QUOTE = "'";

  @Override
  public void setNonNullParameter(
      final PreparedStatement ps,
      final int parameterIndex,
      final String parameter,
      final JdbcType jdbcType) throws SQLException
  {
    ps.setString(parameterIndex, tsEscape(parameter));
  }

  @Override
  public String getNullableResult(final ResultSet rs, final String columnName) throws SQLException {
    throw new UnsupportedOperationException();
  }

  @Override
  public String getNullableResult(final ResultSet rs, final int columnIndex) throws SQLException {
    throw new UnsupportedOperationException();
  }

  @Override
  public String getNullableResult(final CallableStatement cs, final int columnIndex) throws SQLException {
    throw new UnsupportedOperationException();
  }

  public static String tsEscape(final String term) {
    return QUOTE + term.replace("\\", "\\\\").replace(QUOTE, "\\'") + QUOTE;
  }
}
