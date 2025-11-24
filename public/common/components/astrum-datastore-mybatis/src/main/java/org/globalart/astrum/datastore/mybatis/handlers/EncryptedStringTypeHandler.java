
package org.globalart.astrum.datastore.mybatis.handlers;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;

import org.globalart.astrum.datastore.mybatis.CipherAwareTypeHandler;

import com.fasterxml.jackson.core.Base64Variant;
import com.fasterxml.jackson.core.Base64Variants;
import org.apache.ibatis.type.JdbcType;
import org.apache.ibatis.type.TypeHandler;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * MyBatis {@link TypeHandler} that encrypts strings with the property that the same source string
 * will always be encrypted to the same database string, assuming the same database cipher settings.
 * To use this handler you must explicitly refer to it in the appropriate parameter/result mappings.
 *
 * This handler should only be used if you need to search on the encrypted string, otherwise prefer
 * using a character array as that has automatic encryption-at-rest along with some other benefits.
 *
 * @see PasswordCharacterArrayTypeHandler
 *
 * @since 3.21
 */
// not @Component
public class EncryptedStringTypeHandler
    extends CipherAwareTypeHandler<String>
{
  private static final Base64Variant BASE_64 = Base64Variants.getDefaultVariant();

  @Override
  public final void setNonNullParameter(
      final PreparedStatement ps,
      final int parameterIndex,
      final String parameter,
      final JdbcType jdbcType) throws SQLException
  {
    ps.setString(parameterIndex, encrypt(parameter));
  }

  @Override
  public final String getNullableResult(final ResultSet rs, final String columnName) throws SQLException {
    return decrypt(rs.getString(columnName));
  }

  @Override
  public final String getNullableResult(final ResultSet rs, final int columnIndex) throws SQLException {
    return decrypt(rs.getString(columnIndex));
  }

  @Override
  public final String getNullableResult(final CallableStatement cs, final int columnIndex) throws SQLException {
    return decrypt(cs.getString(columnIndex));
  }

  /**
   * Encrypt string using database cipher + Base64.
   */
  private String encrypt(final String value) {
    return BASE_64.encode(cipher().encrypt(value.getBytes(UTF_8)));
  }

  /**
   * Decrypt string using Base64 + database cipher.
   */
  private String decrypt(final String value) {
    return value != null ? new String(cipher().decrypt(BASE_64.decode(value)), UTF_8) : null;
  }
}
