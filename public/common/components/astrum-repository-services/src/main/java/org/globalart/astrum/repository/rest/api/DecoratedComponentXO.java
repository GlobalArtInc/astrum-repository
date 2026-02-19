
package org.globalart.astrum.repository.rest.api;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

import org.globalart.astrum.common.decorator.DecoratedObject;

import com.fasterxml.jackson.annotation.JsonIgnore;

public abstract class DecoratedComponentXO
    implements ComponentXO, DecoratedObject<ComponentXO>
{
  protected String id;

  protected final ComponentXO componentXO;

  protected DecoratedComponentXO(ComponentXO componentXO) {
    this.componentXO = componentXO;
  }

  @Override
  public String getId() {
    return id;
  }

  @Override
  public void setId(String id) {
    this.id = id;
  }

  @Override
  public String getGroup() {
    return componentXO.getGroup();
  }

  @Override
  public void setGroup(String group) {
    componentXO.setGroup(group);
  }

  @Override
  public String getName() {
    return componentXO.getName();
  }

  @Override
  public void setName(String name) {
    componentXO.setName(name);
  }

  @Override
  public String getVersion() {
    return componentXO.getVersion();
  }

  @Override
  public void setVersion(String version) {
    componentXO.setVersion(version);
  }

  @Override
  public String getRepository() {
    return componentXO.getRepository();
  }

  @Override
  public void setRepository(String repository) {
    componentXO.setRepository(repository);
  }

  @Override
  public String getFormat() {
    return componentXO.getFormat();
  }

  @Override
  public void setFormat(String format) {
    componentXO.setFormat(format);
  }

  @Override
  public List<AssetXO> getAssets() {
    return componentXO.getAssets();
  }

  @Override
  public void setAssets(List<AssetXO> assets) {
    componentXO.setAssets(assets);
  }

  @Override
  @JsonIgnore
  public ComponentXO getWrappedObject() {
    return componentXO;
  }

  @JsonIgnore
  public abstract Map<String, Object> getDecoratedExtraJsonAttributes();

  @Override
  public final Map<String, Object> getExtraJsonAttributes() {
    Map<String, Object> extraJsonAttributes = new HashMap<>(componentXO.getExtraJsonAttributes());
    extraJsonAttributes.putAll(getDecoratedExtraJsonAttributes());
    return extraJsonAttributes;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    DecoratedComponentXO that = (DecoratedComponentXO) o;
    return Objects.equals(id, that.id);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id);
  }

  @Override
  public String toString() {
    return "DecoratedComponentXO{" +
        "id='" + id + '\'' +
        ", componentXO=" + componentXO +
        '}';
  }
}
