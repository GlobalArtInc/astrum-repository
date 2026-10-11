package org.globalart.astrum.coreui.internal.capability;

import java.util.AbstractMap.SimpleEntry;
import java.util.HashSet;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Set;

import org.globalart.astrum.capability.Capability;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Tag;
import org.globalart.astrum.capability.Taggable;
import org.globalart.astrum.coreui.FormFieldXO;
import org.globalart.astrum.internal.scheduling.SchedulerCapabilityDescriptor;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.repository.capability.StorageSettingsCapabilityDescriptor;

import static java.util.stream.Collectors.toList;
import static java.util.stream.Collectors.toMap;

/**
 * Converts capability references and descriptors to the exchange objects shared by the ExtDirect component and the
 * REST API, so both expose the same state, masking and system flags.
 */
final class CapabilityConverter
{
  private static final Set<String> SYSTEM_CAPABILITY_TYPES = Set.of(
      SchedulerCapabilityDescriptor.TYPE_ID,
      StorageSettingsCapabilityDescriptor.TYPE_ID);

  private CapabilityConverter() {
    // static utility
  }

  static CapabilityXO asCapability(final CapabilityReference reference) {
    CapabilityDescriptor descriptor = reference.context().descriptor();
    Capability capability = reference.capability();

    CapabilityXO capabilityXO = new CapabilityXO();
    capabilityXO.setId(reference.context().id().toString());
    capabilityXO.setNotes(reference.context().notes());
    capabilityXO.setTypeId(descriptor.type().toString());
    capabilityXO.setTypeName(descriptor.name());
    capabilityXO.setEnabled(reference.context().isEnabled());
    capabilityXO.setActive(reference.context().isActive());
    capabilityXO.setError(reference.context().hasFailure());
    capabilityXO.setState("disabled");
    capabilityXO.setStateDescription(reference.context().stateDescription());
    capabilityXO.setProperties(filterProperties(reference.context().properties(), capability));
    capabilityXO.setDisableWarningMessage(descriptor.getDisableWarningMessage());
    capabilityXO.setDeleteWarningMessage(descriptor.getDeleteWarningMessage());
    capabilityXO.setSystem(isSystemCapability(reference));

    if (capabilityXO.getEnabled() && capabilityXO.getError()) {
      capabilityXO.setState("error");
    }
    else if (capabilityXO.getEnabled() && capabilityXO.getActive()) {
      capabilityXO.setState("active");
    }
    else if (capabilityXO.getEnabled() && !capabilityXO.getActive()) {
      capabilityXO.setState("passive");
    }

    if (capability.description() != null) {
      capabilityXO.setDescription(capability.description());
    }

    if (capability.status() != null) {
      capabilityXO.setStatus(capability.status());
    }

    Set<Tag> tags = new HashSet<>();
    if (descriptor instanceof Taggable && ((Taggable) descriptor).getTags() != null) {
      tags.addAll(((Taggable) descriptor).getTags());
    }
    if (capability instanceof Taggable && ((Taggable) capability).getTags() != null) {
      tags.addAll(((Taggable) capability).getTags());
    }
    if (!tags.isEmpty()) {
      capabilityXO.setTags(tags.stream().collect(toMap(Tag::key, Tag::value)));
    }

    return capabilityXO;
  }

  static CapabilityTypeXO asCapabilityType(final CapabilityDescriptor capabilityDescriptor) {
    CapabilityTypeXO capabilityTypeXO = new CapabilityTypeXO();
    capabilityTypeXO.setId(capabilityDescriptor.type().toString());
    capabilityTypeXO.setName(capabilityDescriptor.name());
    capabilityTypeXO.setAbout(capabilityDescriptor.about());
    if (capabilityDescriptor.formFields() != null) {
      capabilityTypeXO.setFormFields(
          capabilityDescriptor.formFields().stream().map(FormFieldXO::create).collect(toList()));
    }
    return capabilityTypeXO;
  }

  /**
   * Replaces secret values with {@link PasswordPlaceholder} so they never leave the server.
   */
  static Map<String, String> filterProperties(final Map<String, String> properties, final Capability capability) {
    return properties.entrySet()
        .stream()
        .filter(CapabilityConverter::nonNullKeyAndValue)
        .map(entry -> {
          if (capability.isPasswordProperty(entry.getKey())) {
            if ("PKI".equals(properties.get("authenticationType"))) {
              return new SimpleEntry<>(entry.getKey(), "");
            }
            return new SimpleEntry<>(entry.getKey(), PasswordPlaceholder.get());
          }
          return entry;
        })
        .collect(toMap(Entry::getKey, Entry::getValue));
  }

  /**
   * Restores the stored value of every property that comes back as {@link PasswordPlaceholder}.
   */
  static Map<String, String> unfilterProperties(
      final Map<String, String> properties,
      final Map<String, String> referenceProperties)
  {
    return properties.entrySet()
        .stream()
        .filter(CapabilityConverter::nonNullKeyAndValue)
        .map(entry -> {
          if (PasswordPlaceholder.is(entry.getValue())) {
            return new SimpleEntry<>(entry.getKey(), referenceProperties.get(entry.getKey()));
          }
          return entry;
        })
        .collect(toMap(Entry::getKey, Entry::getValue));
  }

  static boolean isSystemCapability(final CapabilityReference reference) {
    CapabilityType type = reference.context().descriptor().type();
    return SYSTEM_CAPABILITY_TYPES.contains(type.toString());
  }

  private static boolean nonNullKeyAndValue(final Entry<?, ?> entry) {
    return entry != null && entry.getKey() != null && entry.getValue() != null;
  }
}
