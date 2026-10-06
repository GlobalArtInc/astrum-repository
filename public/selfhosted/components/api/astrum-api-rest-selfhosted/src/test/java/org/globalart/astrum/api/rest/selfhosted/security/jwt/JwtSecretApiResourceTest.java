
package org.globalart.astrum.api.rest.selfhosted.security.jwt;

import org.globalart.astrum.security.jwt.SecretStore;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentMatchers;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class JwtSecretApiResourceTest
{
  @Mock
  private SecretStore secretStore;

  @InjectMocks
  private JwtSecretApiResourceV1 underTest;

  @Test
  void resetSecret() {
    underTest.resetSecret();

    verify(secretStore).setSecret(ArgumentMatchers.any(String.class));
  }
}
