
package org.globalart.astrum.kv;

import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.internal.kv.AstrumKeyValueDAO;
import org.globalart.astrum.transaction.Transactional;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.google.common.collect.Iterables;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * MyBatis astrum_key_value access interface
 */
@Component
@Qualifier("mybatis")
public class GlobalKeyValueStore
    extends ConfigStoreSupport<AstrumKeyValueDAO>
    implements KeyValueStore
{
  private final ObjectMapper mapper;

  @Autowired
  public GlobalKeyValueStore(final DataSessionSupplier sessionSupplier, final ObjectMapper mapper) {
    super(sessionSupplier, AstrumKeyValueDAO.class);
    this.mapper = checkNotNull(mapper);
  }

  /**
   * gets a value by the given key
   *
   * @param key a string key
   * @return {@link Optional<AstrumKeyValue>}
   */
  @Override
  @Transactional
  public Optional<AstrumKeyValue> getKey(final String key) {
    return dao().get(key);
  }

  // TODO don't mark this as Transactional it is expensive
  @Override
  public <E> Optional<E> get(final String key, final Class<E> clazz) {
    Optional<AstrumKeyValue> val = getKey(key);
    if (val.isEmpty()) {
      return Optional.empty();
    }

    return val.map(o -> o.getAsObject(mapper, clazz));
  }

  @Override
  @Transactional
  public Optional<Boolean> getBoolean(final String key) {
    return getKey(key)
        .map(AstrumKeyValue::getAsBoolean);
  }

  @Override
  @Transactional
  public Optional<String> getString(final String key) {
    return getKey(key)
        .map(AstrumKeyValue::getAsString);
  }

  @Override
  @Transactional
  public Optional<Integer> getInt(final String key) {
    return getKey(key)
        .map(AstrumKeyValue::getAsInt);
  }

  /**
   * sets a key_value record
   *
   * @param keyValue record to be created/updated
   */
  @Override
  @Transactional
  public void setKey(final AstrumKeyValue keyValue) {
    super.postCommitEvent(() -> new KeyValueEvent(keyValue.key(), Iterables.getOnlyElement(keyValue.value().values())));
    dao().set(keyValue);
  }

  @Override
  @Transactional
  public void setBoolean(final String key, final boolean value) {
    setKey(new AstrumKeyValue(key, ValueType.BOOLEAN, value));
  }

  @Override
  @Transactional
  public void setInt(final String key, final int value) {
    setKey(new AstrumKeyValue(key, ValueType.NUMBER, value));
  }

  @Override
  @Transactional
  public void setString(final String key, final String value) {
    setKey(new AstrumKeyValue(key, ValueType.CHARACTER, value));
  }

  @Override
  @Transactional
  public void setString(final String key, final Object value) {
    setKey(new AstrumKeyValue(key, ValueType.OBJECT, value));
  }

  /**
   * removes a value by the given key
   *
   * @param key a string key
   * @return a primitive boolean indicating if the record was deleted successfully or not
   */
  @Override
  @Transactional
  public boolean removeKey(final String key) {
    boolean removed = dao().remove(key);
    if (removed) {
      super.postCommitEvent(() -> new KeyValueEvent(key, null));
    }
    return removed;
  }
}
