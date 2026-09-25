
package com.globalart.astrum.ssl.plugin.configuration.model;

import java.util.List;

import org.globalart.astrum.configuration.model.ConfigurationXO;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Configuration transfer object for SSL/TLS trust store certificates.
 */
public class TrustStoreConfigurationXO
    implements ConfigurationXO
{
  public static final String TYPE_ID = "security.trust";

  @JsonProperty("certificates")
  private List<String> certificates;

  @JsonProperty("configurationTypeId")
  private String configurationTypeId = TYPE_ID;

  public TrustStoreConfigurationXO() {
    // Jackson deserialization
  }

  public TrustStoreConfigurationXO(final List<String> certificates) {
    this.certificates = certificates;
  }

  public List<String> getCertificates() {
    return certificates;
  }

  public void setCertificates(final List<String> certificates) {
    this.certificates = certificates;
  }

  public String getConfigurationTypeId() {
    return configurationTypeId;
  }
}
