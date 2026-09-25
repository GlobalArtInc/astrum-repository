
package com.globalart.astrum.ssl.plugin.configuration.model;

import java.util.List;

import org.globalart.astrum.configuration.model.ConfigurationXO;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Configuration transfer object for SSL/TLS certificate list.
 *
 * @since 3.91
 */
public class CertificateListConfigurationXO
    implements ConfigurationXO
{
  public static final String TYPE_ID = "certificate_list";

  @JsonProperty("certificateConfigurationsXOs")
  private List<String> certificateConfigurationsXOs;

  @JsonProperty("configurationTypeId")
  private String configurationTypeId = TYPE_ID;

  public CertificateListConfigurationXO() {
    // Jackson deserialization
  }

  public CertificateListConfigurationXO(final List<String> certificateConfigurationsXOs) {
    this.certificateConfigurationsXOs = certificateConfigurationsXOs;
  }

  public List<String> getCertificateConfigurationsXOs() {
    return certificateConfigurationsXOs;
  }

  public void setCertificateConfigurationsXOs(final List<String> certificateConfigurationsXOs) {
    this.certificateConfigurationsXOs = certificateConfigurationsXOs;
  }

  public String getConfigurationTypeId() {
    return configurationTypeId;
  }

  public void setConfigurationTypeId(final String configurationTypeId) {
    this.configurationTypeId = configurationTypeId;
  }
}
