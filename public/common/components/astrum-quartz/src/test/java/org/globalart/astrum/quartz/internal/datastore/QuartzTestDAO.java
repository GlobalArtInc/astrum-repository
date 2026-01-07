
package org.globalart.astrum.quartz.internal.datastore;

import java.util.List;

import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

public interface QuartzTestDAO
    extends DataAccess
{
  List<String> tables();

  List<String> primaryKeys();

  List<String> foreignKeys();

  List<String> indexes();

  Integer expectedIndexes();

  /**
   * Get column type from information schema
   *
   * @param tableName table that column belongs to
   * @param columnName column witch type needs to be received
   * @return return the type of specified column
   */
  String getTableColumnType(@Param("table_name") String tableName, @Param("column_name") String columnName);

  String getColumnCharacterLimit(@Param("table_name") String tableName, @Param("column_name") String columnName);
}
