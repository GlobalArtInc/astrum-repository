
package org.globalart.astrum.repository.helm.internal.metadata;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.helm.internal.util.JodaDateTimeDeserializer;
import org.globalart.astrum.repository.helm.internal.util.JodaDateTimeSerializer;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.fasterxml.jackson.databind.annotation.JsonSerialize;
import org.joda.time.DateTime;

/**
 * Object for storing attributes in a Helm index.yaml file
 *
 * @since 3.28
 */
public final class ChartIndex
{
  private String apiVersion;
  private Map<String, List<ChartEntry>> entries;
  @JsonSerialize(using = JodaDateTimeSerializer.class)
  @JsonDeserialize(using = JodaDateTimeDeserializer.class)
  private DateTime generated;

  public ChartIndex() {
    this.entries = new HashMap<>();
  }

  public String getApiVersion() {
    return this.apiVersion;
  }

  public void setApiVersion(final String apiVersion) {
    this.apiVersion = apiVersion;
  }

  public Map<String, List<ChartEntry>> getEntries() {
    return this.entries;
  }

  public void addEntry(final ChartEntry chartEntry) {
    this.entries.computeIfAbsent(chartEntry.getName(), k -> new ArrayList<>()).add(chartEntry);
  }

  public void setEntries(final Map<String, List<ChartEntry>> entries) {
    this.entries = entries;
  }

  public DateTime getGenerated() { return this.generated; }

  public void setGenerated(final DateTime generated) {
    this.generated = generated;
  }
}
