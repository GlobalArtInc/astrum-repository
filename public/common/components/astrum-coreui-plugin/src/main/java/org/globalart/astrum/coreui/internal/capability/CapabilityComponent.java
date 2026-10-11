
package org.globalart.astrum.coreui.internal.capability;

import java.util.Arrays;
import java.util.List;
import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;

import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.CapabilityReferenceFilter;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.validation.Validate;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.google.common.collect.ImmutableMap;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static java.util.stream.Collectors.toSet;
import static org.globalart.astrum.capability.CapabilityIdentity.capabilityIdentity;
import static org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.capabilities;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;
import static org.globalart.astrum.coreui.internal.capability.CapabilityConverter.asCapability;
import static org.globalart.astrum.coreui.internal.capability.CapabilityConverter.isSystemCapability;
import static org.globalart.astrum.coreui.internal.capability.CapabilityConverter.unfilterProperties;
import org.springframework.stereotype.Component;

// FIXME: update action name after refactor to use coreui_*

/**
 * Capabilities {@link DirectComponent}.
 */
@Component
@DirectAction(action = "capability_Capability")
public class CapabilityComponent
    extends DirectComponentSupport
    implements StateContributor
{
  private static final CapabilityReferenceFilter ALL_CREATED = capabilities().includeNotExposed();

  private static final CapabilityReferenceFilter ALL_ACTIVE = capabilities().includeNotExposed().active();

  private final CapabilityDescriptorRegistry capabilityDescriptorRegistry;

  private final CapabilityRegistry capabilityRegistry;

  @Autowired
  public CapabilityComponent(
      final CapabilityDescriptorRegistry capabilityDescriptorRegistry,
      final CapabilityRegistry capabilityRegistry)
  {
    this.capabilityDescriptorRegistry = checkNotNull(capabilityDescriptorRegistry);
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
  }

  /**
   * Retrieves capabilities.
   *
   * @return a list of capabilities
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:capabilities:read")
  public List<CapabilityXO> read() {
    capabilityRegistry.pullAndRefreshReferencesFromDB();
    return capabilityRegistry.get(capabilities()).stream().map(CapabilityConverter::asCapability).collect(toList());
  }

  /**
   * Retrieve available capabilities types.
   *
   * @return a list of capability types
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:capabilities:read")
  public List<CapabilityTypeXO> readTypes() {
    return Arrays.stream(capabilityDescriptorRegistry.getAll())
        .filter(CapabilityDescriptor::isExposed)
        .map(CapabilityConverter::asCapabilityType)
        .collect(toList());
  }

  /**
   * Creates a capability.
   *
   * @param capabilityXO to be created
   * @return created capability
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:create")
  @Validate(groups = {Create.class, Default.class})
  public CapabilityXO create(final @NotNull @Valid CapabilityXO capabilityXO) {
    return asCapability(capabilityRegistry.add(capabilityType(capabilityXO.getTypeId()), capabilityXO.getEnabled(),
        capabilityXO.getNotes(), capabilityXO.getProperties()));
  }

  /**
   * Updates a capability.
   *
   * @param capabilityXO to be updated
   * @return updated capability
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:update")
  @Validate(groups = {Update.class, Default.class})
  public CapabilityXO update(final @NotNull @Valid CapabilityXO capabilityXO) {
    CapabilityReference capabilityReference = capabilityRegistry.get(capabilityIdentity(capabilityXO.getId()));
    return asCapability(capabilityRegistry.update(capabilityIdentity(capabilityXO.getId()), capabilityXO.getEnabled(),
        capabilityXO.getNotes(),
        unfilterProperties(capabilityXO.getProperties(), capabilityReference.context().properties())));
  }

  /**
   * Updates capability notes.
   *
   * @param capabilityNotesXO to be updated
   * @return updated capability
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:update")
  @Validate(groups = {Update.class, Default.class})
  public CapabilityXO updateNotes(final @NotNull @Valid CapabilityNotesXO capabilityNotesXO) {
    CapabilityReference capabilityReference = capabilityRegistry.get(capabilityIdentity(capabilityNotesXO.getId()));
    return asCapability(
        capabilityRegistry.update(capabilityReference.context().id(), capabilityReference.context().isEnabled(),
            capabilityNotesXO.getNotes(), capabilityReference.context().properties()));
  }

  /**
   * Deletes a capability.
   *
   * @param id of capability to be deleted
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:delete")
  @Validate
  public void remove(final @NotEmpty String id) {
    CapabilityReference reference = capabilityRegistry.get(capabilityIdentity(id));
    if (reference != null && isSystemCapability(reference)) {
      throw new IllegalArgumentException("Cannot delete system capability: " +
          reference.context().descriptor().name());
    }
    capabilityRegistry.remove(capabilityIdentity(id));
  }

  /**
   * Enables an existing capability.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:update")
  @Validate
  public void enable(final @NotEmpty String id) {
    capabilityRegistry.enable(capabilityIdentity(id));
  }

  /**
   * Disables an existing capability.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:update")
  @Validate
  public void disable(final @NotEmpty String id) {
    capabilityRegistry.disable(capabilityIdentity(id));
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of("capabilityCreatedTypes",
        capabilityRegistry.get(ALL_CREATED).stream().map(this::capabilityToType).collect(toSet()),
        "capabilityActiveTypes",
        capabilityRegistry.get(ALL_ACTIVE).stream().map(this::capabilityToType).collect(toSet()));
  }

  private String capabilityToType(final CapabilityReference capabilityReference) {
    return capabilityReference.context().descriptor().type().toString();
  }
}
