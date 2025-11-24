
package org.globalart.astrum.datastore.mybatis.handlers;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.Optional;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsFactory;

import org.apache.ibatis.type.BaseTypeHandler;
import org.apache.ibatis.type.JdbcType;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class SecretTypeHandler
    extends BaseTypeHandler<Secret>
{
  private final SecretsFactory factory;

  @Autowired
  public SecretTypeHandler(final SecretsFactory factory) {
    this.factory = checkNotNull(factory);
  }

  @Override
  public void setNonNullParameter(
      final PreparedStatement ps,
      final int i,
      final Secret parameter,
      final JdbcType jdbcType) throws SQLException
  {
    ps.setString(i, parameter.getId());
  }

  @Override
  public Secret getNullableResult(final ResultSet rs, final String columnName) throws SQLException {
    return from(rs.getString(columnName));
  }

  @Override
  public Secret getNullableResult(final ResultSet rs, final int columnIndex) throws SQLException {
    return from(rs.getString(columnIndex));
  }

  @Override
  public Secret getNullableResult(final CallableStatement cs, final int columnIndex) throws SQLException {
    return from(cs.getString(columnIndex));
  }

  @Nullable
  private Secret from(@Nullable final String secret) {
    return Optional.ofNullable(secret)
        .map(factory::from)
        .orElse(null);
  }
}
