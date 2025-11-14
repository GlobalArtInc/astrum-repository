
package org.globalart.astrum.kv;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class AstrumKeyValue
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String VALUE_NESTED_KEY = "value";

  private String key;

  private ValueType type;

  private Map<String, Object> value = new HashMap<>();

  public AstrumKeyValue(final String key, final ValueType type, final Object value) {
    this.key = key;
    this.type = type;
    setValue(value);
  }

  public AstrumKeyValue() {
  }

  public String key() {
    return key;
  }

  public void setKey(final String key) {
    this.key = key;
  }

  public ValueType type() {
    return type;
  }

  public void setType(final ValueType type) {
    this.type = type;
  }

  public Map<String, Object> value() {
    return value;
  }

  public void setValue(final Map<String, Object> value) {
    this.value = value;
  }

  public void setValue(final Object value) {
    this.value.put(VALUE_NESTED_KEY, value);
  }

  public String getAsString() {
    return value.get(VALUE_NESTED_KEY).toString();
  }

  public Integer getAsInt() {
    return Integer.parseInt(getAsString());
  }

  public Boolean getAsBoolean() {
    return Boolean.parseBoolean(getAsString());
  }

  public <T> T getAsObject(final ObjectMapper mapper, final Class<T> typeClass) {
    return mapper.convertValue(value.get(VALUE_NESTED_KEY), typeClass);
  }

  public <T> T getAsObject(final ObjectMapper mapper, final TypeReference<T> typeReference) {
    return mapper.convertValue(value.get(VALUE_NESTED_KEY), typeReference);
  }

  public <T> List<T> getAsObjectList(final ObjectMapper mapper, Class<T> typeClass) {
    return mapper.convertValue(value.get(VALUE_NESTED_KEY),
        mapper.getTypeFactory().constructCollectionType(List.class, typeClass));
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AstrumKeyValue that = (AstrumKeyValue) o;
    return Objects.equals(key, that.key) && type == that.type && Objects.equals(value, that.value);
  }

  @Override
  public int hashCode() {
    return Objects.hash(key, type, value);
  }
}
