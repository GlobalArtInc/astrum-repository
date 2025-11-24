
package org.globalart.astrum.testdb;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.type.TypeHandler;

import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@Target(ElementType.FIELD)
@Retention(RetentionPolicy.RUNTIME)
public @interface DataSessionConfiguration
{
  String storeName() default DEFAULT_DATASTORE_NAME;

  Class<? extends DataAccess>[] daos();

  Class<? extends TypeHandler<?>>[] typeHandlers() default {};

  /**
   * Can be used to prevent the H2 database from being used for all tests. Individual tests may also disable H2 via
   * the {@link DatabaseTest} annotation. Both this and the test method's {@link DatabaseTest#h2()} must be true for
   * the test to run with H2.
   */
  boolean h2() default true;

  /**
   * Can be used to enable the PostgreSQL database for all tests in this test class. Individual tests must also have
   * {@code @DatabaseTest(postgresql = true)} to actually run with PostgreSQL. Both this and the test method's
   * {@link DatabaseTest#postgresql()} must be true for the test to run with PostgreSQL.
   */
  boolean postgresql() default false;
}
