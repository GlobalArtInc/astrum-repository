
package org.globalart.astrum.datastore.mybatis;

import java.sql.CallableStatement;
import java.sql.PreparedStatement;
import java.sql.ResultSet;

import org.globalart.astrum.crypto.LegacyCipherFactory.PbeCipher;
import org.globalart.astrum.datastore.mybatis.handlers.PrincipalCollectionTypeHandler;

import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.junit.Before;
import org.junit.BeforeClass;
import org.junit.Test;
import org.mockito.Mock;

import static org.apache.commons.lang3.SerializationUtils.serialize;
import static org.hamcrest.core.Is.is;
import static org.junit.Assert.assertNull;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class PrincipalCollectionTypeHandlerTest
{
  private static final byte[] PRINCIPAL_COLLECTION_BYTES = {8, 5, 11, 110};

  private static final int AN_INDEX = 0;

  private static final String COLUMN = "COLUMN";

  private static SimplePrincipalCollection principalCollection;

  private static byte[] principalCollectionBytes;

  @Mock
  private ResultSet resultSet;

  @Mock
  private CallableStatement callableStatement;

  @Mock
  private PbeCipher databaseCipher;

  @Mock
  private PreparedStatement preparedStatement;

  private PrincipalCollectionTypeHandler principalCollectionTypeHandler;

  @BeforeClass
  public static void setupClass() {
    principalCollection = new SimplePrincipalCollection("p1", "r1");
    principalCollectionBytes = serialize(principalCollection);
  }

  @Before
  public void setup() {
    principalCollectionTypeHandler = new PrincipalCollectionTypeHandler();
    when(databaseCipher.decrypt(principalCollectionBytes)).thenReturn(principalCollectionBytes);
    ((CipherAwareTypeHandler<?>) principalCollectionTypeHandler).setCipher(databaseCipher);
  }

  @Test
  public void shouldSetEncryptedPrincipalCollectionBytes() throws Exception {
    when(databaseCipher.encrypt(any(byte[].class))).thenReturn(PRINCIPAL_COLLECTION_BYTES);

    principalCollectionTypeHandler.setNonNullParameter(preparedStatement,
        AN_INDEX,
        new SimplePrincipalCollection("p1", "r1"), null);

    verify(preparedStatement).setBytes(AN_INDEX, PRINCIPAL_COLLECTION_BYTES);
  }

  @Test
  public void shouldBeNullWhenResultSetReturnsNullForColumnIndex() throws Exception {

    final PrincipalCollection result = principalCollectionTypeHandler.getNullableResult(resultSet, AN_INDEX);

    assertNull(result);
  }

  @Test
  public void shouldBePrincipalCollectionWhenResultSetReturnsBytesForColumnIndex() throws Exception {
    when(resultSet.getBytes(AN_INDEX)).thenReturn(principalCollectionBytes);

    final PrincipalCollection result = principalCollectionTypeHandler.getNullableResult(resultSet, AN_INDEX);

    assertThat(result, is(principalCollection));
  }

  @Test
  public void shouldBeNullWhenResultSetReturnsNullForColumnName() throws Exception {

    final PrincipalCollection result = principalCollectionTypeHandler.getNullableResult(resultSet, COLUMN);

    assertNull(result);
  }

  @Test
  public void shouldBeNullWhenResultSetReturnsBytesForColumnName() throws Exception {
    when(resultSet.getBytes(COLUMN)).thenReturn(principalCollectionBytes);

    final PrincipalCollection result = principalCollectionTypeHandler.getNullableResult(resultSet, COLUMN);

    assertThat(result, is(principalCollection));
  }

  @Test
  public void shouldBeNullWhenCallableStatementReturnsNull() throws Exception {

    final PrincipalCollection result = principalCollectionTypeHandler.getNullableResult(callableStatement, AN_INDEX);

    assertNull(result);
  }

  @Test
  public void shouldBePrincipalCollectionWhenCallableStatementReturnsBytes() throws Exception {
    when(callableStatement.getBytes(AN_INDEX)).thenReturn(principalCollectionBytes);

    final PrincipalCollection result = principalCollectionTypeHandler.getNullableResult(callableStatement, AN_INDEX);

    assertThat(result, is(principalCollection));
  }
}
