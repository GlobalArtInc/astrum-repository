
package com.globalart.astrum.ssl.plugin.tasks;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.verify;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class TrustedCertificatesMigrationTaskTest
{
  @Mock
  private TrustedCertificateMigrationService trustedCertificateMigrationService;

  private TrustedCertificatesMigrationTask underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new TrustedCertificatesMigrationTask(trustedCertificateMigrationService);
  }

  @Test
  public void execute() throws Exception {
    underTest.execute();
    verify(trustedCertificateMigrationService).migrate();
  }

  @Test
  public void executesWhenExecuteTaskFails() throws Exception {
    doThrow(new RuntimeException("someError")).when(trustedCertificateMigrationService)
        .migrate();

    assertThrows(RuntimeException.class, () -> underTest.execute());

    verify(trustedCertificateMigrationService).migrate();
  }
}
