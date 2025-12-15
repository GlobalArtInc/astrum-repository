
package org.globalart.astrum.internal.security.apikey.store;

import org.globalart.astrum.crypto.secrets.Secret;

import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ApiKeyV2DataTest
{
  @Mock
  private Secret secret;

  private final ApiKeyV2Data underTest = new ApiKeyV2Data();

  @Test
  public void testGetApiKey() {
    when(secret.decrypt()).thenReturn("password".toCharArray());
    underTest.setSecret(secret);
    underTest.setAccessKey("key");

    assertThat(underTest.getApiKey(), is("keypassword".toCharArray()));
  }
}
