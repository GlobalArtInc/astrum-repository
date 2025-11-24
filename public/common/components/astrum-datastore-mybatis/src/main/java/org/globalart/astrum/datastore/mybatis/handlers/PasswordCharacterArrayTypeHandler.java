
package org.globalart.astrum.datastore.mybatis.handlers;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;

import org.globalart.astrum.security.PasswordHelper;

import org.apache.ibatis.type.BaseTypeHandler;
import org.apache.ibatis.type.JdbcType;
import org.apache.ibatis.type.TypeHandler;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * MyBatis {@link TypeHandler} that treats character arrays as passwords and encrypts them before persisting.
 *
 * @since 3.21
 */
// not @Component
public class PasswordCharacterArrayTypeHandler
    extends BaseTypeHandler<char[]>
{
  private final PasswordHelper passwordHelper;

  public PasswordCharacterArrayTypeHandler(final PasswordHelper passwordHelper) {
    this.passwordHelper = checkNotNull(passwordHelper);
  }

  @Override
  public void setNonNullParameter(
      final PreparedStatement ps,
      final int parameterIndex,
      final char[] parameter,
      final JdbcType jdbcType) throws SQLException
  {
    ps.setString(parameterIndex, passwordHelper.encryptChars(parameter));
  }

  @Override
  public char[] getNullableResult(final ResultSet rs, final String columnName) throws SQLException {
    return passwordHelper.decryptChars(rs.getString(columnName));
  }

  @Override
  public char[] getNullableResult(final ResultSet rs, final int columnIndex) throws SQLException {
    return passwordHelper.decryptChars(rs.getString(columnIndex));
  }

  @Override
  public char[] getNullableResult(final CallableStatement cs, final int columnIndex) throws SQLException {
    return passwordHelper.decryptChars(cs.getString(columnIndex));
  }
}
