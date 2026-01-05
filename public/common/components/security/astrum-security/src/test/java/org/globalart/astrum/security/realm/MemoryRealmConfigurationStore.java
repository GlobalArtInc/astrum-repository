
package org.globalart.astrum.security.realm;

import java.util.List;

import javax.annotation.Nullable;

import com.google.common.collect.Lists;
import org.springframework.beans.factory.annotation.Qualifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * In-memory {@link RealmConfigurationStore}.
 *
 * @since 3.0
 */
@Qualifier("memory")
public class MemoryRealmConfigurationStore
    implements RealmConfigurationStore
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private RealmConfiguration model;

  @Override
  public RealmConfiguration newEntity() {
    return new MemoryRealmConfiguration();
  }

  @Override
  @Nullable
  public synchronized RealmConfiguration load() {
    return model;
  }

  @Override
  public synchronized void save(final RealmConfiguration configuration) {
    this.model = checkNotNull(configuration);
  }

  /**
   * @since 3.20
   */
  private static class MemoryRealmConfiguration
      implements RealmConfiguration, Cloneable
  {

    private List<String> realmNames;

    MemoryRealmConfiguration() {
      // package private
    }

    @Override
    public List<String> getRealmNames() {
      return realmNames;
    }

    @Override
    public void setRealmNames(@Nullable final List<String> realmNames) {
      this.realmNames = realmNames;
    }

    @Override
    public MemoryRealmConfiguration copy() {
      try {
        MemoryRealmConfiguration copy = (MemoryRealmConfiguration) clone();
        if (realmNames != null) {
          copy.realmNames = Lists.newArrayList(realmNames);
        }
        return copy;
      }
      catch (CloneNotSupportedException e) {
        throw new RuntimeException(e);
      }
    }
  }
}
