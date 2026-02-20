
package org.globalart.astrum.repository.rest.internal;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadManager;
import org.globalart.astrum.swagger.ParameterContributor;

import com.google.common.collect.ImmutableList;
// ASTRUM-46395 spike: OpenAPI 3.x DROPPED FormParameter as a Parameter type. In OpenAPI 3,
// form/multipart inputs are modeled inside a RequestBody's Content.Schema, not as Parameter
// objects. A faithful migration of this contributor needs to switch from extending
// ParameterContributor to a different mechanism that mutates the Operation's RequestBody.
//
// Out of scope for the D1 spike: stubbed below using a generic Parameter so the surrounding
// code compiles and the Phase 3 migration team has a clear marker.
import io.swagger.v3.oas.models.PathItem.HttpMethod;
import io.swagger.v3.oas.models.parameters.Parameter;

import static io.swagger.v3.oas.models.PathItem.HttpMethod.POST;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * @since 3.8
 */
@Component
public class ComponentUploadParameterContributor
    extends ParameterContributor<Parameter>
{
  private static final List<HttpMethod> HTTP_METHODS = ImmutableList.of(POST);

  private static final List<String> PATHS = ImmutableList.of(V1_API_PREFIX + "/components");

  @Autowired
  public ComponentUploadParameterContributor(final UploadManager uploadManager) {
    super(HTTP_METHODS, PATHS, transformUploadDefinitions(uploadManager.getAvailableDefinitions()));
  }

  /**
   * ASTRUM-46395 SPIKE STUB: returns an empty parameter collection. The proper OpenAPI 3.x
   * design for form data is to mutate the Operation's RequestBody's Content schema rather
   * than push parameters via ParameterContributor. Real Phase 3 work item.
   */
  private static Collection<Parameter> transformUploadDefinitions(
      final Collection<UploadDefinition> uploadDefinitions)
  {
    return new ArrayList<>();
  }
}
