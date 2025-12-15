
package org.globalart.astrum.internal.capability.secrets.migration;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.formfields.PasswordFormField;
import org.globalart.astrum.scheduling.CancelableHelper;
import org.globalart.astrum.scheduling.TaskInterruptedException;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import com.google.common.collect.ImmutableList;

import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doReturn;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CapabilitySecretsMigratorTest
{
  @Mock
  private CapabilityRegistry capabilityRegistry;

  private MockedStatic<CancelableHelper> cancelableHelperMock;

  private CapabilitySecretsMigrator underTest;

  private int canceledStateCheckCount = 0;

  @Before
  public void setUp() {
    underTest = new CapabilitySecretsMigrator(capabilityRegistry);
    cancelableHelperMock = mockStatic(CancelableHelper.class);
  }

  @Test
  public void testMigrationWorksAsExpected() {
    List<CapabilityReference> capabilityReferences = mockCapabilities(10, 5);
    doReturn(capabilityReferences).when(capabilityRegistry).getAll();

    underTest.migrate();

    verify(capabilityRegistry, times(5)).migrateSecrets(any(CapabilityReference.class), any());
  }

  @Test
  public void testMigrationStopsIfCancelled() {
    simulateTaskInterrupted(6);
    List<CapabilityReference> capabilityReferences = mockCapabilities(20, 10);
    doReturn(capabilityReferences).when(capabilityRegistry).getAll();

    TaskInterruptedException expected = assertThrows(TaskInterruptedException.class, underTest::migrate);

    assertNotNull(expected);
    verify(capabilityRegistry, times(6)).migrateSecrets(any(CapabilityReference.class), any());
  }

  private void simulateTaskInterrupted(final int targetCheck) {
    cancelableHelperMock.when(CancelableHelper::checkCancellation)
        .thenAnswer(i -> {
          canceledStateCheckCount++;
          // force failure after we checked targetCheck times
          if (canceledStateCheckCount > targetCheck) {
            throw new TaskInterruptedException("expected", true);
          }

          return false;
        });
  }

  @After
  public void tearDown() {
    cancelableHelperMock.close();
    canceledStateCheckCount = 0;
  }

  private List<CapabilityReference> mockCapabilities(final int size, final int encryptedFieldsCapabilities) {
    List<CapabilityReference> references = new ArrayList<>();

    for (int i = 0; i < size; i++) {
      CapabilityReference reference;
      if (i < encryptedFieldsCapabilities) {
        reference = mockCapabilityReference(true);
      }
      else {
        reference = mockCapabilityReference(false);
      }
      references.add(reference);
    }

    return references;
  }

  private CapabilityReference mockCapabilityReference(boolean hasEncryptedField) {
    CapabilityReference reference = mock(CapabilityReference.class);
    CapabilityContext context = mock(CapabilityContext.class);
    CapabilityDescriptor descriptor = mock(CapabilityDescriptor.class);

    when(reference.context()).thenReturn(context);
    when(context.descriptor()).thenReturn(descriptor);

    if (hasEncryptedField) {
      when(descriptor.formFields()).thenReturn(
          ImmutableList.of(new PasswordFormField("test", "test", "?", true)));
    }
    else {
      when(descriptor.formFields()).thenReturn(ImmutableList.of());
    }

    return reference;
  }
}
