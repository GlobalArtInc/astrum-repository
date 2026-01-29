
package org.globalart.astrum.repository.config.internal;

import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.DetachingList;
import org.globalart.astrum.common.collect.DetachingMap;
import org.globalart.astrum.common.collect.DetachingSet;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.entity.HasEntityId;
import org.globalart.astrum.common.entity.HasName;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.supportzip.PasswordSanitizing;

import com.fasterxml.jackson.annotation.JsonIgnore;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link Configuration} data.
 *
 * @since 3.21
 */
public class ConfigurationData
    implements HasEntityId, HasName, Configuration
{
  private static final PasswordSanitizing<Map<String, Map<String, Object>>> SANITIZING = new PasswordSanitizing<>();

  // do not serialize EntityId, it can be generated on the fly. Is not used as reference.
  @JsonIgnore
  private EntityId id;

  private String name;

  private String recipeName;

  private boolean online;

  private EntityId routingRuleId;

  private volatile Map<String, Map<String, Object>> attributes;

  @Override
  public EntityId getId() {
    return id;
  }

  @Override
  public void setId(final EntityId id) {
    this.id = id;
  }

  @Override
  @JsonIgnore
  public EntityId getRepositoryId() {
    return getId(); // alias repositoryId to id
  }

  @Override
  public String getName() {
    return name;
  }

  @Override
  public void setName(final String name) {
    this.name = name;
  }

  @Override
  public String getRepositoryName() {
    return getName(); // alias repositoryName to name
  }

  @Override
  public void setRepositoryName(final String name) {
    setName(name); // alias repositoryName to name
  }

  @Override
  public String getRecipeName() {
    return recipeName;
  }

  @Override
  public void setRecipeName(final String recipeName) {
    this.recipeName = checkNotNull(recipeName);
  }

  /**
   * @return true, if repository should serve inbound requests
   */
  @Override
  public boolean isOnline() {
    return online;
  }

  /**
   * @param online true, if repository should serve inbound requests
   */
  @Override
  public void setOnline(final boolean online) {
    this.online = online;
  }

  @Override
  public EntityId getRoutingRuleId() {
    return routingRuleId;
  }

  @Override
  public void setRoutingRuleId(final EntityId routingRuleId) {
    this.routingRuleId = routingRuleId;
  }

  @Override
  @Nullable
  public Map<String, Map<String, Object>> getAttributes() {
    return attributes;
  }

  @Override
  public void setAttributes(@Nullable final Map<String, Map<String, Object>> attributes) {
    if (attributes != null && !(attributes instanceof ConcurrentHashMap)) {
      this.attributes = new ConcurrentHashMap<>(attributes);
    }
    else {
      this.attributes = attributes;
    }
  }

  @Override
  public synchronized NestedAttributesMap attributes(final String key) {
    checkNotNull(key);

    if (attributes == null) {
      attributes = new ConcurrentHashMap<>();
    }

    return new NestedAttributesMap(key, attributes.computeIfAbsent(key, k -> new ConcurrentHashMap<>()));
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "repositoryName='" + name + '\'' +
        ", recipeName='" + recipeName + '\'' +
        ", attributes=" + SANITIZING.transform(attributes) +
        '}';
  }

  /**
   * Returns a deeply cloned copy. Note that Entity.entityMetadata is not deep-copied.
   */
  @Override
  public ConfigurationData copy() {
    try {
      ConfigurationData c = (ConfigurationData) clone();
      c.id = null; // don't copy entity id
      c.attributes = copy(attributes);
      return c;
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  /**
   * Returns a lazy copy; for collections the copy is only taken as the content is touched.
   */
  @SuppressWarnings({"unchecked", "rawtypes"})
  private <V> V copy(final V value) {
    Object copy = value;
    if (value instanceof Map) {
      copy = new DetachingMap((Map) value, this::copy);
    }
    else if (value instanceof List) {
      copy = new DetachingList((List) value, this::copy);
    }
    else if (value instanceof Set) {
      copy = new DetachingSet((Set) value, this::copy);
    }
    return (V) copy;
  }

}
