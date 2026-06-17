
package org.globalart.astrum.coreui.internal.sslcertificates;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class SslCertificatesStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    SslCertificatesStateContributor underTest = new SslCertificatesStateContributor(true);

    assertThat(underTest.getState().get("astrum.react.sslCertificates"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    SslCertificatesStateContributor underTest = new SslCertificatesStateContributor(false);

    assertThat(underTest.getState().get("astrum.react.sslCertificates"), is(false));
  }
}
