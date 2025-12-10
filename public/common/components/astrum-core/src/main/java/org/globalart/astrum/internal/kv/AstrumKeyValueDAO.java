
package org.globalart.astrum.internal.kv;

import java.util.Optional;

import org.globalart.astrum.datastore.api.DataAccess;
import org.globalart.astrum.kv.AstrumKeyValue;

import org.apache.ibatis.annotations.Param;

/**
 * {@link AstrumKeyValue} access
 */
public interface AstrumKeyValueDAO
    extends DataAccess
{
  /**
   * gets a value by the given key
   *
   * @param key a string key
   * @return {@link Optional<AstrumKeyValue>}
   */
  Optional<AstrumKeyValue> get(
      @Param("key") final String key);

  /**
   * sets a key_value record
   *
   * @param record {@link AstrumKeyValue} to be created/updated
   */
  void set(final AstrumKeyValue record);

  /**
   * removes a value by the given key
   *
   * @param key a string key
   * @return a primitive boolean indicating if the record was deleted successfully or not
   */
  boolean remove(@Param("key") final String key);
}
