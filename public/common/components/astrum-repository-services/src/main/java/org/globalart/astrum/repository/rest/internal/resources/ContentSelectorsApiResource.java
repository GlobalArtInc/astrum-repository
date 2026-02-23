
package org.globalart.astrum.repository.rest.internal.resources;

import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.selector.ContentSelectorCreatedEvent;
import org.globalart.astrum.selector.ContentSelectorDeletedEvent;
import org.globalart.astrum.selector.ContentSelectorUpdatedEvent;
import org.globalart.astrum.repository.rest.api.ContentSelectorApiCreateRequest;
import org.globalart.astrum.repository.rest.api.ContentSelectorApiResponse;
import org.globalart.astrum.repository.rest.api.ContentSelectorApiUpdateRequest;
import org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc;
import org.globalart.astrum.rest.NotCacheable;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.selector.CselSelector;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorConfigurationStore;
import org.globalart.astrum.selector.SelectorFactory;
import org.globalart.astrum.selector.SelectorManager;
import org.globalart.astrum.validation.Validate;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.singletonMap;
import static java.util.stream.Collectors.toList;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.BAD_REQUEST;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;
import static org.globalart.astrum.selector.SelectorConfiguration.EXPRESSION;

/**
 * @since 3.19
 */
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
public class ContentSelectorsApiResource
    implements Resource, ContentSelectorsResourceDoc
{
  private final SelectorFactory selectorFactory;

  private final SelectorManager selectorManager;

  private final SelectorConfigurationStore store;

  private final EventManager eventManager;

  @Autowired
  public ContentSelectorsApiResource(
      final SelectorFactory selectorFactory,
      final SelectorManager selectorManager,
      final SelectorConfigurationStore store,
      final EventManager eventManager)
  {
    this.selectorFactory = checkNotNull(selectorFactory);
    this.selectorManager = checkNotNull(selectorManager);
    this.store = checkNotNull(store);
    this.eventManager = checkNotNull(eventManager);
  }

  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:selectors:read")
  @NotCacheable
  public List<ContentSelectorApiResponse> getContentSelectors() {
    return store.browse()
        .stream()
        .map(ContentSelectorsApiResource::fromSelectorConfiguration)
        .collect(toList());
  }

  @POST
  @RequiresAuthentication
  @Validate
  @RequiresPermissions("astrum:selectors:create")
  public void createContentSelector(@Valid final ContentSelectorApiCreateRequest request) {
    selectorFactory.validateSelector(CselSelector.TYPE, request.getExpression());
    SelectorConfiguration configuration =
        selectorManager.create(request.getName(), CselSelector.TYPE, request.getDescription(),
            singletonMap(EXPRESSION, request.getExpression()));
    eventManager.post(new ContentSelectorCreatedEvent(configuration));

  }

  @GET
  @Path("{name}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:selectors:read")
  public ContentSelectorApiResponse getContentSelector(@PathParam("name") final String name) {
    SelectorConfiguration configuration = findConfigurationByNameOrThrowNotFound(name);

    return ContentSelectorsApiResource.fromSelectorConfiguration(configuration);
  }

  @PUT
  @Path("{name}")
  @RequiresAuthentication
  @Validate
  @RequiresPermissions("astrum:selectors:update")
  public void updateContentSelector(
      @PathParam("name") final String name,
      @Valid final ContentSelectorApiUpdateRequest request)
  {
    SelectorConfiguration configuration = findConfigurationByNameOrThrowNotFound(name);

    selectorFactory.validateSelector(configuration.getType(), request.getExpression());

    configuration.setDescription(request.getDescription());
    configuration.setAttributes(singletonMap(EXPRESSION, request.getExpression()));
    selectorManager.update(configuration);
    eventManager.post(new ContentSelectorUpdatedEvent(configuration));
  }

  @DELETE
  @Path("{name}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:selectors:delete")
  public void deleteContentSelector(@PathParam("name") final String name) {
    SelectorConfiguration configuration = findConfigurationByNameOrThrowNotFound(name);

    try {
      selectorManager.delete(configuration);
      eventManager.post(new ContentSelectorDeletedEvent(configuration));
    }
    catch (IllegalStateException e) {
      throw new WebApplicationMessageException(BAD_REQUEST, "\"" + e.getMessage() + "\"", APPLICATION_JSON);
    }
  }

  private SelectorConfiguration findConfigurationByNameOrThrowNotFound(final String name) {
    return selectorManager.findByName(name)
        .orElseThrow(() -> new WebApplicationMessageException(NOT_FOUND, "\"No selector found for " + name + '"',
            APPLICATION_JSON));
  }

  private static ContentSelectorApiResponse fromSelectorConfiguration(
      final SelectorConfiguration selectorConfiguration)
  {
    ContentSelectorApiResponse response = new ContentSelectorApiResponse();
    response.setName(selectorConfiguration.getName());
    response.setType(selectorConfiguration.getType());
    response.setDescription(selectorConfiguration.getDescription());
    response.setExpression(selectorConfiguration.getAttributes().get(EXPRESSION));
    return response;
  }

}
