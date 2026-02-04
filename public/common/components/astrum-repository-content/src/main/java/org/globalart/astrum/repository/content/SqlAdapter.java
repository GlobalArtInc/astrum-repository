
package org.globalart.astrum.repository.content;

import org.apache.ibatis.annotations.Param;

public class SqlAdapter
{
  public String select(
      final SqlGenerator<SqlQueryParameters> generator,
      @Param("params") final SqlQueryParameters params)
  {
    return generator.generateSelectStatement(params);
  }
}
