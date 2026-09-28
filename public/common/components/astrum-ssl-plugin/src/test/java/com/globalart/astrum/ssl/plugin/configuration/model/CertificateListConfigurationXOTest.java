
package com.globalart.astrum.ssl.plugin.configuration.model;

import java.util.List;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

/**
 * Tests for {@link CertificateListConfigurationXO}
 */
public class CertificateListConfigurationXOTest
{
  @Test
  public void testDefaultConstructor() {
    CertificateListConfigurationXO xo = new CertificateListConfigurationXO();

    assertThat(xo.getConfigurationTypeId(), is("certificate_list"));
    assertThat(xo.getCertificateConfigurationsXOs(), is(nullValue()));
  }

  @Test
  public void testConstructorWithCertificates() {
    List<String> certs = List.of("cert1", "cert2", "cert3");

    CertificateListConfigurationXO xo = new CertificateListConfigurationXO(certs);

    assertThat(xo.getConfigurationTypeId(), is("certificate_list"));
    assertThat(xo.getCertificateConfigurationsXOs(), is(certs));
  }

  @Test
  public void testSettersAndGetters() {
    CertificateListConfigurationXO xo = new CertificateListConfigurationXO();

    List<String> certs = List.of("encrypted1", "encrypted2");
    xo.setCertificateConfigurationsXOs(certs);

    assertThat(xo.getCertificateConfigurationsXOs(), contains("encrypted1", "encrypted2"));
    assertThat(xo.getConfigurationTypeId(), is("certificate_list"));
  }

  @Test
  public void testTypeIdConstant() {
    assertThat(CertificateListConfigurationXO.TYPE_ID, is("certificate_list"));
  }
}
