
package org.globalart.astrum.repository.rest.api;

import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadManager;
import org.globalart.astrum.rest.Resource;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static java.util.stream.Collectors.toList;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

@Component
@Path(UploadDefinitionResource.BASE_PATH)
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
public class UploadDefinitionResource
    implements Resource, UploadDefinitionResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String BASE_PATH = V1_API_PREFIX + "/formats";

  private final UploadManager uploadManager;

  @Autowired
  public UploadDefinitionResource(final UploadManager uploadManager) {
    this.uploadManager = checkNotNull(uploadManager);
  }

  @Path("upload-specs")
  @GET
  public List<UploadDefinitionXO> get() {
    log.debug("Get upload definitions for all formats.");

    return uploadManager.getAvailableDefinitions().stream().map(this::from).collect(toList());
  }

  @Path("{format}/upload-specs")
  @GET
  public UploadDefinitionXO get(@PathParam("format") final String format) {
    log.debug("Get upload definition for format '{}'.", format);

    UploadDefinition uploadDefinition = uploadManager.getByFormat(format);

    if (uploadDefinition == null || !uploadDefinition.isApiUpload()) {
      log.debug("Could not find upload definition for format '{}'.", format);
      throw new NotFoundException(format("Unable to locate upload definition for format '%s'", format));
    }

    return from(uploadDefinition);
  }

  private UploadDefinitionXO from(UploadDefinition def) {
    return UploadDefinitionXO.fromWithAssetFileField(def);
  }
}
