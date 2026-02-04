
package org.globalart.astrum.repository.content;

import org.apache.ibatis.annotations.Param;

public interface SqlGenerator<P extends SqlQueryParameters>
{
  String generateSelectStatement(@Param("params") final P params);
}
