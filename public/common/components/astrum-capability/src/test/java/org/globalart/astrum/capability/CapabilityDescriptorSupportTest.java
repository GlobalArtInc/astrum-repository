
package org.globalart.astrum.capability;

import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Set;
import javax.annotation.Nullable;
import jakarta.validation.ValidationException;

import org.globalart.astrum.capability.CapabilityDescriptor.ValidationMode;
import org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.CapabilityReferenceFilter;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.RepositoryCombobox;

import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.allOf;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CapabilityDescriptorSupportTest
{
  @Mock
  private CapabilityRegistry capabilityRegistry;

  @Mock
  private CapabilityReference capabilityReference;

  @Mock
  private CapabilityContext capabilityContext;

  private CapabilityIdentity capabilityIdentity;

  @Captor
  private ArgumentCaptor<CapabilityReferenceFilter> filterRecorder;

  @Before
  public void prepare() {
    capabilityIdentity = CapabilityIdentity.capabilityIdentity("test");
    when(capabilityContext.id()).thenReturn(capabilityIdentity);
    when(capabilityReference.context()).thenReturn(capabilityContext);
  }

  @Test
  public void capabilityWithSameTypeDoesNotExist() {
    when(capabilityRegistry.get(filterRecorder.capture())).thenReturn(Collections.emptyList());
    TestCapabilityDescriptor underTest = new TestCapabilityDescriptor(Collections.emptyList(), Collections.emptySet());
    underTest.installComponents(() -> capabilityRegistry);

    underTest.validate(null, Collections.emptyMap(), ValidationMode.CREATE);
    assertThat(filterRecorder.getValue().getTypeId(), is("test"));
    assertThat(filterRecorder.getValue().getIgnoreCapabilityId(), is(nullValue()));
  }

  @Test
  public void capabilityWithSameTypeAlreadyPresent() {
    when(capabilityRegistry.get(filterRecorder.capture())).thenAnswer(
        invocation -> Collections.singletonList(capabilityReference));
    TestCapabilityDescriptor underTest = new TestCapabilityDescriptor(Collections.emptyList(), Collections.emptySet());
    underTest.installComponents(() -> capabilityRegistry);

    ValidationException expected = assertThrows(ValidationException.class,
        () -> underTest.validate(null, Collections.emptyMap(), ValidationMode.CREATE));

    assertThat(expected.getMessage(), containsString("Test"));
    assertThat(filterRecorder.getValue().getTypeId(), is("test"));
    assertThat(filterRecorder.getValue().getIgnoreCapabilityId(), is(nullValue()));
  }

  @Test
  public void sameCapabilityWithSameTypeAlreadyPresent() {
    when(capabilityRegistry.get(filterRecorder.capture())).thenReturn(Collections.emptyList());
    TestCapabilityDescriptor underTest = new TestCapabilityDescriptor(Collections.emptyList(), Collections.emptySet());
    underTest.installComponents(() -> capabilityRegistry);

    underTest.validate(capabilityIdentity, Collections.emptyMap(), ValidationMode.UPDATE);
    assertThat(filterRecorder.getValue().getTypeId(), is("test"));
    assertThat(filterRecorder.getValue().getIgnoreCapabilityId(), is(capabilityIdentity));
  }

  @Test
  public void capabilityWithSameTypeAndSameRepositoryDoesNotExist() {
    when(capabilityRegistry.get(filterRecorder.capture())).thenReturn(Collections.emptyList());
    TestCapabilityDescriptor underTest =
        new TestCapabilityDescriptor(Collections.singletonList(new RepositoryCombobox("repository")),
            Collections.singleton("repository"));

    underTest.installComponents(() -> capabilityRegistry);

    underTest.validate(null, Collections.singletonMap("repository", "foo"), ValidationMode.CREATE);
    assertThat(filterRecorder.getValue().getTypeId(), is("test"));
    assertThat(filterRecorder.getValue().getIgnoreCapabilityId(), is(nullValue()));
    assertThat(filterRecorder.getValue().getProperties().get("repository"), is("foo"));
  }

  @Test
  public void capabilityWithSameTypeAndSameRepositoryAlreadyPresent() {
    when(capabilityRegistry.get(filterRecorder.capture())).thenAnswer(
        invocation -> Collections.singletonList(capabilityReference));
    TestCapabilityDescriptor underTest =
        new TestCapabilityDescriptor(Collections.singletonList(new RepositoryCombobox("repository")),
            Collections.singleton("repository"));

    underTest.installComponents(() -> capabilityRegistry);

    ValidationException expected = assertThrows(ValidationException.class,
        () -> underTest.validate(null, Collections.singletonMap("repository", "foo"), ValidationMode.CREATE));

    assertThat(expected.getMessage(),
        allOf(containsString("Test"), containsString("repository"), containsString("foo")));

    assertThat(filterRecorder.getValue().getTypeId(), is("test"));
    assertThat(filterRecorder.getValue().getIgnoreCapabilityId(), is(nullValue()));
    assertThat(filterRecorder.getValue().getProperties().get("repository"), is("foo"));
  }

  @Test
  public void sameCapabilityWithSameTypeAndSameRepositoryAlreadyPresent() {
    when(capabilityRegistry.get(filterRecorder.capture())).thenReturn(Collections.emptyList());
    TestCapabilityDescriptor underTest =
        new TestCapabilityDescriptor(Collections.singletonList(new RepositoryCombobox("repository")),
            Collections.singleton("repository"));

    underTest.installComponents(() -> capabilityRegistry);

    underTest.validate(capabilityIdentity, Collections.singletonMap("repository", "foo"), ValidationMode.UPDATE);
    assertThat(filterRecorder.getValue().getTypeId(), is("test"));
    assertThat(filterRecorder.getValue().getIgnoreCapabilityId(), is(capabilityIdentity));
    assertThat(filterRecorder.getValue().getProperties().get("repository"), is("foo"));
  }

  private static class TestCapabilityDescriptor
      extends CapabilityDescriptorSupport
  {
    private final List<FormField> formFields;

    private final Set<String> uniqueProperties;

    public TestCapabilityDescriptor(final List<FormField> formFields, final Set<String> uniqueProperties) {
      this.formFields = formFields;
      this.uniqueProperties = uniqueProperties;
    }

    @Override
    public CapabilityType type() {
      return CapabilityType.capabilityType("test");
    }

    @Override
    public String name() {
      return "Test";
    }

    @Override
    public List<FormField> formFields() {
      return formFields;
    }

    @Nullable
    @Override
    protected Set<String> uniqueProperties() {
      return uniqueProperties;
    }

    @Override
    protected Object createConfig(final Map properties) {
      return null;
    }
  }
}
