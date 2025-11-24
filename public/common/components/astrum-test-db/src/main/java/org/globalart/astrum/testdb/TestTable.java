
package org.globalart.astrum.testdb;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@Target({ElementType.PARAMETER, ElementType.FIELD})
@Retention(RetentionPolicy.RUNTIME)
public @interface TestTable
{
  /**
   * The name of the table
   */
  String table();

  /**
   * The data store
   */
  String storeName() default DEFAULT_DATASTORE_NAME;
}
