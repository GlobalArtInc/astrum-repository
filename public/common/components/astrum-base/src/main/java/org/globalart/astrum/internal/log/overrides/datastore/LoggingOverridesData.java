
package org.globalart.astrum.internal.log.overrides.datastore;

import org.globalart.astrum.common.entity.ContinuationAware;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Data object for the {@code logging_overrides} table.
 */
public class LoggingOverridesData
    implements ContinuationAware
{
  Integer id; // NOSONAR: internal id

  private String name;

  private String level;

  public LoggingOverridesData() {
  }

  public LoggingOverridesData(final String name, final String level) {
    this.name = checkNotNull(name);
    this.level = checkNotNull(level);
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getLevel() {
    return level;
  }

  public void setLevel(final String level) {
    this.level = level;
  }

  @Override
  public String nextContinuationToken() {
    return Integer.toString(id);
  }
}
