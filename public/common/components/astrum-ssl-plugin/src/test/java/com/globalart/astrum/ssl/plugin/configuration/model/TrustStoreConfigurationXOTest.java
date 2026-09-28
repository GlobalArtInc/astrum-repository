
package com.globalart.astrum.ssl.plugin.configuration.model;

import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

/**
 * Tests for {@link TrustStoreConfigurationXO}
 */
class TrustStoreConfigurationXOTest
{
  private static final String EXPECTED_TYPE_ID = "security.trust";

  @Test
  void shouldReturnTypeIdAndNullCertsForDefaultConstructor() {
    TrustStoreConfigurationXO xo = new TrustStoreConfigurationXO();

    assertThat(xo.getConfigurationTypeId(), is(EXPECTED_TYPE_ID));
    assertThat(xo.getCertificates(), is(nullValue()));
  }

  @Test
  void shouldReturnTypeIdAndCertsForParameterizedConstructor() {
    List<String> certs = List.of("cert1", "cert2", "cert3");

    TrustStoreConfigurationXO xo = new TrustStoreConfigurationXO(certs);

    assertThat(xo.getConfigurationTypeId(), is(EXPECTED_TYPE_ID));
    assertThat(xo.getCertificates(), is(certs));
  }

  @Test
  void shouldSetAndGetCertificates() {
    TrustStoreConfigurationXO xo = new TrustStoreConfigurationXO();

    List<String> certs = List.of("encrypted1", "encrypted2");
    xo.setCertificates(certs);

    assertThat(xo.getCertificates(), contains("encrypted1", "encrypted2"));
    assertThat(xo.getConfigurationTypeId(), is(EXPECTED_TYPE_ID));
  }

  @Test
  void shouldHaveCorrectTypeIdConstant() {
    assertThat(TrustStoreConfigurationXO.TYPE_ID, is(EXPECTED_TYPE_ID));
  }
}
