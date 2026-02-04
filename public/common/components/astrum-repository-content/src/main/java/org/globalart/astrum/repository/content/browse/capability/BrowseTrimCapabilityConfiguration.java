
package org.globalart.astrum.repository.content.browse.capability;

import java.util.Map;

import org.globalart.astrum.common.text.Strings2;

import static com.google.common.base.Preconditions.checkNotNull;

public class BrowseTrimCapabilityConfiguration
{
  public static final String POSTGRESQL_TRIM_ENABLED = "postgresqlTrimEnabled";

  public static final String BATCH_TRIM_ENABLED = "batchTrimEnabled";

  private static final boolean DEFAULT_POSTGRESQL_TRIM_ENABLED = false;

  private static final boolean DEFAULT_BATCH_TRIM_ENABLED = false;

  private boolean postgresqlTrimEnabled;

  private boolean batchTrimEnabled;

  public BrowseTrimCapabilityConfiguration() {
    this(Map.of());
  }

  public BrowseTrimCapabilityConfiguration(final Map<String, String> properties) {
    checkNotNull(properties);
    this.postgresqlTrimEnabled = parseBoolean(
        properties.get(POSTGRESQL_TRIM_ENABLED),
        DEFAULT_POSTGRESQL_TRIM_ENABLED);
    this.batchTrimEnabled = parseBoolean(
        properties.get(BATCH_TRIM_ENABLED),
        DEFAULT_BATCH_TRIM_ENABLED);
  }

  public boolean isPostgresqlTrimEnabled() {
    return postgresqlTrimEnabled;
  }

  public void setPostgresqlTrimEnabled(final boolean postgresqlTrimEnabled) {
    this.postgresqlTrimEnabled = postgresqlTrimEnabled;
  }

  public boolean isBatchTrimEnabled() {
    return batchTrimEnabled;
  }

  public void setBatchTrimEnabled(final boolean batchTrimEnabled) {
    this.batchTrimEnabled = batchTrimEnabled;
  }

  public Map<String, String> asMap() {
    return Map.of(
        POSTGRESQL_TRIM_ENABLED, Boolean.toString(postgresqlTrimEnabled),
        BATCH_TRIM_ENABLED, Boolean.toString(batchTrimEnabled));
  }

  private boolean parseBoolean(final String value, final boolean defaultValue) {
    if (!isEmpty(value)) {
      return Boolean.parseBoolean(value);
    }
    return defaultValue;
  }

  private boolean isEmpty(final String value) {
    return Strings2.isEmpty(value);
  }

  @Override
  public String toString() {
    return getClass().getSimpleName()
        + "{"
        + "postgresqlTrimEnabled=" + postgresqlTrimEnabled
        + ", batchTrimEnabled=" + batchTrimEnabled
        + "}";
  }
}
