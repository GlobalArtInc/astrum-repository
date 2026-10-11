package org.globalart.astrum.coreui.internal.capability;

import java.util.Arrays;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.StringJoiner;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;
import jakarta.validation.ElementKind;
import jakarta.validation.Path.Node;
import jakarta.validation.ValidationException;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.CREATED;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.capability.CapabilityIdentity.capabilityIdentity;
import static org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.capabilities;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;
import static org.globalart.astrum.coreui.internal.capability.CapabilityConverter.asCapability;
import static org.globalart.astrum.coreui.internal.capability.CapabilityConverter.isSystemCapability;
import static org.globalart.astrum.coreui.internal.capability.CapabilityConverter.unfilterProperties;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

/**
 * REST API for capabilities, backed by the same registries and conversion as the ExtDirect
 * {@link CapabilityComponent}.
 */
@Component
@Path(CapabilitiesApiResource.RESOURCE_URI)
@Produces(APPLICATION_JSON)
public class CapabilitiesApiResource
    implements Resource, CapabilitiesApiResourceDoc
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/capabilities";

  private final CapabilityDescriptorRegistry capabilityDescriptorRegistry;

  private final CapabilityRegistry capabilityRegistry;

  @Autowired
  public CapabilitiesApiResource(
      final CapabilityDescriptorRegistry capabilityDescriptorRegistry,
      final CapabilityRegistry capabilityRegistry)
  {
    this.capabilityDescriptorRegistry = checkNotNull(capabilityDescriptorRegistry);
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
  }

  @Override
  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:read")
  public List<CapabilityXO> getCapabilities() {
    capabilityRegistry.pullAndRefreshReferencesFromDB();
    return capabilityRegistry.get(capabilities()).stream().map(CapabilityConverter::asCapability).collect(toList());
  }

  @Override
  @GET
  @Path("/types")
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:read")
  public List<CapabilityTypeXO> getCapabilityTypes() {
    return Arrays.stream(capabilityDescriptorRegistry.getAll())
        .filter(CapabilityDescriptor::isExposed)
        .map(CapabilityConverter::asCapabilityType)
        .collect(toList());
  }

  @Override
  @GET
  @Path("/{id}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:read")
  public CapabilityXO getCapability(@PathParam("id") final String id) {
    return asCapability(getExposedCapability(id));
  }

  @Override
  @POST
  @Consumes(APPLICATION_JSON)
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:create")
  public Response createCapability(final CapabilityXO capability) {
    if (capability == null) {
      throw new ValidationErrorsException("A capability is required");
    }
    String typeId = capability.getTypeId();
    if (Strings2.isBlank(typeId)) {
      throw new ValidationErrorsException("type", "Capability type is required");
    }
    CapabilityDescriptor descriptor = capabilityDescriptorRegistry.get(capabilityType(typeId));
    if (descriptor == null || !descriptor.isExposed()) {
      throw new ValidationErrorsException("type", "Unknown capability type: " + typeId);
    }

    boolean enabled = capability.getEnabled() == null || capability.getEnabled();
    try {
      CapabilityReference created = capabilityRegistry.add(descriptor.type(), enabled, capability.getNotes(),
          capability.getProperties());
      return Response.status(CREATED).entity(asCapability(created)).build();
    }
    catch (ValidationException e) {
      throw asValidationErrors(e);
    }
  }

  @Override
  @PUT
  @Path("/{id}")
  @Consumes(APPLICATION_JSON)
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:update")
  public CapabilityXO updateCapability(@PathParam("id") final String id, final CapabilityXO capability) {
    CapabilityReference reference = getExposedCapability(id);
    if (capability == null) {
      throw new ValidationErrorsException("A capability is required");
    }
    if (capability.getId() != null && !Objects.equals(capability.getId(), id)) {
      throw new ValidationErrorsException("id", "Capability id does not match the request path");
    }
    String currentTypeId = reference.context().descriptor().type().toString();
    if (capability.getTypeId() != null && !Objects.equals(capability.getTypeId(), currentTypeId)) {
      throw new ValidationErrorsException("type", "Capability type cannot be changed");
    }

    Map<String, String> storedProperties = reference.context().properties();
    boolean enabled = capability.getEnabled() != null ? capability.getEnabled() : reference.context().isEnabled();
    String notes = capability.getNotes() != null ? capability.getNotes() : reference.context().notes();
    Map<String, String> properties = capability.getProperties() != null
        ? unfilterProperties(capability.getProperties(), storedProperties)
        : storedProperties;
    try {
      return asCapability(capabilityRegistry.update(reference.context().id(), enabled, notes, properties));
    }
    catch (ValidationException e) {
      throw asValidationErrors(e);
    }
  }

  @Override
  @DELETE
  @Path("/{id}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:capabilities:delete")
  public void deleteCapability(@PathParam("id") final String id) {
    CapabilityReference reference = getExposedCapability(id);
    if (isSystemCapability(reference)) {
      throw new ValidationErrorsException(
          "Cannot delete system capability: " + reference.context().descriptor().name());
    }
    capabilityRegistry.remove(reference.context().id());
  }

  private CapabilityReference getExposedCapability(final String id) {
    CapabilityReference reference = capabilityRegistry.get(capabilityIdentity(id));
    if (reference == null || !reference.context().descriptor().isExposed()) {
      throw new NotFoundException("Capability not found: " + id);
    }
    return reference;
  }

  /**
   * Turns the validation failures of a capability descriptor into the REST validation error format. Violations of a
   * capability configuration are reported against {@code properties.<name>}, others against {@code *}.
   */
  private static ValidationErrorsException asValidationErrors(final ValidationException e) {
    ValidationErrorsException errors = new ValidationErrorsException();
    if (e instanceof ConstraintViolationException cve && cve.getConstraintViolations() != null) {
      for (ConstraintViolation<?> violation : cve.getConstraintViolations()) {
        String path = propertyPath(violation);
        errors.withError(path.isEmpty() ? "*" : "properties." + path, violation.getMessage());
      }
    }
    if (!errors.hasValidationErrors()) {
      errors.withError(e.getMessage() != null ? e.getMessage() : "Invalid capability");
    }
    return errors;
  }

  private static String propertyPath(final ConstraintViolation<?> violation) {
    StringJoiner path = new StringJoiner(".");
    if (violation.getPropertyPath() != null) {
      Iterator<Node> nodes = violation.getPropertyPath().iterator();
      while (nodes.hasNext()) {
        Node node = nodes.next();
        if (node.getKind() == ElementKind.PROPERTY && node.getName() != null) {
          path.add(node.getName());
        }
      }
    }
    return path.toString();
  }
}
