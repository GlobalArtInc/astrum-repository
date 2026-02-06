
package org.globalart.astrum.repository.content.store;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import javax.annotation.Nullable;
import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.datastore.mybatis.handlers.ContentTypeHandler;

import org.apache.ibatis.type.BaseTypeHandler;
import org.apache.ibatis.type.JdbcType;
import org.springframework.stereotype.Component;

/**
 * MyBatis {@link ContentTypeHandler} that maps {@link BlobRef}s to/from SQL.
 *
 * @since 3.20
 */
@Component
public class BlobRefTypeHandler
    extends BaseTypeHandler<BlobRef>
    implements ContentTypeHandler<BlobRef>
{
  @Override
  public void setNonNullParameter(
      final PreparedStatement ps,
      final int parameterIndex,
      final BlobRef parameter,
      final JdbcType jdbcType) throws SQLException
  {
    ps.setString(parameterIndex, toPersistableString(parameter));
  }

  @Override
  public BlobRef getNullableResult(final ResultSet rs, final String columnName) throws SQLException {
    return nullableBlobRef(rs.getString(columnName));
  }

  @Override
  public BlobRef getNullableResult(final ResultSet rs, final int columnIndex) throws SQLException {
    return nullableBlobRef(rs.getString(columnIndex));
  }

  @Override
  public BlobRef getNullableResult(final CallableStatement cs, final int columnIndex) throws SQLException {
    return nullableBlobRef(cs.getString(columnIndex));
  }

  @Nullable
  private BlobRef nullableBlobRef(@Nullable final String blobRef) {
    return blobRef != null ? parsePersistableFormat(blobRef) : null;
  }

  /**
   * @return the {@link BlobRef} encoded as a string, using the syntax {@code store:blob-id@node}
   *
   * @since 3.26
   */
  public static String toPersistableString(final BlobRef blobRef) {
    return blobRef.toString();
  }

  /**
   * Parse a string representation of a {@link BlobRef}, using the syntax {@code store:blob-id@node}
   *
   * @since 3.26
   */
  public static BlobRef parsePersistableFormat(final String spec) {
    return BlobRef.parse(spec);
  }
}
