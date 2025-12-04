
package org.globalart.astrum.email.internal.secrets.migration;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.email.EmailManager;

import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class EmailSecretsMigratorTests
{
  @Mock
  private EmailManager emailManager;

  @Mock
  private EmailConfiguration emailConfiguration;

  @InjectMocks
  private EmailSecretsMigrator underTest;

  @Test
  public void testMigrate() {
    Secret notMigrated = getMockSecret("legacy", "legacyPassword");
    when(emailConfiguration.getPassword()).thenReturn(notMigrated);
    when(emailManager.getConfiguration()).thenReturn(emailConfiguration);

    underTest.migrate();
    verify(emailManager).setConfiguration(emailConfiguration, "legacyPassword");
  }

  @Test
  public void testSkipMigration() {
    Secret alreadyMigrated = getMockSecret("_1", "alreadyMigrated");
    when(emailConfiguration.getPassword()).thenReturn(alreadyMigrated);
    when(emailManager.getConfiguration()).thenReturn(emailConfiguration);

    underTest.migrate();
    verify(emailManager, never()).setConfiguration(any(EmailConfiguration.class), anyString());

    when(emailConfiguration.getPassword()).thenReturn(null);
    underTest.migrate();
    verify(emailManager, never()).setConfiguration(any(EmailConfiguration.class), anyString());
  }

  private Secret getMockSecret(String tokenId, String token) {
    Secret secret = mock(Secret.class);
    when(secret.getId()).thenReturn(tokenId);
    when(secret.decrypt()).thenReturn(token.toCharArray());
    return secret;
  }
}
