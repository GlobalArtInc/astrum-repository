
package org.globalart.astrum.coreui.internal.capability;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.capability.Capability;
import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.internal.scheduling.SchedulerCapabilityDescriptor;
import org.globalart.astrum.repository.capability.StorageSettingsCapabilityDescriptor;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import com.google.common.base.Predicate;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;

import static java.util.Collections.singletonList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests {@link CapabilityComponent} deletion functionality.
 */
@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class CapabilityComponentDeletionTest
{
  @Mock
  private CapabilityDescriptorRegistry capabilityDescriptorRegistry;

  @Mock
  private CapabilityRegistry capabilityRegistry;

  private CapabilityComponent underTest;

  @BeforeEach
  void setup() {
    underTest = new CapabilityComponent(capabilityDescriptorRegistry, capabilityRegistry);
  }

  @Test
  void shouldSetSystemTrueForSystemCapability() {
    // Given: A scheduler capability (system capability)
    CapabilityReference capabilityReference = mock(CapabilityReference.class);
    CapabilityContext capabilityContext = mock(CapabilityContext.class);
    CapabilityDescriptor capabilityDescriptor = mock(CapabilityDescriptor.class);
    CapabilityIdentity capabilityId = mock(CapabilityIdentity.class);
    Capability capability = mock(Capability.class);

    CapabilityType schedulerType = CapabilityType.capabilityType(SchedulerCapabilityDescriptor.TYPE_ID);
    when(capabilityDescriptor.type()).thenReturn(schedulerType);
    when(capabilityDescriptor.name()).thenReturn("Scheduler");
    when(capabilityContext.isEnabled()).thenReturn(true);
    when(capabilityContext.isActive()).thenReturn(true);
    when(capabilityContext.hasFailure()).thenReturn(false);
    when(capabilityContext.stateDescription()).thenReturn("Active");
    when(capabilityContext.notes()).thenReturn("System capability");
    when(capabilityContext.properties()).thenReturn(Map.of("testProperty", "testValue"));
    when(capabilityContext.descriptor()).thenReturn(capabilityDescriptor);
    when(capabilityContext.id()).thenReturn(capabilityId);
    when(capabilityId.toString()).thenReturn("scheduler-id");
    when(capabilityReference.capability()).thenReturn(capability);
    when(capabilityReference.context()).thenReturn(capabilityContext);
    when(capability.isPasswordProperty(any())).thenReturn(false);
    when(capabilityRegistry.get(any(Predicate.class))).thenReturn(singletonList(capabilityReference));

    // When: Reading capabilities
    List<CapabilityXO> results = underTest.read();

    // Then: Should have isSystem = true
    assertThat(results, hasSize(1));
    CapabilityXO result = results.get(0);
    assertThat(result.isSystem(), is(true));
    assertThat(result.getTypeName(), is("Scheduler"));
  }

  @Test
  void shouldSetSystemFalseForRegularCapability() {
    // Given: A regular capability (not system capability)
    CapabilityReference capabilityReference = mock(CapabilityReference.class);
    CapabilityContext capabilityContext = mock(CapabilityContext.class);
    CapabilityDescriptor capabilityDescriptor = mock(CapabilityDescriptor.class);
    CapabilityIdentity capabilityId = mock(CapabilityIdentity.class);
    Capability capability = mock(Capability.class);

    CapabilityType regularType = CapabilityType.capabilityType("webhook.repository");
    when(capabilityDescriptor.type()).thenReturn(regularType);
    when(capabilityDescriptor.name()).thenReturn("Repository Webhook");
    when(capabilityContext.isEnabled()).thenReturn(true);
    when(capabilityContext.isActive()).thenReturn(true);
    when(capabilityContext.hasFailure()).thenReturn(false);
    when(capabilityContext.stateDescription()).thenReturn("Active");
    when(capabilityContext.notes()).thenReturn("Regular capability");
    when(capabilityContext.properties()).thenReturn(Map.of("testProperty", "testValue"));
    when(capabilityContext.descriptor()).thenReturn(capabilityDescriptor);
    when(capabilityContext.id()).thenReturn(capabilityId);
    when(capabilityId.toString()).thenReturn("webhook-id");
    when(capabilityReference.capability()).thenReturn(capability);
    when(capabilityReference.context()).thenReturn(capabilityContext);
    when(capability.isPasswordProperty(any())).thenReturn(false);
    when(capabilityRegistry.get(any(Predicate.class))).thenReturn(singletonList(capabilityReference));

    // When: Reading capabilities
    List<CapabilityXO> results = underTest.read();

    // Then: Should have isSystem = false (default)
    assertThat(results, hasSize(1));
    CapabilityXO result = results.get(0);
    assertThat(result.isSystem(), is(false));
    assertThat(result.getTypeName(), is("Repository Webhook"));
  }

  @Test
  void shouldThrowExceptionWhenRemovingSystemCapability() {
    // Given: A scheduler capability (system capability)
    CapabilityReference capabilityReference = mock(CapabilityReference.class);
    CapabilityContext capabilityContext = mock(CapabilityContext.class);
    CapabilityDescriptor capabilityDescriptor = mock(CapabilityDescriptor.class);
    CapabilityIdentity capabilityId = mock(CapabilityIdentity.class);

    CapabilityType schedulerType = CapabilityType.capabilityType(SchedulerCapabilityDescriptor.TYPE_ID);
    when(capabilityDescriptor.type()).thenReturn(schedulerType);
    when(capabilityDescriptor.name()).thenReturn("Scheduler");
    when(capabilityContext.descriptor()).thenReturn(capabilityDescriptor);
    when(capabilityReference.context()).thenReturn(capabilityContext);
    when(capabilityRegistry.get(any(CapabilityIdentity.class))).thenReturn(capabilityReference);

    // When/Then: Attempting to remove should throw exception
    IllegalArgumentException exception = assertThrows(IllegalArgumentException.class,
        () -> underTest.remove("scheduler-id"));

    assertThat(exception.getMessage(), is("Cannot delete system capability: Scheduler"));

    // Verify registry.remove was never called
    verify(capabilityRegistry, times(0)).remove(any());
  }

  @Test
  void shouldSucceedWhenRemovingRegularCapability() {
    // Given: A regular capability (not system capability)
    CapabilityReference capabilityReference = mock(CapabilityReference.class);
    CapabilityContext capabilityContext = mock(CapabilityContext.class);
    CapabilityDescriptor capabilityDescriptor = mock(CapabilityDescriptor.class);

    CapabilityType regularType = CapabilityType.capabilityType("webhook.repository");
    when(capabilityDescriptor.type()).thenReturn(regularType);
    when(capabilityContext.descriptor()).thenReturn(capabilityDescriptor);
    when(capabilityReference.context()).thenReturn(capabilityContext);
    when(capabilityRegistry.get(any(CapabilityIdentity.class))).thenReturn(capabilityReference);

    // When: Removing regular capability
    underTest.remove("webhook-id");

    // Then: Should call registry remove
    verify(capabilityRegistry, times(1)).remove(any(CapabilityIdentity.class));
  }

  @Test
  void shouldSetSystemTrueForStorageSettingsCapability() {
    // Given: A storage settings capability (system capability)
    CapabilityReference capabilityReference = mock(CapabilityReference.class);
    CapabilityContext capabilityContext = mock(CapabilityContext.class);
    CapabilityDescriptor capabilityDescriptor = mock(CapabilityDescriptor.class);
    CapabilityIdentity capabilityId = mock(CapabilityIdentity.class);
    Capability capability = mock(Capability.class);

    CapabilityType storageType = CapabilityType.capabilityType(StorageSettingsCapabilityDescriptor.TYPE_ID);
    when(capabilityDescriptor.type()).thenReturn(storageType);
    when(capabilityDescriptor.name()).thenReturn("Storage Settings");
    when(capabilityContext.isEnabled()).thenReturn(true);
    when(capabilityContext.isActive()).thenReturn(true);
    when(capabilityContext.hasFailure()).thenReturn(false);
    when(capabilityContext.stateDescription()).thenReturn("Active");
    when(capabilityContext.notes()).thenReturn("System capability");
    when(capabilityContext.properties()).thenReturn(Map.of("testProperty", "testValue"));
    when(capabilityContext.descriptor()).thenReturn(capabilityDescriptor);
    when(capabilityContext.id()).thenReturn(capabilityId);
    when(capabilityId.toString()).thenReturn("storage-id");
    when(capabilityReference.capability()).thenReturn(capability);
    when(capabilityReference.context()).thenReturn(capabilityContext);
    when(capability.isPasswordProperty(any())).thenReturn(false);
    when(capabilityRegistry.get(any(Predicate.class))).thenReturn(singletonList(capabilityReference));

    // When: Reading capabilities
    List<CapabilityXO> results = underTest.read();

    // Then: Should have isSystem = true
    assertThat(results, hasSize(1));
    CapabilityXO result = results.get(0);
    assertThat(result.isSystem(), is(true));
    assertThat(result.getTypeName(), is("Storage Settings"));
  }
}
