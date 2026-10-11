package org.globalart.astrum.coreui.internal.capability;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;
import jakarta.validation.ElementKind;
import jakarta.validation.Path;
import jakarta.validation.Path.Node;
import jakarta.validation.ValidationException;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.capability.Capability;
import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.coreui.FormFieldXO;
import org.globalart.astrum.formfields.ComboboxFormField;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.PasswordFormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.internal.scheduling.SchedulerCapabilityDescriptor;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.rest.ValidationErrorXO;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.google.common.base.Predicate;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static java.util.Collections.singletonList;
import static org.globalart.astrum.capability.CapabilityIdentity.capabilityIdentity;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyBoolean;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests {@link CapabilitiesApiResource}.
 */
@ExtendWith(MockitoExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class CapabilitiesApiResourceTest
{
  private static final String ID = "4f3c2b1a";

  private static final String SECRET = "its a secret to everybody";

  private static final CapabilityType WEBHOOK = capabilityType("webhook.global");

  @Mock
  private CapabilityDescriptorRegistry capabilityDescriptorRegistry;

  @Mock
  private CapabilityRegistry capabilityRegistry;

  @Mock
  private CapabilityReference reference;

  @Mock
  private CapabilityContext context;

  @Mock
  private CapabilityDescriptor descriptor;

  @Mock
  private Capability capability;

  @Captor
  private ArgumentCaptor<Map<String, String>> propertiesCaptor;

  private CapabilitiesApiResource underTest;

  @BeforeEach
  void setup() {
    underTest = new CapabilitiesApiResource(capabilityDescriptorRegistry, capabilityRegistry);

    lenient().when(descriptor.type()).thenReturn(WEBHOOK);
    lenient().when(descriptor.name()).thenReturn("Webhook: Global");
    lenient().when(descriptor.isExposed()).thenReturn(true);
    lenient().when(context.id()).thenReturn(capabilityIdentity(ID));
    lenient().when(context.descriptor()).thenReturn(descriptor);
    lenient().when(context.notes()).thenReturn("stored notes");
    lenient().when(context.isEnabled()).thenReturn(true);
    lenient().when(context.isActive()).thenReturn(true);
    lenient().when(context.stateDescription()).thenReturn("Active");
    lenient().when(context.properties()).thenReturn(Map.of("url", "https://hooks.example.com", "secret", SECRET));
    lenient().when(reference.context()).thenReturn(context);
    lenient().when(reference.capability()).thenReturn(capability);
    lenient().when(capability.isPasswordProperty(any())).thenAnswer(i -> "secret".equals(i.getArgument(0)));
    lenient().when(capability.description()).thenReturn("https://hooks.example.com");
    lenient().when(capability.status()).thenReturn("<p>Sending events</p>");
    lenient().when(capabilityRegistry.get(any(CapabilityIdentity.class))).thenAnswer(
        i -> capabilityIdentity(ID).equals(i.getArgument(0)) ? reference : null);
    lenient().when(capabilityRegistry.add(any(), anyBoolean(), any(), any())).thenReturn(reference);
    lenient().when(capabilityRegistry.update(any(), anyBoolean(), any(), any())).thenReturn(reference);
  }

  @Test
  void listsCapabilitiesWithStateAndMaskedSecrets() {
    when(capabilityRegistry.get(any(Predicate.class))).thenReturn(singletonList(reference));

    List<CapabilityXO> capabilities = underTest.getCapabilities();

    verify(capabilityRegistry).pullAndRefreshReferencesFromDB();
    assertThat(capabilities, hasSize(1));
    CapabilityXO result = capabilities.get(0);
    assertThat(result.getId(), is(ID));
    assertThat(result.getTypeId(), is("webhook.global"));
    assertThat(result.getTypeName(), is("Webhook: Global"));
    assertThat(result.getEnabled(), is(true));
    assertThat(result.getActive(), is(true));
    assertThat(result.getError(), is(false));
    assertThat(result.getState(), is("active"));
    assertThat(result.getNotes(), is("stored notes"));
    assertThat(result.getDescription(), is("https://hooks.example.com"));
    assertThat(result.getStatus(), is("<p>Sending events</p>"));
    assertThat(result.getProperties().get("url"), is("https://hooks.example.com"));
    assertThat(result.getProperties().get("secret"), is(PasswordPlaceholder.get()));
  }

  @Test
  void listsOnlyExposedTypesWithTheirFormFields() {
    CapabilityDescriptor hidden = mock(CapabilityDescriptor.class);
    when(hidden.isExposed()).thenReturn(false);
    List<FormField> fields = List.of(
        new StringTextFormField("url", "URL", "Where to send events", true, "^https?://.*"),
        new PasswordFormField("secret", "Secret Key", "Signs the payload", false),
        new ComboboxFormField<String>("names", "Event Types", "Events to send", true)
            .withStoreApi("coreui_Webhook.listWithTypeGlobal")
            .withIdMapping("id")
            .withNameMapping("name")
            .withStoreFilter("type", "global"));
    when(descriptor.about()).thenReturn("<p>Sends events to a URL</p>");
    when(descriptor.formFields()).thenReturn(fields);
    when(capabilityDescriptorRegistry.getAll()).thenReturn(new CapabilityDescriptor[]{descriptor, hidden});

    List<CapabilityTypeXO> types = underTest.getCapabilityTypes();

    assertThat(types, hasSize(1));
    CapabilityTypeXO type = types.get(0);
    assertThat(type.getId(), is("webhook.global"));
    assertThat(type.getName(), is("Webhook: Global"));
    assertThat(type.getAbout(), is("<p>Sends events to a URL</p>"));
    assertThat(type.getFormFields(), hasSize(3));
    FormFieldXO url = type.getFormFields().get(0);
    assertThat(url.getType(), is("string"));
    assertThat(url.getLabel(), is("URL"));
    assertThat(url.getHelpText(), is("Where to send events"));
    assertThat(url.getRequired(), is(true));
    assertThat(url.getRegexValidation(), is("^https?://.*"));
    assertThat(type.getFormFields().get(1).getType(), is("password"));
    FormFieldXO names = type.getFormFields().get(2);
    assertThat(names.getType(), is("combobox"));
    assertThat(names.getStoreApi(), is("coreui_Webhook.listWithTypeGlobal"));
    assertThat(names.getStoreFilters(), is(Map.of("type", "global")));
    assertThat(names.getIdMapping(), is("id"));
    assertThat(names.getNameMapping(), is("name"));
  }

  @Test
  void getsOneCapability() {
    CapabilityXO result = underTest.getCapability(ID);

    assertThat(result.getId(), is(ID));
    assertThat(result.getStatus(), is("<p>Sending events</p>"));
    assertThat(result.getProperties().get("secret"), is(PasswordPlaceholder.get()));
  }

  @Test
  void returnsNotFoundForAnUnknownId() {
    assertThrows(NotFoundException.class, () -> underTest.getCapability("missing"));
    assertThrows(NotFoundException.class, () -> underTest.updateCapability("missing", new CapabilityXO()));
    assertThrows(NotFoundException.class, () -> underTest.deleteCapability("missing"));
    verify(capabilityRegistry, never()).update(any(), anyBoolean(), any(), any());
    verify(capabilityRegistry, never()).remove(any());
  }

  @Test
  void returnsNotFoundForAHiddenCapability() {
    when(descriptor.isExposed()).thenReturn(false);

    assertThrows(NotFoundException.class, () -> underTest.getCapability(ID));
  }

  @Test
  void createsACapability() {
    when(capabilityDescriptorRegistry.get(WEBHOOK)).thenReturn(descriptor);
    CapabilityXO request = request("webhook.global", true, "new notes",
        Map.of("url", "https://hooks.example.com", "secret", SECRET));

    Response response = underTest.createCapability(request);

    assertThat(response.getStatus(), is(201));
    assertThat(((CapabilityXO) response.getEntity()).getId(), is(ID));
    assertThat(((CapabilityXO) response.getEntity()).getProperties().get("secret"), is(PasswordPlaceholder.get()));
    verify(capabilityRegistry).add(eq(WEBHOOK), eq(true), eq("new notes"), propertiesCaptor.capture());
    assertThat(propertiesCaptor.getValue().get("secret"), is(SECRET));
  }

  @Test
  void createsEnabledWhenEnabledIsLeftOut() {
    when(capabilityDescriptorRegistry.get(WEBHOOK)).thenReturn(descriptor);

    underTest.createCapability(request("webhook.global", null, null, Map.of()));

    verify(capabilityRegistry).add(eq(WEBHOOK), eq(true), eq(null), any());
  }

  @Test
  void rejectsAMissingOrUnknownType() {
    ValidationErrorsException missing =
        assertThrows(ValidationErrorsException.class, () -> underTest.createCapability(request(" ", true, null, null)));
    assertThat(missing.getValidationErrors().get(0).getId(), is("type"));

    ValidationErrorsException unknown = assertThrows(ValidationErrorsException.class,
        () -> underTest.createCapability(request("no.such.type", true, null, null)));
    assertThat(unknown.getValidationErrors().get(0).getId(), is("type"));
    assertThat(unknown.getValidationErrors().get(0).getMessage(), is("Unknown capability type: no.such.type"));
    verify(capabilityRegistry, never()).add(any(), anyBoolean(), any(), any());
  }

  @Test
  void reportsDescriptorValidationInTheRestErrorFormat() {
    when(capabilityDescriptorRegistry.get(WEBHOOK)).thenReturn(descriptor);
    ConstraintViolationException invalid = new ConstraintViolationException(Set.of(violation("url", "URL is not valid")));
    when(capabilityRegistry.add(any(), anyBoolean(), any(), any())).thenThrow(invalid);

    ValidationErrorsException e = assertThrows(ValidationErrorsException.class,
        () -> underTest.createCapability(request("webhook.global", true, null, Map.of("url", "nope"))));

    assertThat(e.getValidationErrors(), hasSize(1));
    assertThat(e.getValidationErrors().get(0).getId(), is("properties.url"));
    assertThat(e.getValidationErrors().get(0).getMessage(), is("URL is not valid"));
  }

  @Test
  void reportsUniquenessValidationAsAGeneralError() {
    when(capabilityRegistry.update(any(), anyBoolean(), any(), any()))
        .thenThrow(new ValidationException("Only one capability of type 'Webhook: Global' can be created"));

    ValidationErrorsException e = assertThrows(ValidationErrorsException.class,
        () -> underTest.updateCapability(ID, request(null, true, null, null)));

    assertThat(e.getValidationErrors().get(0).getId(), is(ValidationErrorXO.GENERIC));
    assertThat(e.getValidationErrors().get(0).getMessage(),
        is("Only one capability of type 'Webhook: Global' can be created"));
  }

  @Test
  void updatesACapabilityAndKeepsTheStoredSecretForThePlaceholder() {
    CapabilityXO request = request("webhook.global", true, "changed notes",
        Map.of("url", "https://other.example.com", "secret", PasswordPlaceholder.get()));
    request.setId(ID);

    CapabilityXO result = underTest.updateCapability(ID, request);

    verify(capabilityRegistry).update(eq(capabilityIdentity(ID)), eq(true), eq("changed notes"),
        propertiesCaptor.capture());
    assertThat(propertiesCaptor.getValue().get("url"), is("https://other.example.com"));
    assertThat(propertiesCaptor.getValue().get("secret"), is(SECRET));
    assertThat(result.getProperties().get("secret"), is(PasswordPlaceholder.get()));
  }

  @Test
  void updatesTheSecretWhenANewValueIsSent() {
    underTest.updateCapability(ID, request(null, true, null, Map.of("url", "https://hooks.example.com",
        "secret", "new secret")));

    verify(capabilityRegistry).update(any(), anyBoolean(), any(), propertiesCaptor.capture());
    assertThat(propertiesCaptor.getValue().get("secret"), is("new secret"));
  }

  @Test
  void disablesAndEnablesThroughUpdate() {
    Map<String, String> masked = Map.of("url", "https://hooks.example.com", "secret", PasswordPlaceholder.get());

    underTest.updateCapability(ID, request("webhook.global", false, "stored notes", masked));
    verify(capabilityRegistry).update(eq(capabilityIdentity(ID)), eq(false), eq("stored notes"),
        propertiesCaptor.capture());
    assertThat(propertiesCaptor.getValue().get("secret"), is(SECRET));

    underTest.updateCapability(ID, request("webhook.global", true, "stored notes", masked));
    verify(capabilityRegistry).update(eq(capabilityIdentity(ID)), eq(true), eq("stored notes"), any());
  }

  @Test
  void keepsCurrentValuesForFieldsLeftOut() {
    when(context.isEnabled()).thenReturn(false);
    CapabilityXO request = new CapabilityXO();

    underTest.updateCapability(ID, request);

    verify(capabilityRegistry).update(eq(capabilityIdentity(ID)), eq(false), eq("stored notes"),
        propertiesCaptor.capture());
    assertThat(propertiesCaptor.getValue().get("secret"), is(SECRET));
  }

  @Test
  void rejectsAChangeOfTypeOrId() {
    ValidationErrorsException type = assertThrows(ValidationErrorsException.class,
        () -> underTest.updateCapability(ID, request("baseurl", true, null, null)));
    assertThat(type.getValidationErrors().get(0).getId(), is("type"));

    CapabilityXO otherId = request(null, true, null, null);
    otherId.setId("another");
    ValidationErrorsException id =
        assertThrows(ValidationErrorsException.class, () -> underTest.updateCapability(ID, otherId));
    assertThat(id.getValidationErrors().get(0).getId(), is("id"));
    verify(capabilityRegistry, never()).update(any(), anyBoolean(), any(), any());
  }

  @Test
  void deletesACapability() {
    underTest.deleteCapability(ID);

    verify(capabilityRegistry).remove(capabilityIdentity(ID));
  }

  @Test
  void refusesToDeleteASystemCapability() {
    when(descriptor.type()).thenReturn(capabilityType(SchedulerCapabilityDescriptor.TYPE_ID));
    when(descriptor.name()).thenReturn("Scheduler");

    ValidationErrorsException e =
        assertThrows(ValidationErrorsException.class, () -> underTest.deleteCapability(ID));

    assertThat(e.getValidationErrors().get(0).getMessage(), is("Cannot delete system capability: Scheduler"));
    verify(capabilityRegistry, never()).remove(any());
  }

  @Test
  void serializesWithTheNamesTheUiReads() throws Exception {
    ObjectMapper mapper = new ObjectMapper();
    CapabilityXO xo = underTest.getCapability(ID);

    JsonNode json = mapper.valueToTree(xo);

    assertThat(json.get("type").asText(), is("webhook.global"));
    assertThat(json.get("isSystem").asBoolean(), is(false));
    assertThat(json.has("typeId"), is(false));
    assertThat(json.has("system"), is(false));
    assertThat(json.get("properties").get("secret").asText(), is(PasswordPlaceholder.get()));
    for (String field : List.of("id", "typeName", "enabled", "active", "error", "state", "stateDescription",
        "description", "status", "notes", "properties", "tags", "disableWarningMessage", "deleteWarningMessage")) {
      assertThat(field, json.has(field), is(true));
    }

    CapabilityXO parsed = mapper.readValue(
        "{\"id\":\"x\",\"type\":\"webhook.global\",\"enabled\":false,\"notes\":\"n\",\"properties\":{\"url\":\"u\"}}",
        CapabilityXO.class);
    assertThat(parsed.getTypeId(), is("webhook.global"));
    assertThat(parsed.getEnabled(), is(false));
    assertThat(parsed.getProperties(), is(Map.of("url", "u")));
    assertThat(parsed.getTags(), nullValue());
  }

  private static CapabilityXO request(
      final String type,
      final Boolean enabled,
      final String notes,
      final Map<String, String> properties)
  {
    CapabilityXO xo = new CapabilityXO();
    xo.setTypeId(type);
    xo.setEnabled(enabled);
    xo.setNotes(notes);
    xo.setProperties(properties == null ? null : new HashMap<>(properties));
    return xo;
  }

  private static ConstraintViolation<?> violation(final String property, final String message) {
    Node node = mock(Node.class);
    when(node.getKind()).thenReturn(ElementKind.PROPERTY);
    when(node.getName()).thenReturn(property);
    Path path = mock(Path.class);
    when(path.iterator()).thenAnswer(i -> List.of(node).iterator());
    ConstraintViolation<?> violation = mock(ConstraintViolation.class);
    when(violation.getPropertyPath()).thenReturn(path);
    when(violation.getMessage()).thenReturn(message);
    return violation;
  }
}
