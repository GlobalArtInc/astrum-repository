
package org.globalart.astrum.capability;

import java.util.Map;
import java.util.Map.Entry;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.formfields.Selectable;

import static com.google.common.base.Preconditions.checkNotNull;

public class CapabilityDTO
{
  public static final String PASSWORD_PLACEHOLDER = "#~ASTRUM~PLACEHOLDER~PASSWORD~#";

  private String id;

  private String type;

  private String typeName;

  private String notes;

  private boolean enabled;

  private boolean active;

  private boolean error;

  private String state;

  private String stateDescription;

  private String description;

  private String status;

  private Map<String, String> properties;

  private Map<String, String> tags;

  public CapabilityDTO() {
    // deserialization and tests
  }

  public CapabilityDTO(final CapabilityReference reference) {
    checkNotNull(reference);
    CapabilityContext context = checkNotNull(reference.context());
    Capability capability = checkNotNull(reference.capability());
    CapabilityDescriptor descriptor = checkNotNull(context.descriptor());

    id = context.id().toString();
    type = context.type().toString();
    typeName = descriptor.name();
    enabled = context.isEnabled();
    active = context.isActive();
    error = context.hasFailure();
    stateDescription = context.stateDescription();
    description = capability.description();
    status = capability.status();
    notes = context.notes();
    properties = filterProperties(context.properties(), capability);

    // Compute state
    if (!enabled) {
      state = "disabled";
    }
    else if (error) {
      state = "error";
    }
    else if (!active) {
      state = "passive";
    }
    else {
      state = "active";
    }

    // Populate tags
    if (descriptor instanceof Taggable) {
      tags = ((Taggable) descriptor).getTags()
          .stream()
          .collect(Collectors.toMap(Tag::key, Tag::value));
    }
    else if (capability instanceof Taggable) {
      tags = ((Taggable) capability).getTags()
          .stream()
          .collect(Collectors.toMap(Tag::key, Tag::value));
    }
  }

  public String getId() {
    return id;
  }

  public String getNotes() {
    return notes;
  }

  public Map<String, String> getProperties() {
    return properties;
  }

  public String getType() {
    return type;
  }

  public String getTypeName() {
    return typeName;
  }

  public void setTypeName(final String typeName) {
    this.typeName = typeName;
  }

  public boolean isEnabled() {
    return enabled;
  }

  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  public boolean isActive() {
    return active;
  }

  public void setActive(final boolean active) {
    this.active = active;
  }

  public boolean isError() {
    return error;
  }

  public void setError(final boolean error) {
    this.error = error;
  }

  public String getState() {
    return state;
  }

  public void setState(final String state) {
    this.state = state;
  }

  public String getStateDescription() {
    return stateDescription;
  }

  public void setStateDescription(final String stateDescription) {
    this.stateDescription = stateDescription;
  }

  public String getDescription() {
    return description;
  }

  public void setDescription(final String description) {
    this.description = description;
  }

  public String getStatus() {
    return status;
  }

  public void setStatus(final String status) {
    this.status = status;
  }

  public Map<String, String> getTags() {
    return tags;
  }

  public void setTags(final Map<String, String> tags) {
    this.tags = tags;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public void setNotes(final String notes) {
    this.notes = notes;
  }

  public void setProperties(final Map<String, String> properties) {
    this.properties = properties;
  }

  public void setType(final String type) {
    this.type = type;
  }

  public static CapabilityTypeDTO fromCapabilityDescriptor(final CapabilityDescriptor capabilityDescriptor) {
    CapabilityTypeDTO dto = new CapabilityTypeDTO();
    dto.setId(capabilityDescriptor.type().toString());
    dto.setName(capabilityDescriptor.name());
    dto.setAbout(capabilityDescriptor.about());
    if (capabilityDescriptor.formFields() != null) {
      dto.setFormFields(
          capabilityDescriptor.formFields()
              .stream()
              .map(CapabilityDTO::toFormFieldDTO)
              .toList());
    }
    return dto;
  }

  private static FormFieldDTO toFormFieldDTO(final FormField<?> source) {
    FormFieldDTO dto = new FormFieldDTO();
    dto.setId(source.getId());
    dto.setType(source.getType());
    dto.setLabel(source.getLabel());
    dto.setHelpText(source.getHelpText());
    dto.setRequired(source.isRequired());
    dto.setDisabled(source.isDisabled());
    dto.setReadOnly(source.isReadOnly());
    dto.setRegexValidation(source.getRegexValidation());
    dto.setInitialValue(Optional.ofNullable(source.getInitialValue()).map(Objects::toString).orElse(null));
    dto.setAttributes(source.getAttributes());

    if (source instanceof NumberTextFormField ntf) {
      dto.setMinimumValue(Optional.ofNullable(ntf.getMinimumValue()).map(Object::toString).orElse(null));
      dto.setMaximumValue(Optional.ofNullable(ntf.getMaximumValue()).map(Object::toString).orElse(null));
    }

    if (source instanceof Selectable selectable) {
      dto.setStoreApi(selectable.getStoreApi());
      dto.setStoreFilters(selectable.getStoreFilters());
      dto.setAllowAutocomplete(source.getAllowAutocomplete());
      dto.setIdMapping(selectable.getIdMapping());
      dto.setNameMapping(selectable.getNameMapping());
    }

    return dto;
  }

  private static Map<String, String> filterProperties(
      final Map<String, String> properties,
      final Capability capability)
  {
    return properties.entrySet()
        .stream()
        .collect(Collectors.toMap(Entry::getKey, entry -> {
          if (capability.isPasswordProperty(entry.getKey())) {
            if ("PKI".equals(properties.get("authenticationType"))) {
              return "";
            }
            else {
              return PASSWORD_PLACEHOLDER;
            }
          }
          return entry.getValue() != null ? entry.getValue() : ""; // ensure no null values
        }));
  }

  @Override
  public String toString() {
    return "CapabilityDTO(" +
        "id:'" + id + '\'' +
        ", type:'" + type + '\'' +
        ", typeName:'" + typeName + '\'' +
        ", notes:'" + notes + '\'' +
        ", enabled:" + enabled +
        ", active:" + active +
        ", error:" + error +
        ", state:'" + state + '\'' +
        ", stateDescription:'" + stateDescription + '\'' +
        ", description:'" + description + '\'' +
        ", status:'" + status + '\'' +
        ", properties:" + properties +
        ", tags:" + tags +
        ')';
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    CapabilityDTO that = (CapabilityDTO) o;
    return enabled == that.enabled && active == that.active && error == that.error &&
        Objects.equals(id, that.id) && Objects.equals(type, that.type) &&
        Objects.equals(typeName, that.typeName) && Objects.equals(notes, that.notes) &&
        Objects.equals(state, that.state) && Objects.equals(stateDescription, that.stateDescription) &&
        Objects.equals(description, that.description) && Objects.equals(status, that.status) &&
        Objects.equals(properties, that.properties) && Objects.equals(tags, that.tags);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, type, typeName, notes, enabled, active, error, state, stateDescription, description,
        status, properties, tags);
  }
}
