
package org.globalart.astrum.repository.rest.internal;

import java.util.Collection;
import java.util.List;
import java.util.stream.StreamSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappingsService;
import org.globalart.astrum.repository.rest.internal.resources.SearchResource;
import org.globalart.astrum.swagger.ParameterContributor;

import com.google.common.collect.ImmutableList;
// ASTRUM-46395: migrated from Swagger 1.x parameters to OpenAPI 3.x.
// QueryParameter still exists in OpenAPI 3 (different package); the schema/type is now
// modeled via Parameter.schema(new StringSchema()) instead of Parameter.type("string").
import io.swagger.v3.oas.models.PathItem.HttpMethod;
import io.swagger.v3.oas.models.media.StringSchema;
import io.swagger.v3.oas.models.parameters.QueryParameter;

import static io.swagger.v3.oas.models.PathItem.HttpMethod.GET;
import static java.util.stream.Collectors.toList;
import org.springframework.stereotype.Component;

/**
 * @since 3.7
 */
@Component
public class SearchParameterContributor
    extends ParameterContributor<QueryParameter>
{
  private static final List<HttpMethod> HTTP_METHODS = ImmutableList.of(GET);

  private static final List<String> PATHS = ImmutableList.of(
      SearchResource.RESOURCE_URI,
      SearchResource.RESOURCE_URI + SearchResource.SEARCH_ASSET_URI,
      SearchResource.RESOURCE_URI + SearchResource.SEARCH_AND_DOWNLOAD_URI);

  @Autowired
  public SearchParameterContributor(final SearchMappingsService searchMappings) {
    super(HTTP_METHODS, PATHS, transformMappings(searchMappings.getAllMappings()));
  }

  private static Collection<QueryParameter> transformMappings(final Iterable<SearchMapping> searchMappings) { // NOSONAR
    return StreamSupport.stream(searchMappings.spliterator(), false)
        .map(m -> (QueryParameter) new QueryParameter()
            .name(m.getAlias())
            .schema(new StringSchema())
            .description(m.getDescription()))
        .collect(toList());
  }
}
